//! devkit - AI-powered development toolkit
//!
//! From Infer No Dev - Just describe what you want, no manual coding needed.
//! Built in Rust for developers who are too lazy to write code manually.

// Module declarations
mod agents;
mod ai;
mod blueprint;
mod cli;
mod codegen;
mod config;
mod context;
mod interactive;
mod logging;
mod plugins;
mod shell;
mod ui;
mod web;

#[cfg(test)]
mod testing;

// External crate imports
use anyhow::Result;
use tokio;

// Internal imports - use proper CLI module
use crate::cli::{Cli, CliRunner};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments using proper CLI module
    let cli = Cli::parse();

    // Initialize basic tracing for immediate startup logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create CLI runner which handles configuration and command execution
    let mut cli_runner =
        CliRunner::new(&cli).map_err(|e| anyhow::anyhow!("Failed to create CLI runner: {}", e))?;

    // Run the command through the CLI runner
    cli_runner
        .run(cli.command)
        .await
        .map_err(|e| anyhow::anyhow!("Command execution failed: {}", e))?;

    Ok(())
}
