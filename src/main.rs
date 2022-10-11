#![deny(missing_docs)]
//! A utility to generate dlc list for Paradox games

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use color_eyre::Result;
use tracing_log::AsTrace;

mod dlc;
mod patch;

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
        Command::Dlc { target, output } => dlc::generate(target, output),
        Command::Patch { target, proxy } => patch::patch(target, proxy),
    }
}

/// Recognized game list
pub enum Game {
    /// Europa Universalis IV
    Eu4,
    /// Hearts of Iron IV
    Hoi4,
    /// Stellaris
    Stellaris,
    /// Crusader Kings III
    Ck3,
}

/// Check if the game is supported
pub fn check_game(path: &Path) -> Option<Game> {
    if path.ends_with("Europa Universalis IV") {
        Some(Game::Eu4)
    } else if path.ends_with("Hearts of Iron IV") {
        Some(Game::Hoi4)
    } else if path.ends_with("Stellaris") {
        Some(Game::Stellaris)
    } else if path.ends_with("Crusader Kings III") {
        Some(Game::Ck3)
    } else {
        None
    }
}
