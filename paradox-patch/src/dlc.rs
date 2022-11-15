use std::{
    fs::File,
    io::{BufRead, Write},
    path::PathBuf,
};

use tracing::{info, warn};

use crate::{check_game, Game, GenerateError};

pub fn generate(target: Option<PathBuf>, output: Option<PathBuf>) -> Result<(), GenerateError> {
    let target = target.unwrap_or_default();

    let dlc = {
        let mut dlc = target.join("dlc");
        if !dlc.exists() {
            dlc.pop();
            dlc.push("game/dlc");
        }
        dlc
    };

    let dlc_target = dlc
        .clone()
        .canonicalize()
        .map_err(|e| GenerateError::DlcFolderNotExists(e, dlc))?;
    info!("DLC folder: {}", dlc_target.display());

    let output = output
        .or_else(|| {
            check_game(&target)
                .map(|g| match g {
                    Game::Eu4 => "eu4.app/Contents/Frameworks/steam_settings/DLC.txt",
                    Game::Ck3 | Game::Hoi4 | Game::Stellaris => "steam_settings/DLC.txt",
                })
                .map(|s| target.join(s))
        })
        .ok_or(GenerateError::InvalidOutput)?;
    info!("Output file: {}", output.display());

    let mut output_file = output
        .parent()
        .map(std::fs::create_dir_all)
        .transpose()
        .and_then(|_| std::fs::File::create(&output))
        .map_err(|e| GenerateError::CreateOutputFileFailed(e, output.clone()))?;

    let mut files = vec![];
    for entry in std::fs::read_dir(&dlc_target)
        .map_err(|e| GenerateError::ReadDlcFolderFailed(e, dlc_target))?
        .flatten()
    {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                let name = entry.file_name();
                let name = name.to_string_lossy();

                let mut path = entry.path();
                if let Some(id) = name.split('_').next() {
                    path.push(format!("{}.dlc", id));
                    if path.exists() {
                        files.push(path);
                    } else {
                        warn!("DLC file does not exist: {}", path.display());
                    }
                } else {
                    warn!("Invalid DLC folder: {}", path.display());
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
                    info!("DLC: id='{}' name='{}'", id, name);
                    writeln!(output_file, "{}={}", id, name)
                })
            })
        })
        .count();

    eprintln!("Found {} DLCs, write to: '{}'", count, output.display());

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
