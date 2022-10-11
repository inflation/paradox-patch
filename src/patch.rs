use std::path::PathBuf;

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use paradox_patch::{check_game, Game};
use tracing::{info, warn};

static URL: &str =
    "https://github.com/inflation/goldberg_emulator/releases/download/8b9ce58/libsteam_api.dylib";
static PROXY_URL: &str =
    "https://ghproxy.com/https://github.com/inflation/goldberg_emulator/releases/download/8b9ce58/libsteam_api.dylib";

pub fn patch(target: Option<PathBuf>, proxy: bool) -> Result<()> {
    let url = if proxy { PROXY_URL } else { URL };

    let target_folder = target.unwrap_or_default();
    let lib_file = target_folder.join(
        match check_game(&target_folder)
            .ok_or_else(|| eyre!("Game '{}' not recognized", target_folder.display()))?
        {
            Game::Eu4 => "eu4.app/Contents/Frameworks/libsteam_api.dylib",
            Game::Hoi4 | Game::Stellaris | Game::Ck3 => "libsteam_api.dylib",
        },
    );
    if !lib_file.exists() {
        return Err(eyre!(
            "File '{}' not found, are you sure this is a valid game folder?",
            lib_file.display()
        ));
    }
    info!("Patching '{}'...", lib_file.display());

    let result = reqwest::blocking::get(url)
        .wrap_err_with(|| eyre!("Failed to fetch library"))?
        .bytes()
        .wrap_err_with(|| eyre!("Failed to fetch result"))?;

    let mut backup_file = lib_file.clone();
    backup_file.set_extension("bak");
    if !backup_file.exists() {
        std::fs::rename(&lib_file, &backup_file)
            .wrap_err_with(|| eyre!("Failed to backup file to: '{}'", backup_file.display()))?;
    } else {
        warn!(
            "Library file '{}' already backed up to '{}'. Skipping...",
            lib_file.display(),
            &backup_file.display()
        );
    }

    std::fs::write(&lib_file, result)
        .wrap_err_with(|| eyre!("Failed to write result to file: '{}'", lib_file.display()))?;

    println!("Patched '{}' successfully!", lib_file.display());

    Ok(())
}
