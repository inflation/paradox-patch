#![deny(missing_docs)]
//! A utility to generate dlc list for Paradox games

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use color_eyre::Result;
use paradox_patch::{generate, patch};
use tracing_log::AsTrace;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Command
    #[command(subcommand)]
    command: Command,
    /// Verbosity
    #[command(flatten)]
    verbose: Verbosity,
}

#[derive(Subcommand)]
enum Command {
    /// Generate DLC list
    Dlc {
        /// Target folder
        target: Option<PathBuf>,
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Patch libsteam_api.dylib using goldberg_emulator
    Patch {
        /// Target folder
        target: Option<PathBuf>,
        /// Use proxy to download libsteam_api.dylib in China Mainland
        #[arg(short, long)]
        proxy: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbose.log_level_filter().as_trace())
        .init();

    match args.command {
        Command::Dlc { target, output } => generate(target, output),
        Command::Patch { target, proxy } => patch(target, proxy),
    }
}
