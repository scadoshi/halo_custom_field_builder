//! What a run created and what it failed on.

use colored::Colorize;

use crate::inbound::screens::RULE;

/// A field Halo refused, with the reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The field's label.
    pub label: String,
    /// Why the field was not created.
    pub error: String,
}

/// The labels of the fields a run created and the fields it failed on.
#[derive(Debug, Default)]
pub struct ImportResults {
    /// Labels of the fields created, in the order they were sent.
    pub successful: Vec<String>,
    /// The fields Halo refused, in the order they were sent.
    pub failed: Vec<Failure>,
}

impl ImportResults {
    /// Results with nothing in them yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a created field.
    pub fn add_success(&mut self, label: String) {
        self.successful.push(label);
    }

    /// Records a refused field and the reason.
    pub fn add_failure(&mut self, label: String, error: String) {
        self.failed.push(Failure { label, error });
    }

    /// Prints the totals, then each failed field with its reason.
    #[expect(
        clippy::print_stdout,
        reason = "the menus and summaries are the program's own terminal output"
    )]
    pub fn log_summary(&self) {
        println!("\n{}", "Import Summary:".bright_blue().bold());
        println!("{}", RULE.bright_blue());

        println!(
            "• Total fields processed: {}",
            self.successful
                .len()
                .saturating_add(self.failed.len())
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
