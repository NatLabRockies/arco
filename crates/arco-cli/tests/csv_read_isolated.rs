use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn isolated_csv_reading_performance() -> Result<(), Box<dyn std::error::Error>> {
    let test_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("e2e");

    // Find all CSV files in test data
    let csv_files = vec![
        "price-taker-battery/data/prices.csv",
        "pcm/data/demand.csv",
        "pcm/data/fuel_cost.csv",
        "pcm/data/lol_penalty.csv",
        "pcm/data/initial_commitment.csv",
        "pcm/data/thermal_units.csv",
        "sdom/data/demand.csv",
        "sdom/data/solar_cf.csv",
        "sdom/data/wind_cf.csv",
        "sdom/data/export_price.csv",
        "sdom/data/import_price.csv",
        "sdom/data/wind_plants.csv",
        "sdom/data/pv_plants.csv",
        "sdom/data/hydro_min.csv",
        "sdom/data/hydro_max.csv",
        "sdom/data/budget_periods.csv",
        "capacity-expansion/data/demand.csv",
        "capacity-expansion/data/voll.csv",
        "capacity-expansion/data/candidate_gas.csv",
        "capacity-expansion/data/existing_thermal.csv",
    ];

    let start = Instant::now();
    let mut total_rows = 0;

    for csv_path in csv_files {
        let path = test_dir.join(csv_path);
        if path.exists() {
            let mut reader = csv::Reader::from_path(&path)?;
            let headers = reader.headers()?.clone();

            for record in reader.records() {
                let record = record?;
                let mut _row = HashMap::with_capacity(headers.len());
                for i in 0..headers.len() {
                    if let Some(value) = record.get(i) {
                        _row.insert(headers[i].to_string(), value.to_string());
                    }
                }
                total_rows += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    let total_us = elapsed.as_micros();
    println!("METRIC csv_isolated_us={}", total_us);
    println!("Read {} total rows in {} us", total_rows, total_us);

    assert!(total_rows > 0, "Should have read some CSV rows");
    Ok(())
}
