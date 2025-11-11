use chrono::{DateTime, Local};
use colored::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FieldResult {
    pub label: String,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: DateTime<Local>,
}

#[derive(Debug)]
pub struct ImportResults {
    pub successful: Vec<FieldResult>,
    pub failed: Vec<FieldResult>,
}

impl ImportResults {
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
        }
    }

    pub fn add_success(&mut self, label: String) {
        self.successful.push(FieldResult {
            label,
            success: true,
            error: None,
            timestamp: Local::now(),
        });
    }

    pub fn add_failure(&mut self, label: String, error: String) {
        self.failed.push(FieldResult {
            label,
            success: false,
            error: Some(error),
            timestamp: Local::now(),
        });
    }

    pub fn log_summary(&self) {
        println!("\n{}", "Import Summary:".bright_blue().bold());
        println!("{}", "=".repeat(80).bright_blue());

        println!(
            "• Total fields processed: {}",
            (self.successful.len() + self.failed.len())
                .to_string()
                .bright_yellow()
        );
        println!(
            "• Successful imports: {}",
            self.successful.len().to_string().bright_green()
        );
        println!(
            "• Failed imports: {}",
            self.failed.len().to_string().bright_red()
        );

        if !self.failed.is_empty() {
            println!("\n{}", "Failed Fields:".bright_red().bold());
            for result in &self.failed {
                println!(
                    "• {} ({})",
                    result.label.bright_yellow(),
                    result.error.as_ref().unwrap().bright_red()
                );
            }
        }

        println!("{}", "=".repeat(80).bright_blue());
    }
}
