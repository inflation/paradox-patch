use std::process::Command;

#[test]
fn pgo() {
    for arg in ["eu4", "hoi4", "stellaris", "ck3", "-i72850"] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_steam-dlc"));
        cmd.args(["-d", arg]);
        assert!(cmd.status().unwrap().success());
    }
}
