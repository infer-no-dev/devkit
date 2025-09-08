use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use tempfile::TempDir;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct CompoundInterestCalculator {
    principal: f64,
    interest_rate: f64,
    time: u32,
}

impl CompoundInterestCalculator {
    pub fn new(principal: f64, interest_rate: f64, time: u32) -> Self {
        Self { principal, interest_rate, time }
    }

    async fn calculate(&self) -> (f64, f64) {
        let compound_interest = self.principal * ((1.0 + self.interest_rate / 100.0) as f64).powi(1 as i32 * self.time);
        (compound_interest, self.principal)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let project_path = create_sample_project(temp_dir.path());
    add(project_path)?;

    let calculator = CompoundInterestCalculator::new(1000.0, 5.0, 10);
    let (compound_interest, principal) = calculator.calculate().await;
    println!("Compound interest: {:.2}", compound_interest);
    println!("Principal: {:.2}", principal);

    Ok(())
}