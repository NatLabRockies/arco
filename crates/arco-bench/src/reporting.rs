use crate::comparison::summarize_records;
use crate::types::{BenchRecord, CompareRow, CscMatrix, OutputFormat, SummaryRow};
use std::fs::{File, create_dir_all};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub(crate) fn render_output(
    format: OutputFormat,
    records: &[BenchRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => {
            let rows = summarize_records(records);
            print_summary_table(&rows);
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(records)?);
            Ok(())
        }
        OutputFormat::Ndjson => {
            for record in records {
                println!("{}", serde_json::to_string(record)?);
            }
            Ok(())
        }
    }
}

pub(crate) fn render_compare_output(
    format: OutputFormat,
    rows: &[CompareRow],
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Table => {
            print_compare_table(rows);
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(rows)?);
            Ok(())
        }
        OutputFormat::Ndjson => {
            for row in rows {
                println!("{}", serde_json::to_string(row)?);
            }
            Ok(())
        }
    }
}

pub(crate) fn write_records_jsonl(
    path: &Path,
    records: &[BenchRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    for record in records {
        serde_json::to_writer(&mut writer, record)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}

pub(crate) fn load_records_jsonl(
    path: &Path,
) -> Result<Vec<BenchRecord>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        records.push(serde_json::from_str::<BenchRecord>(&line)?);
    }
    Ok(records)
}

pub(crate) fn write_csc_matrix(
    dir: &Path,
    matrix: &CscMatrix,
    variables: usize,
    constraints: usize,
) -> std::io::Result<()> {
    create_dir_all(dir)?;

    fn write_u64(path: &Path, data: &[u64]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for value in data {
            writer.write_all(&value.to_le_bytes())?;
        }
        Ok(())
    }

    fn write_u32(path: &Path, data: &[u32]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for value in data {
            writer.write_all(&value.to_le_bytes())?;
        }
        Ok(())
    }

    fn write_f64(path: &Path, data: &[f64]) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for value in data {
            writer.write_all(&value.to_le_bytes())?;
        }
        Ok(())
    }

    write_u64(&dir.join("col_ptrs.bin"), &matrix.col_ptrs)?;
    write_u32(&dir.join("row_indices.bin"), &matrix.row_indices)?;
    write_f64(&dir.join("values.bin"), &matrix.values)?;

    let mut meta = BufWriter::new(File::create(dir.join("metadata.txt"))?);
    writeln!(meta, "columns (variables): {}", variables)?;
    writeln!(meta, "rows (constraints): {}", constraints)?;
    writeln!(meta, "nonzeros: {}", matrix.values.len())?;
    Ok(())
}

fn print_summary_table(rows: &[SummaryRow]) {
    println!(
        "{:<12} {:<16} {:<12} {:>7} {:>12} {:>12} {:>14} {:>14}",
        "scenario", "case", "stage", "samples", "mean_ms", "max_ms", "mean_rss_mb", "max_rss_mb"
    );
    for row in rows {
        println!(
            "{:<12} {:<16} {:<12} {:>7} {:>12.3} {:>12.3} {:>14} {:>14}",
            row.scenario,
            row.case_name,
            row.stage,
            row.samples,
            row.mean_duration_ms,
            row.max_duration_ms,
            format_option_mb_f64(row.mean_rss_delta_bytes),
            format_option_mb_u64(row.max_rss_after_bytes),
        );
    }
}

fn print_compare_table(rows: &[CompareRow]) {
    println!(
        "{:<12} {:<16} {:<12} {:>12} {:>12} {:>10} {:>12} {:>12} {:>10}",
        "scenario",
        "case",
        "stage",
        "base_ms",
        "cand_ms",
        "dur_%",
        "base_rss_mb",
        "cand_rss_mb",
        "rss_%"
    );
    for row in rows {
        println!(
            "{:<12} {:<16} {:<12} {:>12.3} {:>12.3} {:>10} {:>12} {:>12} {:>10}",
            row.scenario,
            row.case_name,
            row.stage,
            row.baseline_mean_duration_ms,
            row.candidate_mean_duration_ms,
            format_option_pct(row.duration_change_pct),
            format_option_mb_f64(row.baseline_mean_rss_delta_bytes),
            format_option_mb_f64(row.candidate_mean_rss_delta_bytes),
            format_option_pct(row.rss_change_pct),
        );
    }
}

fn format_option_mb_f64(value: Option<f64>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |bytes| format!("{:.3}", bytes / (1024.0 * 1024.0)),
    )
}

fn format_option_mb_u64(value: Option<u64>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |bytes| format!("{:.3}", bytes as f64 / (1024.0 * 1024.0)),
    )
}

fn format_option_pct(value: Option<f64>) -> String {
    value.map_or_else(|| "-".to_string(), |pct| format!("{:.2}", pct))
}
