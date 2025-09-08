use crate::cli::{CliRunner, ProfileCommands};

pub async fn run(_runner: &mut CliRunner, _command: ProfileCommands) -> Result<(), Box<dyn std::error::Error>> {
    println!("Profile command not yet implemented");
    Ok(())
}
