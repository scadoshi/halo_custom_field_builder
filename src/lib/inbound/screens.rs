use colored::*;
use log::{error, info};
use std::io::{self, Write};

use crate::domain::import_result::ImportResults;
use crate::domain::models::custom_field::CustomField;
use crate::outbound::field_client::FieldClient;

#[derive(Debug)]
pub enum RunMode {
    Import,
    Debug,
    Quit,
}

#[derive(Debug)]
pub enum DebugAction {
    Process,
    Skip,
    Quit,
}

pub struct ScreenManager {
    fields: Vec<CustomField>,
}

impl ScreenManager {
    pub fn new(fields: Vec<CustomField>) -> Self {
        Self { fields }
    }

    pub fn show_initial_stats(&self, token_type: &str) -> anyhow::Result<()> {
        println!("\n{}", "initial status:".bright_blue().bold());
        println!("{}", "=".repeat(80).bright_blue());

        println!(
            "• authentication: {} (token type: {})",
            "success".bright_green().bold(),
            token_type.bright_yellow()
        );

        println!(
            "• fields loaded: {}",
            self.fields.len().to_string().bright_yellow()
        );

        println!("• status: {}", "ready to process".bright_green().bold());

        println!("{}\n", "=".repeat(80).bright_blue());
        Ok(())
    }

    pub fn get_run_mode(&self) -> anyhow::Result<RunMode> {
        println!("{}", "\navailable operations:".bright_blue().bold());
        println!("{}", "=".repeat(80).bright_blue());

        println!(
            "{}. {}",
            "1".bright_yellow().bold(),
            "import all fields".bright_green()
        );

        println!(
            "{}. {}",
            "2".bright_yellow().bold(),
            "debug mode (field by field)".bright_cyan()
        );

        println!(
            "{}. {}",
            "3".bright_yellow().bold(),
            "quit program".bright_red()
        );

        print!("\n{}", "enter your choice (1-3): ".bright_white().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => {
                println!("\n{}", "selected: import all fields".bright_green());
                Ok(RunMode::Import)
            }
            "2" => {
                println!("\n{}", "selected: debug mode".bright_cyan());
                Ok(RunMode::Debug)
            }
            "3" => {
                println!("\n{}", "selected: quit program".bright_red());
                Ok(RunMode::Quit)
            }
            _ => {
                error!("{}", "invalid selection. please try again.".bright_red());
                self.get_run_mode()
            }
        }
    }

    pub async fn process_all_fields(
        &self,
        field_client: &FieldClient,
    ) -> anyhow::Result<ImportResults> {
        let mut results = ImportResults::new();

        for field in &self.fields {
            match field_client.create_field(field).await {
                Ok(_) => {
                    results.add_success(field.label.to_string());
                    info!("✓ field processed successfully: {}", field.label);
                }
                Err(e) => {
                    results.add_failure(field.label.to_string(), e.to_string());
                    error!("✗ field processing failed: {}", e);
                }
            }
        }

        Ok(results)
    }

    pub async fn debug_mode(&self, field_client: &FieldClient) -> anyhow::Result<ImportResults> {
        info!("\nentering debug mode");
        info!("this mode will process fields one at a time\n");

        let mut results = ImportResults::new();

        for (index, field) in self.fields.iter().enumerate() {
            match self.show_field_debug_prompt(index, field)? {
                DebugAction::Process => {
                    info!("processing field: {}", field.label);

                    match field_client.create_field(field).await {
                        Ok(_) => {
                            results.add_success(field.label.to_string());
                            info!("✓ field processed successfully\n");
                        }
                        Err(e) => {
                            results.add_failure(field.label.to_string(), e.to_string());
                            error!("✗ field processing failed: {}\n", e);
                        }
                    }
                }
                DebugAction::Skip => {
                    info!("skipping field: {}\n", field.label);
                    continue;
                }
                DebugAction::Quit => {
                    info!("debug mode terminated by user");
                    break;
                }
            }
        }

        Ok(results)
    }

    fn show_field_debug_prompt(
        &self,
        index: usize,
        field: &CustomField,
    ) -> anyhow::Result<DebugAction> {
        println!("\n{}", "=".repeat(80).bright_blue());
        println!(
            "{}",
            format!(
                "field {} of {}",
                (index + 1).to_string().bright_yellow(),
                self.fields.len().to_string().bright_yellow()
            )
            .bright_blue()
            .bold()
        );
        println!("{}", "=".repeat(80).bright_blue());

        println!("\n{}", "field details:".bright_blue().bold());
        println!("• label: {}", field.label.to_string().bright_yellow());
        println!("• name: {}", field.name.to_string().bright_yellow());
        println!(
            "• type id: {}",
            field.field_type.field_type_id().to_string().bright_yellow()
        );

        if let Some(input_type_id) = field.field_type.input_type_id() {
            println!(
                "• input type id: {}",
                input_type_id.to_string().bright_yellow()
            );
        }

        if let Some(options) = field.field_type.selection_options_string() {
            println!("• options: {}", options.bright_yellow());
        }

        println!("\n{}", "available actions:".bright_blue().bold());
        println!(
            "{}. {} field",
            "1".bright_yellow().bold(),
            "process".bright_green()
        );
        println!(
            "{}. {} field",
            "2".bright_yellow().bold(),
            "skip".bright_cyan()
        );
        println!(
            "{}. {} debug mode",
            "3".bright_yellow().bold(),
            "quit".bright_red()
        );

        print!("\n{}", "enter your choice (1-3): ".bright_white().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => Ok(DebugAction::Process),
            "2" => Ok(DebugAction::Skip),
            "3" => Ok(DebugAction::Quit),
            _ => {
                error!("{}", "invalid selection. please try again.".bright_red());
                self.show_field_debug_prompt(index, field)
            }
        }
    }
}
