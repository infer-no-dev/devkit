use crossterm::style::Stylize;
use std::io::{self, Write};

use crate::cli::CliRunner;

/// Interactive development mode
pub struct InteractiveMode {
    runner: CliRunner,
}

impl InteractiveMode {
    pub fn new(runner: CliRunner) -> Self {
        Self { runner }
    }

    /// Start the interactive development session
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "{}",
            "🚀 Starting Interactive Development Mode".green().bold()
        );
        println!("Type 'help' for available commands or 'quit' to exit.\n");

        loop {
            print!("{} ", "agentic>".cyan().bold());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match input {
                "quit" | "exit" => {
                    println!("{}", "👋 Goodbye!".green());
                    break;
                }
                "help" => {
                    self.show_help();
                }
                _ => {
                    // TODO: Implement command parsing and execution
                    println!("{} {}", "Command not implemented:".yellow(), input);
                    println!("Available commands: help, quit, exit");
                }
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("\n{}", "Available Commands:".bold().underlined());
        println!("  {} - Show this help message", "help".cyan());
        println!("  {} - Exit interactive mode", "quit/exit".cyan());
        println!("\n{}", "Coming Soon:".bold());
        println!("  {} - Analyze current codebase", "analyze".dim());
        println!(
            "  {} - Generate code from description",
            "generate <prompt>".dim()
        );
        println!("  {} - Get project status", "status".dim());
        println!("  {} - Show configuration", "config".dim());
        println!();
    }
}
