use std::{ffi::OsStr, io::Write, os::unix::ffi::OsStrExt, path::PathBuf};

use miette::{NamedSource, SourceSpan};
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
                    Game::Vic3 => "binaries/steam_settings/DLC.txt",
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

                let mut path = entry.path();
                if let Some(id) = name.as_bytes().split(|&c| c == b'_').next() {
                    path.push(OsStr::from_bytes(id));
                    path.set_extension("dlc");

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

    let count = files.len();
    for file in files {
        let (id, name) = parse(file)?;
        info!("DLC: id='{id}' name='{name}'");
        _ = writeln!(output_file, "{id}={name}");
    }

    eprintln!("Found {count} DLCs, write to: '{}'", output.display());

    Ok(())
}

fn parse(path: PathBuf) -> Result<(u32, String), GenerateError> {
    let file = std::fs::read_to_string(&path)
        .map_err(|e| GenerateError::ReadDlcFileFailed(e, path.clone()))?;
    let mut id = None;
    let mut name = None;

    for line in file.lines() {
        let mut split = line.split(" = ");
        let (key, val) = (split.next(), split.next());

        if let Some(key) = key {
            if key == "name" {
                name = val.map(|s| String::from(s.trim_matches('"')));
            } else if key == "steam_id" {
                id = val
                    .map(|s| {
                        s.trim_matches('"').parse().map_err(|_| {
                            let idx = val.unwrap().as_ptr() as usize - file.as_ptr() as usize;
                            GenerateError::ParseDlcFailed(
                                String::from("Wrong number format"),
                                NamedSource::new(format!("{}", path.display()), file.clone()),
                                Some(SourceSpan::new(idx.into(), s.len().into())),
                            )
                        })
                    })
                    .transpose()?;
            }
        }
    }

    id.zip(name).ok_or_else(|| {
        GenerateError::ParseDlcFailed(
            String::from("Cannot find 'steam_id' or 'name'"),
            NamedSource::new(format!("{}", path.display()), file.clone()),
            None,
        )
    })
}
