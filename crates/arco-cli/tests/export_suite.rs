use arco_cli::export::{write_lp, write_mps};
use arco_kdl::pipeline::compile_file;
use std::path::PathBuf;

#[test]
fn exports_price_taker_battery_fixture_to_lp() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("price-taker-battery")
        .join("input.kdl");

    let compiled = compile_file(&path)?;
    let mut output = Vec::new();
    write_lp(&compiled.lowered_problem.algebra, &mut output)?;

    let text = String::from_utf8(output)?;
    assert!(text.contains("Maximize"));
    assert!(text.contains("ArbitrageProfit:"));
    assert!(text.contains("Subject To"));
    assert!(text.contains("soc_balance[Battery1,1]:"));
    assert!(text.contains("Bounds"));
    assert!(text.contains("End"));

    // Verify the charge efficiency coefficient (1/0.92 ≈ 1.0869...) appears in soc balance rows.
    assert!(text.contains("1.086956521739 discharge[Battery1,1]"));
    // Verify the -0.92 charge efficiency coefficient.
    assert!(text.contains("- 0.92 charge[Battery1,1]"));
    // Verify the initial SOC boundary (initial_soc_mwh = 200) is the RHS of the first balance.
    assert!(text.contains("soc_balance[Battery1,1]: - 0.92 charge[Battery1,1] + 1.086956521739 discharge[Battery1,1] + soc[Battery1,1] = 200"));
    // Verify the power capacity bound (power_mw = 100).
    assert!(text.contains("charge_limit[Battery1,1]: charge[Battery1,1] <= 100"));
    // Verify the energy capacity bound (energy_mwh = 400).
    assert!(text.contains("soc_bounds[Battery1,1]_upper: soc[Battery1,1] <= 400"));
    // Verify the terminal SOC constraint (terminal_soc_mwh = 200 over 24 steps).
    assert!(text.contains("terminal_soc[Battery1]: soc[Battery1,24] = 200"));

    Ok(())
}

#[test]
fn exports_price_taker_battery_fixture_to_mps() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e")
        .join("price-taker-battery")
        .join("input.kdl");

    let compiled = compile_file(&path)?;
    let mut output = Vec::new();
    write_mps(&compiled.lowered_problem.algebra, &mut output)?;

    let text = String::from_utf8(output)?;
    assert!(text.contains("NAME          MODEL"));
    assert!(text.contains("ROWS"));
    assert!(text.contains("COLUMNS"));
    assert!(text.contains("RHS"));
    assert!(text.contains("BOUNDS"));
    assert!(text.contains("ENDATA"));

    // Verify constraint types: equality for soc_balance, inequality for charge_limit.
    assert!(text.contains(" E  soc_balance[Battery1,1]"));
    assert!(text.contains(" L  charge_limit[Battery1,1]"));
    // Verify the charge efficiency coefficient in the COLUMNS section.
    assert!(text.contains("soc_balance[Battery1,1]             -0.92"));
    // Verify the power capacity RHS (power_mw = 100).
    assert!(text.contains("RHS1      charge_limit[Battery1,1]               100"));
    // Verify the initial SOC as the soc_balance RHS (initial_soc_mwh = 200).
    assert!(text.contains("RHS1      soc_balance[Battery1,1]               200"));
    // Verify the energy capacity RHS (energy_mwh = 400).
    assert!(text.contains("RHS1      soc_bounds[Battery1,1]_upper               400"));
    // Verify soc variables are declared free (no lower bound).
    assert!(text.contains(" FR BND1      soc[Battery1,1]"));

    Ok(())
}
