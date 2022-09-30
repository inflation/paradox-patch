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
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let dlc = args.dlc;
    let mut output = std::fs::File::create("DLC.txt")?;
    let mut res = vec![];

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
                if let Some(line) = parse(path)? {
                    res.push(line);
                }
            }
        }
    }

    res.sort_by(|(id1, _), (id2, _)| id1.cmp(id2));
    for (id, name) in res {
        writeln!(output, "{}={}", id, name)?;
    }

    Ok(())
}

fn parse(path: PathBuf) -> Result<Option<(u32, String)>> {
    let file = std::fs::File::open(&path)?;
    let reader = std::io::BufReader::new(file);
    let mut id = 0;
    let mut name = String::new();

    for line in reader.lines() {
        let line = line?;

        let mut split = line.split('=');
        let key = split
            .next()
            .map(|s| s.trim())
            .ok_or_else(|| eyre!("invalid line: {:?}", line))?;
        let val = split
            .next()
            .map(|s| s.trim().trim_matches('"'))
            .ok_or_else(|| eyre!("invalid line: {:?}", line))?;

        if key == "steam_id" {
            id = val.to_string().parse()?;
        } else if key == "name" {
            name = val.to_string();
        }
    }

    if id == 0 || name.is_empty() {
        Ok(None)
    } else {
        Ok(Some((id, name)))
    }
}
