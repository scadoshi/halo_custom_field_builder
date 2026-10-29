use colored::Colorize;

use crate::inbound::screens::RULE;

/// A field Halo refused, with the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub label: String,
    pub error: String,
}

/// The labels of the fields a run created and the fields it failed on.
#[derive(Debug, Default)]
pub struct ImportResults {
    pub successful: Vec<String>,
    pub failed: Vec<Failure>,
}

impl ImportResults {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_success(&mut self, label: String) {
        self.successful.push(label);
    }

    pub fn add_failure(&mut self, label: String, error: String) {
        self.failed.push(Failure { label, error });
    }

    pub fn log_summary(&self) {
        println!("\n{}", "Import Summary:".bright_blue().bold());
        println!("{}", RULE.bright_blue());

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
            for failure in &self.failed {
                println!(
                    "• {} ({})",
                    failure.label.bright_yellow(),
                    failure.error.bright_red()
                );
            }
        }

        println!("{}", RULE.bright_blue());
    }
}
