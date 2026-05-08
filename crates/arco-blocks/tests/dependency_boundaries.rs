use std::path::PathBuf;

#[test]
fn arco_blocks_is_language_neutral_core() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).expect("read Cargo.toml");
    let cargo: toml::Value = toml::from_str(&manifest).expect("parse Cargo.toml");

    let dependencies = cargo["dependencies"]
        .as_table()
        .expect("dependencies table");

    assert!(
        !dependencies.contains_key("pyo3"),
        "arco-blocks core must stay language-neutral; PyO3 belongs in bindings/python"
    );
    assert!(
        !dependencies.contains_key("arco-highs")
            && !dependencies.contains_key("arco-ipopt")
            && !dependencies.contains_key("arco-scip")
            && !dependencies.contains_key("arco-xpress"),
        "arco-blocks must not depend on concrete solver adapter crates"
    );
}
