use std::{
    io::{BufRead, Write},
    path::PathBuf,
};

use clap::Parser;
use color_eyre::{eyre::eyre, Result};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// DLC folder
    dlc: PathBuf,
    /// Output file
    #[arg(short, long, default_value = "DLC.txt")]
    output: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let dlc = args.dlc;
    let mut output = std::fs::File::create(args.output)?;

    let mut files = vec![];
    for entry in std::fs::read_dir(dlc)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|e| eyre!("invalid file name: {:?}", e))?;

            let mut path = entry.path();
            path.push(format!(
                "{}.dlc",
                name.split('_')
                    .next()
                    .ok_or_else(|| eyre!("invalid file name: {:?}", name))?
            ));

            if path.exists() {
                files.push(path);
            }
        }
    }
    files.sort();

    for dlc in files {
        if let Some((id, name)) = parse(dlc) {
            writeln!(output, "{}={}", id, name)?;
        }
    }

    Ok(())
}

fn parse(path: PathBuf) -> Option<(u32, String)> {
    let file = std::fs::File::open(&path).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut id = 0;
    let mut name = String::new();

    for line in reader.lines() {
        let line = line.unwrap();

        let mut split = line.split('=');
        let key = split.next().map(|s| s.trim()).unwrap();
        let val = split.next().map(|s| s.trim().trim_matches('"')).unwrap();

        if key == "steam_id" {
            id = val.to_string().parse().unwrap();
        } else if key == "name" {
            name = val.to_string();
        }
    }

    if id == 0 || name.is_empty() {
        None
    } else {
        Some((id, name))
    }
}
