//! A local HTTP server for tests: answers requests with canned replies, in order, and records what was asked.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use url::Url;

use crate::config::Config;

/// What the server was asked.
#[derive(Debug, Clone)]
pub(crate) struct Request {
    /// The request line, such as `POST /auth/token HTTP/1.1`.
    pub(crate) line: String,
    /// The header lines after the request line.
    pub(crate) headers: Vec<String>,
    /// The request body as text.
    pub(crate) body: String,
}

impl Request {
    /// The value of the header `name`, matched without regard to case.
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name).then_some(value.trim())
        })
    }
}

/// How the server answers one request.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Reply {
    /// A JSON response with this status and body.
    Json(u16, &'static str),
}

/// A request beyond the scripted replies gets this status.
const UNEXPECTED_STATUS: u16 = 599;

/// How long a test waits for the server to see the requests it expects.
const PATIENCE: Duration = Duration::from_secs(10);

/// A server on a free local port that answers the nth request with the nth
/// reply. A request past the last reply is recorded and answered with status
/// 599, so a test can tell the client asked for more than it should.
pub(crate) struct TestServer {
    base_url: String,
    requests: Arc<Mutex<Vec<Request>>>,
    task: JoinHandle<()>,
}

impl TestServer {
    pub(crate) async fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base_url = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);
        let task = tokio::spawn(async move {
            let mut replies = replies.into_iter();
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let reply = replies
                    .next()
                    .unwrap_or(Reply::Json(UNEXPECTED_STATUS, "{}"));
                tokio::spawn(answer(socket, reply, Arc::clone(&recorded)));
            }
        });
        Self {
            base_url,
            requests,
            task,
        }
    }

    /// A config pointing at this server.
    pub(crate) fn config(&self) -> Config {
        let base_url = Url::parse(&self.base_url).unwrap();
        Config {
            token_url: base_url.join("auth/token").unwrap(),
            api_url: base_url.join("api").unwrap(),
            base_url,
            client_id: "client-id".to_string(),
            client_secret: CLIENT_SECRET.to_string(),
            source_file_name: "source.csv".to_string(),
        }
    }

    /// Every request the server has seen so far.
    pub(crate) fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }

    /// Waits until the server has seen `count` requests and returns them.
    /// Panics instead of hanging when they never come.
    pub(crate) async fn wait_for(&self, count: usize) -> Vec<Request> {
        tokio::time::timeout(PATIENCE, async {
            loop {
                let requests = self.requests();
                if requests.len() >= count {
                    return requests;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("the server did not get the requests it expected")
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Reads one request from `socket`, records it, and answers with `reply`.
async fn answer(mut socket: TcpStream, reply: Reply, recorded: Arc<Mutex<Vec<Request>>>) {
    let Some(request) = read_request(&mut socket).await else {
        return;
    };
    recorded.lock().unwrap().push(request);
    let Reply::Json(status, body) = reply;
    let response = format!(
        "HTTP/1.1 {status} Test\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = socket.write_all(response.as_bytes()).await;
}

/// Reads a request head and its body. `None` when the client closes the
/// connection first.
async fn read_request(socket: &mut TcpStream) -> Option<Request> {
    let mut request = Vec::new();
    let mut buf = [0; 4096];
    let body_start = loop {
        let n = socket.read(&mut buf).await.ok()?;
        if n == 0 {
            return None;
        }
        request.extend_from_slice(&buf[..n]);
        if let Some(i) = request.windows(4).position(|w| w == b"\r\n\r\n") {
            break i.saturating_add(4);
        }
    };
    let head = String::from_utf8_lossy(&request[..body_start]).into_owned();
    let content_length: usize = head
        .to_lowercase()
        .lines()
        .find_map(|line| line.strip_prefix("content-length:"))
        .map_or(0, |len| len.trim().parse().unwrap());
    while request.len() < body_start.saturating_add(content_length) {
        let n = socket.read(&mut buf).await.ok()?;
        if n == 0 {
            return None;
        }
        request.extend_from_slice(&buf[..n]);
    }
    let mut lines = head.lines();
    Some(Request {
        line: lines.next().unwrap_or_default().to_owned(),
        headers: lines
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect(),
        body: String::from_utf8_lossy(&request[body_start..]).into_owned(),
    })
}

/// The client secret in [`TestServer::config`].
pub(crate) const CLIENT_SECRET: &str = "client-secret-value";

/// A token response the auth client accepts.
pub(crate) const TOKEN: Reply = Reply::Json(
    200,
    r#"{"access_token":"tok","token_type":"Bearer","expires_in":3600}"#,
);

/// The request lines, in order.
pub(crate) fn lines(requests: &[Request]) -> Vec<String> {
    requests.iter().map(|r| r.line.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_server_answers_in_order_and_records_each_request() {
        let server = TestServer::start(vec![TOKEN]).await;
        let config = server.config();
        let client = reqwest::Client::new();
        let first = client
            .post(config.token_url.as_str())
            .header("X-Probe", "one")
            .body("hello")
            .send()
            .await
            .unwrap();
        assert_eq!(first.status(), 200);
        assert!(first.text().await.unwrap().contains("tok"));
        let second = client
            .post(format!("{}/fieldinfo", config.api_url))
            .send()
            .await
            .unwrap();
        assert_eq!(second.status().as_u16(), UNEXPECTED_STATUS);

        let requests = server.wait_for(2).await;
        assert_eq!(
            lines(&requests),
            ["POST /auth/token HTTP/1.1", "POST /api/fieldinfo HTTP/1.1"]
        );
        assert_eq!(requests[0].header("x-probe"), Some("one"));
        assert_eq!(requests[0].body, "hello");
    }
}
