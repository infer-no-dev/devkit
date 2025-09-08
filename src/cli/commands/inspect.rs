use crate::cli::{CliRunner, InspectCommands};

pub async fn run(_runner: &mut CliRunner, _command: InspectCommands) -> Result<(), Box<dyn std::error::Error>> {
    println!("Inspect command not yet implemented");
    Ok(())
}
