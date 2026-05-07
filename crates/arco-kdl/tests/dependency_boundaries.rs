use std::path::PathBuf;

#[test]
fn arco_kdl_stays_authoring_only_among_arco_crates() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).expect("read Cargo.toml");
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

    assert_eq!(arco_deps, vec!["arco-model"]);
}
