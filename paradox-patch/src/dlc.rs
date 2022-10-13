use std::{
    fs::File,
    io::{BufRead, Write},
    path::PathBuf,
};

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use tracing::{info, warn};

use crate::{check_game, Game};

pub fn generate(target: Option<PathBuf>, output: Option<PathBuf>) -> Result<()> {
    let target = target.unwrap_or_default();
    let mut dlc = target.join("dlc");

    let dlc = {
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

    let output = output
        .or_else(|| {
            check_game(&target)
                .map(|g| match g {
                    Game::Eu4 => "eu4.app/Contents/Frameworks/steam_settings/DLC.txt",
                    Game::Ck3 | Game::Hoi4 | Game::Stellaris => "steam_settings/DLC.txt",
                })
                .map(|s| target.join(s))
        })
        .ok_or_else(|| eyre!("Failed to find output file"))?;
    info!("Output file: {}", output.display());

    output.parent().map(std::fs::create_dir_all).transpose()?;
    let mut output_file = std::fs::File::create(&output)
        .wrap_err_with(|| eyre!("Failed to create the output file: '{}'", output.display()))?;

    let mut files = vec![];
    for entry in std::fs::read_dir(&dlc)
        .wrap_err_with(|| eyre!("Failed to read directory: '{}'", dlc.display()))?
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
                    writeln!(output_file, "{}={}", id, name)
                })
            })
        })
        .count();

    println!("Found {} DLCs, write to '{}'", count, output.display());

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
