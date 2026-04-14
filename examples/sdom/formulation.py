# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "arco",
# ]
#
# [tool.uv.sources]
# arco = { path = "../../bindings/python" }
# ///

from __future__ import annotations

import argparse
import csv
import json
import math
from dataclasses import dataclass
from pathlib import Path

import arco


@dataclass(frozen=True)
class SdomData:
    times: list[int]
    demand: dict[int, float]
    nuclear_mw: dict[int, float]
    hydro_mw: dict[int, float]
    other_renewables_mw: dict[int, float]
    wind_plants: list[str]
    wind_max_capacity: dict[str, float]
    wind_capex_m: dict[str, float]
    wind_fom_m: dict[str, float]
    wind_trans_cost: dict[str, float]
    solar_plants: list[str]
    solar_max_capacity: dict[str, float]
    solar_capex_m: dict[str, float]
    solar_fom_m: dict[str, float]
    solar_trans_cost: dict[str, float]
    wind_cf: dict[tuple[int, str], float]
    solar_cf: dict[tuple[int, str], float]
    storage_techs: list[str]
    p_capex: dict[str, float]
    e_capex: dict[str, float]
    efficiency: dict[str, float]
    min_duration: dict[str, float]
    max_duration: dict[str, float]
    max_p: dict[str, float]
    max_cycles: dict[str, float]
    coupled: dict[str, float]
    storage_fom: dict[str, float]
    storage_vom: dict[str, float]
    storage_lifetime: dict[str, float]
    cost_ratio: dict[str, float]
    storage_crf: dict[str, float]
    thermal_units: list[str]
    min_capacity: dict[str, float]
    max_capacity: dict[str, float]
    thermal_capex: dict[str, float]
    heat_rate: dict[str, float]
    fuel_cost: dict[str, float]
    thermal_vom: dict[str, float]
    thermal_fom: dict[str, float]
    thermal_crf: dict[str, float]
    fcr_vre: float = 0.072649
    mw_to_kw: float = 1000.0



def _read_rows(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))



def _read_time_series(path: Path, *, value_field: str) -> dict[int, float]:
    return {int(row["t"]): float(row[value_field]) for row in _read_rows(path)}



def load_data(*, base_dir: Path | None = None) -> SdomData:
    resolved_base_dir = base_dir or Path(__file__).resolve().parent / "data"

    demand = _read_time_series(resolved_base_dir / "demand.csv", value_field="demand")
    nuclear_mw = _read_time_series(resolved_base_dir / "nuclear.csv", value_field="nuclear_mw")
    hydro_mw = _read_time_series(resolved_base_dir / "hydro.csv", value_field="hydro_mw")
    other_renewables_mw = _read_time_series(
        resolved_base_dir / "other_renewables.csv",
        value_field="other_renewables_mw",
    )
    times = sorted(demand)

    wind_rows = _read_rows(resolved_base_dir / "wind_plants.csv")
    wind_plants = [row["plant_id"] for row in wind_rows]
    wind_max_capacity = {row["plant_id"]: float(row["max_capacity"]) for row in wind_rows}
    wind_capex_m = {row["plant_id"]: float(row["capex_m"]) for row in wind_rows}
    wind_fom_m = {row["plant_id"]: float(row["fom_m"]) for row in wind_rows}
    wind_trans_cost = {row["plant_id"]: float(row["trans_cap_cost"]) for row in wind_rows}

    solar_rows = _read_rows(resolved_base_dir / "solar_plants.csv")
    solar_plants = [row["plant_id"] for row in solar_rows]
    solar_max_capacity = {row["plant_id"]: float(row["max_capacity"]) for row in solar_rows}
    solar_capex_m = {row["plant_id"]: float(row["capex_m"]) for row in solar_rows}
    solar_fom_m = {row["plant_id"]: float(row["fom_m"]) for row in solar_rows}
    solar_trans_cost = {row["plant_id"]: float(row["trans_cap_cost"]) for row in solar_rows}

    wind_cf = {
        (int(row["t"]), row["plant_id"]): float(row["wind_cf"])
        for row in _read_rows(resolved_base_dir / "wind_cf.csv")
    }
    solar_cf = {
        (int(row["t"]), row["plant_id"]): float(row["solar_cf"])
        for row in _read_rows(resolved_base_dir / "solar_cf.csv")
    }

    storage_rows = _read_rows(resolved_base_dir / "storage.csv")
    storage_techs = [row["tech_id"] for row in storage_rows]
    p_capex = {row["tech_id"]: float(row["p_capex"]) for row in storage_rows}
    e_capex = {row["tech_id"]: float(row["e_capex"]) for row in storage_rows}
    efficiency = {row["tech_id"]: float(row["efficiency"]) for row in storage_rows}
    min_duration = {row["tech_id"]: float(row["min_duration"]) for row in storage_rows}
    max_duration = {row["tech_id"]: float(row["max_duration"]) for row in storage_rows}
    max_p = {row["tech_id"]: float(row["max_p"]) for row in storage_rows}
    max_cycles = {row["tech_id"]: float(row["max_cycles"]) for row in storage_rows}
    coupled = {row["tech_id"]: float(row["coupled"]) for row in storage_rows}
    storage_fom = {row["tech_id"]: float(row["fom"]) for row in storage_rows}
    storage_vom = {row["tech_id"]: float(row["vom"]) for row in storage_rows}
    storage_lifetime = {row["tech_id"]: float(row["lifetime"]) for row in storage_rows}
    cost_ratio = {row["tech_id"]: float(row["cost_ratio"]) for row in storage_rows}
    storage_crf = {row["tech_id"]: float(row["crf"]) for row in storage_rows}

    thermal_rows = _read_rows(resolved_base_dir / "thermal.csv")
    thermal_units = [row["plant_id"] for row in thermal_rows]
    min_capacity = {row["plant_id"]: float(row["min_capacity"]) for row in thermal_rows}
    max_capacity = {row["plant_id"]: float(row["max_capacity"]) for row in thermal_rows}
    thermal_capex = {row["plant_id"]: float(row["capex"]) for row in thermal_rows}
    heat_rate = {row["plant_id"]: float(row["heat_rate"]) for row in thermal_rows}
    fuel_cost = {row["plant_id"]: float(row["fuel_cost"]) for row in thermal_rows}
    thermal_vom = {row["plant_id"]: float(row["vom"]) for row in thermal_rows}
    thermal_fom = {row["plant_id"]: float(row["fom"]) for row in thermal_rows}
    thermal_crf = {row["plant_id"]: float(row["crf"]) for row in thermal_rows}

    return SdomData(
        times=times,
        demand=demand,
        nuclear_mw=nuclear_mw,
        hydro_mw=hydro_mw,
        other_renewables_mw=other_renewables_mw,
        wind_plants=wind_plants,
        wind_max_capacity=wind_max_capacity,
        wind_capex_m=wind_capex_m,
        wind_fom_m=wind_fom_m,
        wind_trans_cost=wind_trans_cost,
        solar_plants=solar_plants,
        solar_max_capacity=solar_max_capacity,
        solar_capex_m=solar_capex_m,
        solar_fom_m=solar_fom_m,
        solar_trans_cost=solar_trans_cost,
        wind_cf=wind_cf,
        solar_cf=solar_cf,
        storage_techs=storage_techs,
        p_capex=p_capex,
        e_capex=e_capex,
        efficiency=efficiency,
        min_duration=min_duration,
        max_duration=max_duration,
        max_p=max_p,
        max_cycles=max_cycles,
        coupled=coupled,
        storage_fom=storage_fom,
        storage_vom=storage_vom,
        storage_lifetime=storage_lifetime,
        cost_ratio=cost_ratio,
        storage_crf=storage_crf,
        thermal_units=thermal_units,
        min_capacity=min_capacity,
        max_capacity=max_capacity,
        thermal_capex=thermal_capex,
        heat_rate=heat_rate,
        fuel_cost=fuel_cost,
        thermal_vom=thermal_vom,
        thermal_fom=thermal_fom,
        thermal_crf=thermal_crf,
    )



def build_model(*, data: SdomData) -> arco.Model:
    model = arco.Model()

    time = arco.IndexSet("time", size=len(data.times))
    wind_plant = arco.IndexSet("wind_plant", size=len(data.wind_plants))
    solar_plant = arco.IndexSet("solar_plant", size=len(data.solar_plants))
    storage_tech = arco.IndexSet("storage_tech", size=len(data.storage_techs))
    thermal_unit = arco.IndexSet("thermal_unit", size=len(data.thermal_units))

    wind_cap_frac = model.add_variables(
        wind_plant,
        bounds=arco.Bounds(lower=0.0, upper=1.0),
        name="wind_cap_frac",
    )
    solar_cap_frac = model.add_variables(
        solar_plant,
        bounds=arco.Bounds(lower=0.0, upper=1.0),
        name="solar_cap_frac",
    )

    wind_gen = model.add_variables(time, bounds=arco.NonNegativeFloat, name="wind_gen")
    wind_curtail = model.add_variables(time, bounds=arco.NonNegativeFloat, name="wind_curtail")
    solar_gen = model.add_variables(time, bounds=arco.NonNegativeFloat, name="solar_gen")
    solar_curtail = model.add_variables(time, bounds=arco.NonNegativeFloat, name="solar_curtail")

    pc = model.add_variables(time, storage_tech, bounds=arco.NonNegativeFloat, name="pc")
    pd = model.add_variables(time, storage_tech, bounds=arco.NonNegativeFloat, name="pd")
    soc = model.add_variables(time, storage_tech, bounds=arco.NonNegativeFloat, name="soc")

    pcha = model.add_variables(storage_tech, bounds=arco.NonNegativeFloat, name="pcha")
    pdis = model.add_variables(storage_tech, bounds=arco.NonNegativeFloat, name="pdis")
    ecap = model.add_variables(storage_tech, bounds=arco.NonNegativeFloat, name="ecap")
    storage_binary = model.add_variables(
        time,
        storage_tech,
        bounds=arco.Binary,
        name="storage_binary",
    )

    thermal_cap = model.add_variables(thermal_unit, bounds=arco.NonNegativeFloat, name="thermal_cap")
    thermal_gen = model.add_variables(
        time,
        thermal_unit,
        bounds=arco.NonNegativeFloat,
        name="thermal_gen",
    )
    hydro_gen = model.add_variables(time, bounds=arco.NonNegativeFloat, name="hydro_gen")

    sqrt_efficiency = {tech: math.sqrt(data.efficiency[tech]) for tech in data.storage_techs}
    time_indices = range(len(data.times))
    first_time_index = 0
    last_time_index = len(data.times) - 1

    wind_capex = sum(
        data.fcr_vre
        * ((data.wind_capex_m[plant] * data.mw_to_kw) + data.wind_trans_cost[plant])
        * data.wind_max_capacity[plant]
        * wind_cap_frac[plant_index]
        for plant_index, plant in enumerate(data.wind_plants)
    )
    solar_capex = sum(
        data.fcr_vre
        * ((data.solar_capex_m[plant] * data.mw_to_kw) + data.solar_trans_cost[plant])
        * data.solar_max_capacity[plant]
        * solar_cap_frac[plant_index]
        for plant_index, plant in enumerate(data.solar_plants)
    )
    wind_fom = sum(
        data.wind_fom_m[plant] * data.mw_to_kw * data.wind_max_capacity[plant] * wind_cap_frac[plant_index]
        for plant_index, plant in enumerate(data.wind_plants)
    )
    solar_fom = sum(
        data.solar_fom_m[plant] * data.mw_to_kw * data.solar_max_capacity[plant] * solar_cap_frac[plant_index]
        for plant_index, plant in enumerate(data.solar_plants)
    )
    storage_power_capex = sum(
        data.storage_crf[tech]
        * (
            data.mw_to_kw * data.cost_ratio[tech] * data.p_capex[tech] * pcha[storage_index]
            + data.mw_to_kw * (1.0 - data.cost_ratio[tech]) * data.p_capex[tech] * pdis[storage_index]
        )
        for storage_index, tech in enumerate(data.storage_techs)
    )
    storage_energy_capex = sum(
        data.storage_crf[tech] * data.mw_to_kw * data.e_capex[tech] * ecap[storage_index]
        for storage_index, tech in enumerate(data.storage_techs)
    )
    storage_fixed_om = sum(
        data.mw_to_kw * data.cost_ratio[tech] * data.storage_fom[tech] * pcha[storage_index]
        + data.mw_to_kw * (1.0 - data.cost_ratio[tech]) * data.storage_fom[tech] * pdis[storage_index]
        for storage_index, tech in enumerate(data.storage_techs)
    )
    storage_var_om = sum(
        data.storage_vom[tech] * sum(pd[time_index, storage_index] for time_index in time_indices)
        for storage_index, tech in enumerate(data.storage_techs)
    )
    thermal_total_capex = sum(
        data.thermal_crf[unit] * data.thermal_capex[unit] * data.mw_to_kw * thermal_cap[unit_index]
        for unit_index, unit in enumerate(data.thermal_units)
    )
    thermal_total_fom = sum(
        data.thermal_fom[unit] * data.mw_to_kw * thermal_cap[unit_index]
        for unit_index, unit in enumerate(data.thermal_units)
    )
    thermal_fuel_cost = sum(
        data.fuel_cost[unit] * data.heat_rate[unit] * sum(thermal_gen[time_index, unit_index] for time_index in time_indices)
        for unit_index, unit in enumerate(data.thermal_units)
    )
    thermal_var_om = sum(
        data.thermal_vom[unit] * sum(thermal_gen[time_index, unit_index] for time_index in time_indices)
        for unit_index, unit in enumerate(data.thermal_units)
    )

    total_vre_cost = wind_capex + solar_capex + wind_fom + solar_fom
    total_storage_cost = storage_power_capex + storage_energy_capex + storage_fixed_om + storage_var_om
    total_thermal_cost = thermal_total_capex + thermal_total_fom + thermal_fuel_cost + thermal_var_om

    for time_index, time_value in enumerate(data.times):
        model.add_constraint(
            data.demand[time_value]
            + sum(pc[time_index, storage_index] for storage_index in range(len(data.storage_techs)))
            - sum(pd[time_index, storage_index] for storage_index in range(len(data.storage_techs)))
            - data.nuclear_mw[time_value]
            - hydro_gen[time_index]
            - data.other_renewables_mw[time_value]
            - solar_gen[time_index]
            - wind_gen[time_index]
            - sum(thermal_gen[time_index, unit_index] for unit_index in range(len(data.thermal_units)))
            == 0.0,
            name=f"supply_balance[{time_value}]",
        )
        model.add_constraint(
            wind_gen[time_index] + wind_curtail[time_index]
            == sum(
                data.wind_cf[(time_value, plant)] * data.wind_max_capacity[plant] * wind_cap_frac[plant_index]
                for plant_index, plant in enumerate(data.wind_plants)
            ),
            name=f"wind_balance[{time_value}]",
        )
        model.add_constraint(
            solar_gen[time_index] + solar_curtail[time_index]
            == sum(
                data.solar_cf[(time_value, plant)] * data.solar_max_capacity[plant] * solar_cap_frac[plant_index]
                for plant_index, plant in enumerate(data.solar_plants)
            ),
            name=f"solar_balance[{time_value}]",
        )
        model.add_constraint(
            hydro_gen[time_index] == data.hydro_mw[time_value],
            name=f"hydro_run_of_river[{time_value}]",
        )

        for storage_index, tech in enumerate(data.storage_techs):
            model.add_constraint(
                pc[time_index, storage_index] <= data.max_p[tech] * storage_binary[time_index, storage_index],
                name=f"charge_binary_limit[{time_value},{tech}]",
            )
            model.add_constraint(
                pd[time_index, storage_index]
                <= data.max_p[tech] * (1.0 - storage_binary[time_index, storage_index]),
                name=f"discharge_binary_limit[{time_value},{tech}]",
            )
            model.add_constraint(
                pc[time_index, storage_index] <= pcha[storage_index],
                name=f"max_hourly_charging[{time_value},{tech}]",
            )
            model.add_constraint(
                pd[time_index, storage_index] <= pdis[storage_index],
                name=f"max_hourly_discharging[{time_value},{tech}]",
            )
            model.add_constraint(
                soc[time_index, storage_index] <= ecap[storage_index],
                name=f"max_soc[{time_value},{tech}]",
            )

        for unit_index, unit in enumerate(data.thermal_units):
            model.add_constraint(
                thermal_cap[unit_index] >= thermal_gen[time_index, unit_index],
                name=f"thermal_capacity_limit[{time_value},{unit}]",
            )

    model.add_constraint(
        sum(
            thermal_gen[time_index, unit_index]
            for time_index in time_indices
            for unit_index in range(len(data.thermal_units))
        )
        <= 0.0,
        name="gen_mix_share",
    )

    for storage_index, tech in enumerate(data.storage_techs):
        model.add_constraint(
            soc[first_time_index, storage_index]
            == soc[last_time_index, storage_index]
            + sqrt_efficiency[tech] * pc[first_time_index, storage_index]
            - pd[first_time_index, storage_index] / sqrt_efficiency[tech],
            name=f"soc_balance_initial[{tech}]",
        )
        for previous_time_index, current_time_index in zip(time_indices, list(time_indices)[1:]):
            current_time_value = data.times[current_time_index]
            model.add_constraint(
                soc[current_time_index, storage_index]
                == soc[previous_time_index, storage_index]
                + sqrt_efficiency[tech] * pc[current_time_index, storage_index]
                - pd[current_time_index, storage_index] / sqrt_efficiency[tech],
                name=f"soc_balance[{current_time_value},{tech}]",
            )

        model.add_constraint(
            pcha[storage_index] <= data.max_p[tech],
            name=f"max_charge_capacity[{tech}]",
        )
        model.add_constraint(
            pdis[storage_index] <= data.max_p[tech],
            name=f"max_discharge_capacity[{tech}]",
        )
        if data.coupled[tech] > 0.0:
            model.add_constraint(
                pcha[storage_index] == pdis[storage_index],
                name=f"charge_equals_discharge[{tech}]",
            )
        model.add_constraint(
            ecap[storage_index] >= data.min_duration[tech] * pdis[storage_index] / sqrt_efficiency[tech],
            name=f"min_energy_capacity[{tech}]",
        )
        model.add_constraint(
            ecap[storage_index] <= data.max_duration[tech] * pdis[storage_index] / sqrt_efficiency[tech],
            name=f"max_energy_capacity[{tech}]",
        )
        model.add_constraint(
            sum(pd[time_index, storage_index] for time_index in time_indices)
            <= (data.max_cycles[tech] / data.storage_lifetime[tech]) * ecap[storage_index],
            name=f"max_cycle_year[{tech}]",
        )

    model.minimize(total_vre_cost + total_storage_cost + total_thermal_cost)
    return model



def solve_model(*, model: arco.Model) -> arco.Solution:
    solver = arco.HiGHS(log_to_console=False)
    return model.solve(solver=solver)



def main() -> int:
    parser = argparse.ArgumentParser(description="Build or solve SDOM with Arco Python bindings")
    parser.add_argument("--solve", action="store_true", help="Solve after building")
    parser.add_argument("--json", action="store_true", help="Emit machine-readable output")
    args = parser.parse_args()

    data = load_data()
    model = build_model(data=data)
    payload: dict[str, object] = {
        "example": "sdom",
        "time_steps": len(data.times),
        "wind_plants": len(data.wind_plants),
        "solar_plants": len(data.solar_plants),
        "storage_techs": len(data.storage_techs),
        "thermal_units": len(data.thermal_units),
        "solved": False,
    }

    if args.solve:
        solution = solve_model(model=model)
        payload.update(
            {
                "solved": True,
                "status": str(solution.status),
                "is_optimal": solution.is_optimal(),
                "objective_value": solution.objective_value,
            }
        )

    if args.json:
        print(json.dumps(payload, indent=2))
    elif args.solve:
        print(
            "sdom solved: "
            f"time_steps={payload['time_steps']}, objective={payload['objective_value']}, "
            f"status={payload['status']}"
        )
    else:
        print(
            "sdom built: "
            f"time_steps={payload['time_steps']}, wind_plants={payload['wind_plants']}, "
            f"solar_plants={payload['solar_plants']}"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
