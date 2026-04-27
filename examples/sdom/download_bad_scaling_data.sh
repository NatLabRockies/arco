#!/usr/bin/env bash
set -euo pipefail

# Download SDOM bad-scaling CSV inputs into examples/bad-scaling/data.
# Override ARCO_BAD_SCALING_BASE_URL to use a different source.

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/../.." && pwd)"
target_dir="${repo_root}/examples/bad-scaling/data"
base_url="${ARCO_BAD_SCALING_BASE_URL:-https://raw.githubusercontent.com/NatLabRockies/arco/main/examples/bad-scaling/data}"

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

mkdir -p "${target_dir}"

echo "Downloading bad-scaling data into ${target_dir}"
for file in "${files[@]}"; do
  url="${base_url}/${file}"
  out="${target_dir}/${file}"
  curl -fL --retry 3 --retry-delay 1 -o "${out}" "${url}"
done

echo "Done."
