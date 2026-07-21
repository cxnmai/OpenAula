mod args;
mod commands;
mod parse;

use anyhow::Result;
use clap::Parser;

use crate::args::Cli;

fn main() -> Result<()> {
    commands::run(Cli::parse())
}
