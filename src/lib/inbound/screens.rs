use std::io::{self, BufRead, Write};

use colored::Colorize;
use log::{error, info};

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

pub struct Menu {
    fields: Vec<CustomField>,
}

impl Menu {
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

    pub fn prompt_run_mode(&self) -> anyhow::Result<RunMode> {
        self.run_mode_from(&mut io::stdin().lock())
    }

    fn run_mode_from(&self, input: &mut impl BufRead) -> anyhow::Result<RunMode> {
        loop {
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

            let Some(choice) = read_choice(input)? else {
                return Ok(RunMode::Quit);
            };
            match choice.as_str() {
                "1" => {
                    println!("\n{}", "selected: import all fields".bright_green());
                    return Ok(RunMode::Import);
                }
                "2" => {
                    println!("\n{}", "selected: debug mode".bright_cyan());
                    return Ok(RunMode::Debug);
                }
                "3" => {
                    println!("\n{}", "selected: quit program".bright_red());
                    return Ok(RunMode::Quit);
                }
                _ => error!("{}", "invalid selection. please try again.".bright_red()),
            }
        }
    }

    pub async fn process_all_fields(
        &self,
        field_client: &mut FieldClient,
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

    pub async fn debug_mode(
        &self,
        field_client: &mut FieldClient,
    ) -> anyhow::Result<ImportResults> {
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
        self.debug_action_from(index, field, &mut io::stdin().lock())
    }

    fn debug_action_from(
        &self,
        index: usize,
        field: &CustomField,
        input: &mut impl BufRead,
    ) -> anyhow::Result<DebugAction> {
        loop {
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

            let Some(choice) = read_choice(input)? else {
                return Ok(DebugAction::Quit);
            };
            match choice.as_str() {
                "1" => return Ok(DebugAction::Process),
                "2" => return Ok(DebugAction::Skip),
                "3" => return Ok(DebugAction::Quit),
                _ => error!("{}", "invalid selection. please try again.".bright_red()),
            }
        }
    }
}

/// Reads one line from `input`, trimmed. `None` at end of input.
fn read_choice(input: &mut impl BufRead) -> io::Result<Option<String>> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> Menu {
        Menu::new(vec![field()])
    }

    fn field() -> CustomField {
        CustomField::new("notes", "notes", 1, None, None).unwrap()
    }

    fn run_mode(input: &str) -> RunMode {
        menu().run_mode_from(&mut input.as_bytes()).unwrap()
    }

    fn debug_action(input: &str) -> DebugAction {
        menu()
            .debug_action_from(0, &field(), &mut input.as_bytes())
            .unwrap()
    }

    #[test]
    fn the_run_mode_menu_reads_a_choice_and_asks_again_after_a_bad_one() {
        assert!(matches!(run_mode("1\n"), RunMode::Import));
        assert!(matches!(run_mode(" 2 \r\n"), RunMode::Debug));
        assert!(matches!(run_mode("3\n"), RunMode::Quit));
        assert!(matches!(run_mode("x\n\n1\n"), RunMode::Import));
        assert!(matches!(run_mode("2"), RunMode::Debug), "no final newline");
    }

    #[test]
    fn the_run_mode_menu_quits_at_end_of_input() {
        assert!(matches!(run_mode(""), RunMode::Quit));
        assert!(matches!(run_mode("x\n"), RunMode::Quit));
    }

    #[test]
    fn the_debug_menu_reads_a_choice_and_asks_again_after_a_bad_one() {
        assert!(matches!(debug_action("1\n"), DebugAction::Process));
        assert!(matches!(debug_action("2\n"), DebugAction::Skip));
        assert!(matches!(debug_action("3\n"), DebugAction::Quit));
        assert!(matches!(debug_action("4\n1\n"), DebugAction::Process));
    }

    #[test]
    fn the_debug_menu_quits_at_end_of_input() {
        assert!(matches!(debug_action(""), DebugAction::Quit));
        assert!(matches!(debug_action("x\n"), DebugAction::Quit));
    }

    #[test]
    fn a_long_run_of_bad_choices_does_not_overflow_the_stack() {
        let input = format!("{}1\n", "x\n".repeat(100_000));
        assert!(matches!(run_mode(&input), RunMode::Import));
    }
}
