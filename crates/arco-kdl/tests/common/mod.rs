use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn temp_root(prefix: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos()
        .to_string();
    let root = std::env::temp_dir().join(format!("arco-kdl-{prefix}-{unique}"));
    fs::create_dir_all(&root)?;
    Ok(root)
}
