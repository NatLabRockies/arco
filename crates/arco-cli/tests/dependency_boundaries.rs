use std::path::PathBuf;

fn arco_dependencies_for(manifest_path: &PathBuf) -> Vec<String> {
    let manifest = std::fs::read_to_string(manifest_path).expect("read Cargo.toml");
    let cargo: toml::Value = toml::from_str(&manifest).expect("parse Cargo.toml");

    let dependencies = cargo["dependencies"]
        .as_table()
        .expect("dependencies table");

    let mut arco_deps = dependencies
        .keys()
        .filter(|name| name.starts_with("arco-"))
        .cloned()
        .collect::<Vec<_>>();
    arco_deps.sort();
    arco_deps
}

#[test]
fn arco_cli_depends_on_arco_ops_only_among_arco_crates() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let arco_deps = arco_dependencies_for(&manifest_path);

    assert_eq!(arco_deps, vec!["arco-ops"]);
}

#[test]
fn python_bindings_depend_on_ops_and_blocks_only_among_arco_crates() {
    let manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bindings/python/Cargo.toml");
    let arco_deps = arco_dependencies_for(&manifest_path);

    assert_eq!(arco_deps, vec!["arco-blocks", "arco-ops"]);
}
