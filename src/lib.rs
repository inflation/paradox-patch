use std::path::Path;

pub enum Game {
    Eu4,
    Hoi4,
    Stellaris,
    Ck3,
}

pub fn check_game(path: &Path) -> Option<Game> {
    if path.ends_with("Europa Universalis IV") {
        Some(Game::Eu4)
    } else if path.ends_with("Hearts of Iron IV") {
        Some(Game::Hoi4)
    } else if path.ends_with("Stellaris") {
        Some(Game::Stellaris)
    } else if path.ends_with("Crusader Kings III") {
        Some(Game::Ck3)
    } else {
        None
    }
}
