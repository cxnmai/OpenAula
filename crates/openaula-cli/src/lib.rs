//! Command-line frontend for OpenAula.

mod args;
mod commands;
mod parse;

use anyhow::Result;
use clap::Parser;

use crate::args::Cli;

/// Parse command-line arguments and execute the selected operation.
pub fn run() -> Result<()> {
    commands::run(Cli::parse())
}
