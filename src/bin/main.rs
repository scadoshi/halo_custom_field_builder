use std::path::Path;
use std::process::ExitCode;

use halo_custom_field_builder::config::Config;
use halo_custom_field_builder::domain::logging;
use halo_custom_field_builder::inbound::screens::{Menu, RULE, RunMode};
use halo_custom_field_builder::inbound::{readers, screens};
use halo_custom_field_builder::outbound::auth::client::AuthClient;
use halo_custom_field_builder::outbound::field_client::FieldClient;
use log::{error, info};

async fn run() -> anyhow::Result<()> {
    info!("starting application...\n");

    info!("loading configuration...");
    let config = Config::load_from_env()?;
    info!("✓ configuration loaded successfully\n");

    info!("authenticating with api...");
    let mut auth_client = AuthClient::new(&config);

    let token = auth_client.fetch_token().await.map_err(|e| {
        error!("✗ authentication failed: {e}");
        e
    })?;

    info!("✓ authentication successful");
    info!("✓ token acquired and valid\n");

    info!("reading csv file...");
    let fields = readers::read_fields(Path::new(&config.source_file_name))?;
    info!("✓ successfully validated {} fields\n", fields.len());

    let menu = Menu::new(fields);
    menu.show_initial_stats(token.split_whitespace().next().unwrap_or("unknown"));

    let mut field_client = FieldClient::new(&config, auth_client);

    match screens::prompt_run_mode()? {
        RunMode::Import => {
            info!("\n{RULE}");
            info!("starting full import mode");
            info!("{RULE}\n");

            let results = menu.process_all_fields(&mut field_client).await;
            results.log_summary();
        }
        RunMode::Debug => {
            info!("\n{RULE}");
            info!("starting debug mode");
            info!("{RULE}\n");

            let results = menu.debug_mode(&mut field_client).await?;
            results.log_summary();
        }
        RunMode::Quit => {
            info!("program terminated by user");
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    // Without a logger, error! prints nothing, so this failure goes straight to stderr.
    if let Err(e) = logging::setup_logging() {
        eprintln!("✗ error: {e:#}");
        return ExitCode::FAILURE;
    }
    if let Err(e) = run().await {
        error!("\n✗ error: {e:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
