mod dlc;
mod error;
mod patch;

use std::path::Path;

pub use dlc::generate;
pub use error::GenerateError;
pub use patch::patch;

/// Recognized game list
pub(crate) enum Game {
    /// Europa Universalis IV
    Eu4,
    /// Hearts of Iron IV
    Hoi4,
    /// Stellaris
    Stellaris,
    /// Crusader Kings III
    Ck3,
    /// Victoria 3
    Vic3,
}

/// Check if the game is supported
pub(crate) fn check_game(path: &Path) -> Option<Game> {
    if path.ends_with("Europa Universalis IV") {
        Some(Game::Eu4)
    } else if path.ends_with("Hearts of Iron IV") {
        Some(Game::Hoi4)
    } else if path.ends_with("Stellaris") {
        Some(Game::Stellaris)
    } else if path.ends_with("Crusader Kings III") {
        Some(Game::Ck3)
    } else if path.ends_with("Victoria 3") {
        Some(Game::Vic3)
    } else {
        None
    }
}
