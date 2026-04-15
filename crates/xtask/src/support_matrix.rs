use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {}

pub fn run(_args: Args) -> Result<(), String> {
    Err("support-matrix is reserved for later implementation".to_string())
}
