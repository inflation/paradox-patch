#![deny(missing_docs)]
//! A utility to generate dlc list for Paradox games

use std::{
    fs::File,
    io::{BufRead, Write},
    path::PathBuf,
};

use clap::Parser;
use clap_verbosity_flag::Verbosity;
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use log::{info, warn};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// DLC folder
    #[arg(short, long, default_value = "dlc")]
    dlc: PathBuf,
    /// Output file
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// Verbosity
    #[command(flatten)]
    verbose: Verbosity,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let dlc = {
        let mut dlc = args.dlc;
        if !dlc.exists() {
            dlc.pop();
            dlc.push("game/dlc");
            if !dlc.exists() {
                return Err(eyre!("DLC folder does not exist"));
            }
        }
        dlc
    }
    .canonicalize()
    .wrap_err_with(|| eyre!("Failed to find absolute path of the DLC folder"))?;
    info!("DLC folder: {}", dlc.display());

    let parent = dlc
        .parent()
        .ok_or_else(|| eyre!("DLC folder has no parent"))?;

    let target = args
        .output
        .or_else(|| {
            if parent.ends_with("Europa Universalis IV") {
                Some("eu4.app/Contents/Frameworks/steam_settings/DLC.txt")
            } else if parent.ends_with("Hearts of Iron IV") || parent.ends_with("Stellaris") {
                Some("steam_settings/DLC.txt")
            } else if parent.ends_with("game") {
                parent.parent().and_then(|p| {
                    p.ends_with("Crusader Kings III")
                        .then_some("../steam_settings/DLC.txt")
                })
            } else {
                None
            }
            .map(|s| parent.join(s))
        })
        .ok_or_else(|| eyre!("Failed to find output file"))?;
    info!("Output file: {}", target.display());

    target.parent().map(std::fs::create_dir_all).transpose()?;
    let mut output = std::fs::File::create(&target)
        .wrap_err_with(|| eyre!("Failed to create the output file at {}", target.display()))?;

    let mut files = vec![];
    for entry in std::fs::read_dir(&dlc)
        .wrap_err_with(|| eyre!("Failed to read directory: {}", dlc.display()))?
        .flatten()
    {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|e| eyre!("invalid folder name: {:?}", e))?;

                let mut path = entry.path();
                path.push(format!(
                    "{}.dlc",
                    name.split('_')
                        .next()
                        .ok_or_else(|| eyre!("invalid dlc file name: {:?}", name))?
                ));

                if path.exists() {
                    files.push(path);
                } else {
                    warn!("DLC file does not exist: {}", path.display());
                }
            }
        }
    }
    files.sort();

    let count = files
        .into_iter()
        .map(|d| {
            std::fs::File::open(d).map(|d| {
                parse(d).map(|(id, name)| {
                    info!("DLC: id={} name={}", id, name);
                    writeln!(output, "{}={}", id, name)
                })
            })
        })
        .count();

    println!(
        "Found {} DLCs, write to '{}'",
        count,
        target.canonicalize()?.display()
    );

    Ok(())
}

fn parse(file: File) -> Option<(u32, String)> {
    let reader = std::io::BufReader::new(file);
    let mut id = None;
    let mut name = None;

    for line in reader.lines().flatten() {
        let mut split = line.split('=');
        let key = split.next().map(|s| s.trim());
        let val = split.next().map(|s| s.trim().trim_matches('"'));

        if let Some(key) = key {
            if key == "steam_id" {
                id = val.and_then(|s| s.parse().ok());
            } else if key == "name" {
                name = val.map(String::from);
            }
        }
    }

    if id.is_none() || name.is_none() {
        None
    } else {
        id.zip(name)
    }
}
