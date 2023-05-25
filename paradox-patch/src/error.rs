use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

/// Errors when generate DLC list
#[derive(Error, Debug, Diagnostic)]
pub enum GenerateError {
    #[error("Failed to find absolute path of the DLC folder: {1}")]
    #[diagnostic(
        code(generate::dlc_folder_not_exists),
        help("Please check if the DLC folder exists")
    )]
    DlcFolderNotExists(#[source] std::io::Error, PathBuf),

    #[error("Invalid output DLC.txt file")]
    #[diagnostic(
        code(generate::unknown_game),
        help("Please specify the output DLC.txt file, if you are not choosing supported Paradox games")
    )]
    InvalidOutput,

    #[error("Failed to read DLC folder: {1}")]
    #[diagnostic(
        code(generate::read_dlc_folder_failed),
        help("Please check if the DLC folder has the right permission")
    )]
    ReadDlcFolderFailed(#[source] std::io::Error, PathBuf),

    #[error("Failed to create output DLC.txt file: {1}")]
    #[diagnostic(
        code(generate::create_output_file_failed),
        help("Please check if the output DLC.txt file exists")
    )]
    CreateOutputFileFailed(#[source] std::io::Error, PathBuf),
}

/// Errors when download patching
#[derive(Error, Debug, Diagnostic)]
pub enum PatchError {
    #[error("Invalid game")]
    #[diagnostic(
        code(patch::game_not_recognized),
        help("Please check if the game is supported")
    )]
    GameNotRecognized,

    #[error("Failed to find absolute path of the libsteam_api.dylib file: {0}")]
    #[diagnostic(
        code(patch::libsteam_api_not_exists),
        help("Please check if the libsteam_api.dylib file exists")
    )]
    LibNotExists(PathBuf),

    #[error("Failed to download libsteam_api.dylib")]
    #[diagnostic(
        code(patch::download_failed),
        help("Please check if you are in China Mainland and try again with --proxy option")
    )]
    DownloadFailed(#[source] reqwest::Error),

    #[error("Failed to backup libsteam_api.dylib to : {0}")]
    #[diagnostic(
        code(patch::backup_failed),
        help("Please check if the backup path is writable")
    )]
    BackupFailed(#[source] std::io::Error, PathBuf),

    #[error("Failed to patch libsteam_api.dylib")]
    #[diagnostic(
        code(patch::patch_failed),
        help("Please check if the operation is permitted")
    )]
    PatchFailed(#[source] std::io::Error),
}
