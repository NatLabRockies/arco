#!/usr/bin/env bash
set -euo pipefail

# Download SDOM bad-scaling CSV inputs into examples/bad-scaling/data.
# By default this uses a pinned upstream commit for reproducibility.
# Override ARCO_BAD_SCALING_BASE_URL to use a different source.

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/../.." && pwd)"
target_dir="${repo_root}/examples/bad-scaling/data"
compat_dir="${repo_root}/examples/sdom/data-bad-scaling"
default_base_ref="d610f77e5f72abbb8359e4396b96ae9799e47628"
default_base_url="https://raw.githubusercontent.com/Omar0902/SDOM/${default_base_ref}/Data/exchange_hydro_daily_budget_multiple_balancing_p95"
base_url="${ARCO_BAD_SCALING_BASE_URL:-${default_base_url}}"

files=(
  CapSolar_2025.csv
  CapWind_2025.csv
  CFSolar_2025.csv
  CFWind_2025.csv
  Data_BalancingUnits_2025.csv
  Export_Cap_2025.csv
  export_prices_2025.csv
  formulations.csv
  Import_Cap_2025.csv
  import_prices_2025.csv
  lahy_hourly_2025.csv
  lahy_max_hourly_2025.csv
  lahy_min_hourly_2025.csv
  Load_hourly_2025.csv
  Nucl_hourly_2025.csv
  otre_hourly_2025.csv
  scalars.csv
  set_hydro_monthly_budget.txt
  "Set_k(SolarPV).txt"
  "Set_st(StorageTech).txt"
  "Set_w(Wind).txt"
  StorageData_2025.csv
)

mkdir -p "${target_dir}" "${compat_dir}"

echo "Downloading bad-scaling data into ${target_dir}"
echo "Source: ${base_url}"
for file in "${files[@]}"; do
  url="${base_url}/${file}"
  out="${target_dir}/${file}"
  curl -fL --retry 3 --retry-delay 1 -o "${out}" "${url}"
done

echo "Generating SDOM compatibility tables in ${compat_dir}"
ARCO_REPO_ROOT="${repo_root}" python - <<'PY'
from __future__ import annotations

import csv
import os
from pathlib import Path

repo_root = Path(os.environ["ARCO_REPO_ROOT"])
target_dir = repo_root / "examples" / "bad-scaling" / "data"
compat_dir = repo_root / "examples" / "sdom" / "data-bad-scaling"
compat_dir.mkdir(parents=True, exist_ok=True)


def write_vre_table(*, source_name: str, out_name: str) -> None:
    source_path = target_dir / source_name
    out_path = compat_dir / out_name

    with source_path.open(newline="", encoding="utf-8") as source_file:
        reader = csv.DictReader(source_file)
        rows = list(reader)

    with out_path.open("w", newline="", encoding="utf-8") as out_file:
        writer = csv.DictWriter(
            out_file,
            fieldnames=["plant_id", "max_capacity", "capex_m", "fom_m", "trans_cap_cost"],
            lineterminator="\n",
        )
        writer.writeheader()
        for row in rows:
            writer.writerow(
                {
                    "plant_id": row["sc_gid"],
                    "max_capacity": row["capacity"],
                    "capex_m": row["CAPEX_M"],
                    "fom_m": row["FOM_M"],
                    "trans_cap_cost": row["trans_cap_cost"],
                }
            )


def write_storage_table() -> None:
    source_path = target_dir / "StorageData_2025.csv"
    out_path = compat_dir / "storage.txt"

    with source_path.open(newline="", encoding="utf-8") as source_file:
        reader = csv.DictReader(source_file)
        fieldnames = reader.fieldnames
        if fieldnames is None:
            raise ValueError("StorageData_2025.csv has no header")

        metric_column = fieldnames[0]
        techs = fieldnames[1:]
        metric_rows = {
            row[metric_column]: row
            for row in reader
        }

    required_metrics = [
        "P_Capex",
        "E_Capex",
        "Eff",
        "Min_Duration",
        "Max_Duration",
        "Max_P",
        "MaxCycles",
        "Coupled",
        "FOM",
        "VOM",
        "Lifetime",
        "CostRatio",
    ]
    for metric in required_metrics:
        if metric not in metric_rows:
            raise ValueError(f"StorageData_2025.csv missing metric row: {metric}")

    with out_path.open("w", newline="", encoding="utf-8") as out_file:
        writer = csv.DictWriter(
            out_file,
            fieldnames=[
                "tech_id",
                "p_capex",
                "e_capex",
                "efficiency",
                "min_duration",
                "max_duration",
                "max_p",
                "max_cycles",
                "coupled",
                "fom",
                "vom",
                "lifetime",
                "cost_ratio",
            ],
            lineterminator="\n",
        )
        writer.writeheader()
        for tech in techs:
            writer.writerow(
                {
                    "tech_id": tech,
                    "p_capex": metric_rows["P_Capex"][tech],
                    "e_capex": metric_rows["E_Capex"][tech],
                    "efficiency": metric_rows["Eff"][tech],
                    "min_duration": metric_rows["Min_Duration"][tech],
                    "max_duration": metric_rows["Max_Duration"][tech],
                    "max_p": metric_rows["Max_P"][tech],
                    "max_cycles": metric_rows["MaxCycles"][tech],
                    "coupled": metric_rows["Coupled"][tech],
                    "fom": metric_rows["FOM"][tech],
                    "vom": metric_rows["VOM"][tech],
                    "lifetime": metric_rows["Lifetime"][tech],
                    "cost_ratio": metric_rows["CostRatio"][tech],
                }
            )


write_vre_table(source_name="CapWind_2025.csv", out_name="wind_plants.txt")
write_vre_table(source_name="CapSolar_2025.csv", out_name="solar_plants.txt")
write_storage_table()
PY

echo "Done."
