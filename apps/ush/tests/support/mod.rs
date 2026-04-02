use std::{env, fs, path::PathBuf};

pub fn assert_snapshot(path: &str, actual: &str) {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(path);
    if env::var_os("UPDATE_SNAPSHOTS").is_some() {
        if let Some(parent) = fixture.parent() {
            fs::create_dir_all(parent).expect("create snapshot directory");
        }
        fs::write(&fixture, actual).expect("write snapshot");
    }
    let expected = fs::read_to_string(&fixture)
        .unwrap_or_else(|err| panic!("failed to read snapshot {}: {err}", fixture.display()));
    assert_eq!(actual, expected, "snapshot mismatch: {}", fixture.display());
}
