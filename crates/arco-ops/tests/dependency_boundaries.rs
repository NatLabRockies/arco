use std::path::PathBuf;

#[test]
fn arco_ops_does_not_declare_builtin_solver_adapters_directly() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).expect("read Cargo.toml");
    let cargo: toml::Value = toml::from_str(&manifest).expect("parse Cargo.toml");

    let dependencies = cargo["dependencies"]
        .as_table()
        .expect("dependencies table");

    assert!(
        !dependencies.contains_key("arco-highs"),
        "arco-ops should not depend on concrete HiGHS adapter crates"
    );
    assert!(
        !dependencies.contains_key("arco-scip"),
        "arco-ops should not depend on concrete SCIP adapter crates"
    );
    assert!(
        !dependencies.contains_key("arco-xpress"),
        "arco-ops should not depend on concrete Xpress adapter crates"
    );
    assert!(
        !dependencies.contains_key("arco-solver-builtins"),
        "arco-ops should not depend on arco-solver-builtins"
    );
}
