use arco_kdl::source::parse_program_file;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_kdl_files(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if path.ends_with(".git") || path.ends_with("target") || path.ends_with(".worktrees") {
                continue;
            }
            collect_kdl_files(&path, files)?;
            continue;
        }

        if path.extension().is_some_and(|ext| ext == "kdl") {
            files.push(path);
        }
    }

    Ok(())
}

#[test]
fn all_repo_kdl_files_parse_with_arco_parser() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("failed to locate repo root")?
        .to_path_buf();

    let mut kdl_files = Vec::new();
    collect_kdl_files(&repo_root, &mut kdl_files)?;
    kdl_files.sort();

    assert!(
        !kdl_files.is_empty(),
        "expected at least one .kdl file in repo"
    );

    let mut failures = Vec::new();

    for file in &kdl_files {
        if let Err(error) = parse_program_file(file) {
            failures.push(format!("{}: {error}", file.display()));
        }
    }

    assert!(
        failures.is_empty(),
        "failed to parse {} KDL file(s):\n{}",
        failures.len(),
        failures.join("\n")
    );

    Ok(())
}
