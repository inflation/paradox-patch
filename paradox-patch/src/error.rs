use std::path::PathBuf;

use miette::{Diagnostic, NamedSource, SourceSpan};
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

    #[error("Failed to read DLC file: {1}")]
    #[diagnostic(
        code(generate::read_dlc_file_failed),
        help("Please check if the DLC file has the right permission")
    )]
    ReadDlcFileFailed(#[source] std::io::Error, PathBuf),

    #[error("Failed to parse the DLC file: {0}")]
    #[diagnostic(
        code(generate::failed_to_parse_dlc),
        help("Please check if the DLC file has the correct format")
    )]
    ParseDlcFailed(
        String,
        #[source_code] NamedSource,
        #[label("{0}")] Option<SourceSpan>,
    ),

    #[error("Failed to write output to: {1}")]
    #[diagnostic(
        code(generate::create_output_file_failed),
        help("Please check if the output DLC.txt is writable")
    )]
    WriteOutputFileFailed(#[source] std::io::Error, PathBuf),
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
