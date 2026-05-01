#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn temp_root(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-kdl-{prefix}-{unique}"));
    fs::create_dir_all(&root)?;
    Ok(root)
}

pub(crate) fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(relative)
}

pub(crate) fn fixture_text(relative: &str) -> Result<String, Box<dyn std::error::Error>> {
    let path = fixture_path(relative);
    Ok(fs::read_to_string(path)?)
}

pub(crate) fn write_fixture_to_path(
    relative: &str,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = fixture_path(relative);
    let _ = fs::copy(source, destination)?;
    Ok(())
}
