//! Runtime services used while executing solves.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct RuntimeWorkspace {
    path: PathBuf,
}

impl RuntimeWorkspace {
    pub(crate) fn create(prefix: &str) -> Result<Self, std::io::Error> {
        let path = unique_temp_path(prefix);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for RuntimeWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn unique_temp_path(prefix: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!("arco-{prefix}-{}-{now}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_creates_and_cleans_directory() {
        let path = {
            let workspace = RuntimeWorkspace::create("test-runtime").expect("create workspace");
            let path = workspace.path().to_path_buf();
            assert!(path.exists());
            path
        };

        assert!(!path.exists());
    }
}
