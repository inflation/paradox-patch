use std::{borrow::Cow, path::PathBuf};

use tracing::{info, warn};

use crate::{check_game, error::PatchError, Game};

const URL: &str =
    "https://github.com/inflation/goldberg_emulator/releases/download/latest/libsteam_api.dylib";
const PROXY_PREFIX: &str = "https://ghproxy.com/";

pub fn patch(target: Option<PathBuf>, proxy: bool) -> Result<(), PatchError> {
    let url = if proxy {
        info!("Using proxy...");
        Cow::from([PROXY_PREFIX, URL].concat())
    } else {
        Cow::from(URL)
    };

    let target_folder = target.unwrap_or_default();
    let lib_file = target_folder.join(
        match check_game(&target_folder).ok_or(PatchError::GameNotRecognized)? {
            Game::Eu4 => "eu4.app/Contents/Frameworks/libsteam_api.dylib",
            Game::Hoi4 | Game::Stellaris | Game::Ck3 => "libsteam_api.dylib",
        },
    );
    if !lib_file.exists() {
        return Err(PatchError::LibNotExists(lib_file));
    }

    info!("Patching '{}'...", lib_file.display());

    let result = reqwest::blocking::get(url.as_ref())
        .and_then(|d| d.bytes())
        .map_err(PatchError::DownloadFailed)?;

    let mut backup_file = lib_file.clone();
    backup_file.set_extension("bak");
    if !backup_file.exists() {
        std::fs::rename(&lib_file, &backup_file)
            .map_err(|e| PatchError::BackupFailed(e, backup_file))?;
    } else {
        warn!(
            "Library file '{}' already backed up to '{}'. Skipping...",
            lib_file.display(),
            backup_file.display()
        );
    }

    std::fs::write(&lib_file, result).map_err(PatchError::PatchFailed)?;

    println!("Patched '{}' successfully!", lib_file.display());

    Ok(())
}
