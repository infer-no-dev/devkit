use crate::cli::{CliRunner, ShellCommands};

pub async fn run(_runner: &mut CliRunner, _command: ShellCommands) -> Result<(), Box<dyn std::error::Error>> {
    println!("Shell command not yet implemented");
    Ok(())
}
