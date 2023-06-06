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
    info!(dlc = ?dlc_target, "DLC folder");

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
    info!(?output, "Output file");

    let mut output_file = output
        .parent()
        .map(std::fs::create_dir_all)
        .transpose()
        .and_then(|_| std::fs::File::create(&output))
        .map_err(|e| GenerateError::WriteOutputFileFailed(e, output.clone()))?;

    let mut files: Vec<PathBuf> = std::fs::read_dir(&dlc_target)
        .map_err(|e| GenerateError::ReadDlcFolderFailed(e, dlc_target))?
        .flatten()
        .filter_map(|entry| {
            entry.file_type().ok().and_then(|t| {
                if t.is_dir() {
                    let name = entry.file_name();

                    let mut path = entry.path();
                    if let Some(id) = name.as_bytes().split(|&c| c == b'_').next() {
                        path.push(OsStr::from_bytes(id));
                        path.set_extension("dlc");

                        if path.exists() {
                            return Some(path);
                        } else {
                            warn!(?path, "DLC file does not exist");
                        }
                    } else {
                        warn!(?path, "Invalid DLC folder");
                    }
                }

                None
            })
        })
        .collect();
    files.sort();

    let count = files.len();
    let output_str = files
        .into_iter()
        .map(|f| parse(f).map(|(id, name)| format!("{id}={name}")))
        .collect::<Result<Vec<_>, GenerateError>>()?
        .join("\n");
    writeln!(output_file, "{}", output_str)
        .map_err(|e| GenerateError::WriteOutputFileFailed(e, output.clone()))?;
    println!("Found {count} DLCs, write to: '{output:?}'");

    Ok(())
}

fn parse(path: PathBuf) -> Result<(u32, String), GenerateError> {
    let file = std::fs::read_to_string(&path)
        .map_err(|e| GenerateError::ReadDlcFileFailed(e, path.clone()))?;

    let (id, name) = file.lines().try_fold((0, String::new()), |mut acc, line| {
        let mut split = line.split(" = ");

        if let (Some(key), Some(val)) = (split.next(), split.next()) {
            if key == "name" {
                acc.1 = String::from(val.trim_matches('"'));
            } else if key == "steam_id" {
                acc.0 = val.trim_matches('"').parse().map_err(|_| {
                    let idx = val.as_ptr() as usize - file.as_ptr() as usize;
                    GenerateError::ParseDlcFailed(
                        String::from("Wrong number format"),
                        NamedSource::new(format!("{path:?}"), file.clone()),
                        Some(SourceSpan::new(idx.into(), val.len().into())),
                    )
                })?
            }
        }

        Ok(acc)
    })?;

    if id == 0 || name.is_empty() {
        return Err(GenerateError::ParseDlcFailed(
            String::from("Cannot find 'steam_id' or 'name'"),
            NamedSource::new(format!("{path:?}"), file),
            None,
        ));
    }
    info!(id, name, "Found DLC");
    Ok((id, name))
}
