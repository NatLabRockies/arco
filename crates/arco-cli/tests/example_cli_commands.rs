use std::path::PathBuf;
use std::process::Command;

fn example_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn assert_cli_success(args: &[&str]) {
    let output = Command::new(env!("CARGO_BIN_EXE_arco"))
        .args(args)
        .output()
        .expect("failed to execute arco binary");

    assert!(
        output.status.success(),
        "command `arco {}` failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn examples_support_validate_print_model_inspect_and_run() {
    let examples = [
        "examples/generator-allocation/input.kdl",
        "examples/price-taker-battery/input.kdl",
        "examples/simple-electricity-market-storage/input.kdl",
        "examples/capacity-expansion/input.kdl",
    ];

    for example in examples {
        let model_path = example_path(example);
        let model = model_path
            .to_str()
            .expect("example path contains invalid unicode");

        assert_cli_success(&["validate", model]);
        assert_cli_success(&["print-model", model]);
        assert_cli_success(&["inspect", model, "--section", "constraints"]);
        assert_cli_success(&["run", model, "--compact"]);
    }
}
