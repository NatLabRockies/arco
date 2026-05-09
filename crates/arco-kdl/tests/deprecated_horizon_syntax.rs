use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn repo_examples_and_docs_do_not_use_deprecated_horizon_syntax() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve");
    let roots = [
        repo_root.join("docs"),
        repo_root.join("examples"),
        repo_root.join("crates/arco-kdl/tests/fixtures"),
    ];
    let forbidden = [
        format!("from=\"{}\"", "horizon"),
        ["from", "=", "horizon"].concat(),
        ["horizon", " steps="].concat(),
    ];
    let mut offenders = Vec::new();

    for root in roots {
        collect_offenders(&root, &forbidden, &mut offenders);
    }

    assert!(
        offenders.is_empty(),
        "deprecated horizon syntax found in:\n{}",
        offenders.join("\n")
    );
}

fn collect_offenders(root: &Path, forbidden: &[String], offenders: &mut Vec<String>) {
    if root.is_file() {
        record_if_offending(root, forbidden, offenders);
        return;
    }

    let entries = fs::read_dir(root).expect("directory should be readable");
    for entry in entries {
        let path = entry.expect("directory entry should load").path();
        if path.is_dir() {
            collect_offenders(&path, forbidden, offenders);
            continue;
        }
        record_if_offending(&path, forbidden, offenders);
    }
}

fn record_if_offending(path: &Path, forbidden: &[String], offenders: &mut Vec<String>) {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return;
    };
    if !matches!(extension, "md" | "kdl") {
        return;
    }

    let contents = fs::read_to_string(path).expect("file should be readable");
    if forbidden.iter().any(|needle| contents.contains(needle)) {
        offenders.push(path.display().to_string());
    }
}
