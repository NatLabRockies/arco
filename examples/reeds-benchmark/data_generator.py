"""
Parametric test problem generator for ReEDS framework comparison.

Models a simplified single-vintage capacity expansion LP with the same
indexing structure and constraint patterns as c_model.gms:

  Variables:   GEN[i,r,h,t], CAP[i,r,t], INV[i,r,t], FLOW[r,rr,h,t],
               RAMPUP[i,r,h,t], CHARGE[i,r,h,t], SOC[i,r,h,t]
  Constraints: supply_demand, cap_limit, cap_accum, reserve_margin,
               trans_limit, mingen, emit_cap, ramping, soc_balance,
               soc_cap, charge_cap, min_cf
  Objective:   min sum_t pvf[t] * (inv_cost + op_cost + startcost)

valcap[i,r,t] sparsity restricts which tech/region/year combinations are
active, matching GAMS $valcap dollar-conditionals.

VRE capacity factors vary by region (wind phase/amplitude, solar peak CF).

Transmission uses a sparse mesh (ring + random extra edges) rather than a
pure ring, giving each region ~4 corridors instead of 2.

n_years=2 for all sizes: ReEDS solves each solve year independently
(sequential myopic), so 2 years captures the intertemporal cap_accum
structure without the over-coupling of 10-15 year horizons.

Four problem sizes:
  small  —   5 R,  4 I,  24 H,  2 T
  medium —  20 R,  5 I, 100 H,  2 T
  large  —  60 R,  7 I, 400 H,  2 T
  xlarge — 120 R,  8 I, 800 H,  2 T
"""

from __future__ import annotations
from dataclasses import dataclass, field
import numpy as np


SIZES = {
    "small": dict(n_regions=5, n_techs=4, n_hours=24, n_years=2),
    "medium": dict(n_regions=20, n_techs=5, n_hours=100, n_years=2),
    "large": dict(n_regions=60, n_techs=7, n_hours=400, n_years=2),
    "xlarge": dict(n_regions=120, n_techs=8, n_hours=800, n_years=2),
}

# name, cost_inv($/MW), cost_op($/MWh), avail, is_vre, emit_rate(tCO2/MWh),
#       startcost($/MW-ramp), is_storage
TECH_PARAMS = [
    ("gas_cc", 800_000, 30.0, 0.87, False, 0.40, 40.0, False),
    ("gas_ct", 400_000, 65.0, 0.90, False, 0.55, 30.0, False),
    ("wind", 1_500_000, 4.0, None, True, 0.00, 0.0, False),
    ("solar", 1_000_000, 3.0, None, True, 0.00, 0.0, False),
    ("coal", 3_000_000, 18.0, 0.85, False, 0.82, 100.0, False),
    ("nuclear", 5_000_000, 10.0, 0.92, False, 0.01, 200.0, False),
    ("battery", 600_000, 12.0, 1.00, False, 0.00, 0.0, True),
    ("geotherm", 2_000_000, 20.0, 0.85, False, 0.04, 50.0, False),
]

# Minimum dispatch fraction per hour (0 = unconstrained)
MINLOADFRAC = {
    "gas_cc": 0.00,
    "gas_ct": 0.00,
    "coal": 0.00,
    "nuclear": 0.70,
    "battery": 0.00,
    "geotherm": 0.00,
}

# Minimum annual capacity factor for non-VRE, non-storage techs
MIN_CF = 0.06


@dataclass
class ProblemData:
    """All sets and parameters for one problem instance."""

    # ---- sets -------------------------------------------------------
    regions: list[str]
    techs: list[str]
    hours: list[int]
    years: list[int]
    routes: list[tuple]

    # ---- scalar parameters ------------------------------------------
    prm: float
    tranloss: float
    charge_eff: float  # one-way charging efficiency for storage
    duration_h: float  # energy capacity = duration_h * power capacity

    # ---- indexed parameters (numpy, indexed by position) ------------
    load: np.ndarray  # [r, h, t]  demand [MW]
    cf: np.ndarray  # [i, r, h]  capacity factor (region-specific for VRE)
    cost_inv: np.ndarray  # [i]        $/MW annualised investment
    cost_op: np.ndarray  # [i]        $/MWh variable O&M + fuel
    startcost: np.ndarray  # [i]        $/MW-ramp start cost (0 for VRE/storage)
    cap_init: np.ndarray  # [i, r]     existing capacity [MW]
    transcap: dict  # {(r,rr): MW}
    pvf: np.ndarray  # [t]        present value factor
    hours_weight: np.ndarray  # [h]        hours per timeslice

    # ---- sparsity & constraint parameters ---------------------------
    valcap: np.ndarray  # [i, r, t]  bool: tech/region/year active
    is_vre: np.ndarray  # [i]        bool: VRE tech (no mingen, no emit)
    is_storage: np.ndarray  # [i]        bool: storage (CHARGE+SOC constraints)
    minloadfrac: np.ndarray  # [i]        min dispatch fraction per hour (0=none)
    min_cf: np.ndarray  # [i]        min annual CF (0 for VRE/storage)
    emit_rate: np.ndarray  # [i]        tonne CO2/MWh (0 for VRE)
    emit_cap: np.ndarray  # [t]        annual CO2 cap (tonne)

    # ---- index maps -------------------------------------------------
    r_idx: dict = field(repr=False)
    i_idx: dict = field(repr=False)
    h_idx: dict = field(repr=False)
    t_idx: dict = field(repr=False)

    # -----------------------------------------------------------------
    @property
    def n_active_irt(self) -> int:
        return int(self.valcap.sum())

    @property
    def n_vars(self) -> int:
        nH, nT = len(self.hours), len(self.years)
        n_routes = len(self.routes)
        irt = self.n_active_irt
        is_disp = ~self.is_vre & ~self.is_storage
        n_disp_irt = int(self.valcap[is_disp].sum())
        n_storage_irt = int(self.valcap[self.is_storage].sum())
        return (
            irt * nH  # GEN
            + irt  # CAP
            + irt  # INV
            + n_routes * nH * nT  # FLOW
            + n_disp_irt * (nH - 1)  # RAMPUP (indexed by starting hour)
            + n_storage_irt * nH  # CHARGE
            + n_storage_irt * nH  # SOC
        )

    @property
    def n_constraints(self) -> int:
        nR, nH, nT = len(self.regions), len(self.hours), len(self.years)
        n_routes = len(self.routes)
        irt = self.n_active_irt
        is_disp = ~self.is_vre & ~self.is_storage
        n_disp_irt = int(self.valcap[is_disp].sum())
        n_storage_irt = int(self.valcap[self.is_storage].sum())
        n_mingen_irt = int((self.valcap & (self.minloadfrac[:, None, None] > 0)).sum())
        return (
            nR * nH * nT  # supply_demand
            + irt * nH  # cap_limit
            + irt  # cap_accum
            + nR * nT  # reserve_margin
            + n_routes * nH * nT  # trans_limit
            + n_mingen_irt * nH  # mingen
            + nT  # emit_cap
            + n_disp_irt * (nH - 1)  # ramping
            + n_storage_irt * nH  # soc_balance
            + n_storage_irt * nH  # soc_cap
            + n_storage_irt * nH  # charge_cap
            + n_disp_irt  # min_cf
        )

    def summary(self) -> str:
        return (
            f"|R|={len(self.regions)}  |I|={len(self.techs)}  "
            f"|H|={len(self.hours)}  |T|={len(self.years)}  "
            f"|routes|={len(self.routes)}  "
            f"valcap={self.n_active_irt:,}/{len(self.techs) * len(self.regions) * len(self.years):,}  "
            f"vars~{self.n_vars:,}  cons~{self.n_constraints:,}"
        )


# -----------------------------------------------------------------
# VRE capacity factor profiles (region-specific)
# -----------------------------------------------------------------


def _wind_cf_regional(
    n_regions: int, n_hours: int, rng: np.random.Generator
) -> np.ndarray:
    """Region-specific wind CF: random phase and amplitude per region."""
    t = np.linspace(0, 2 * np.pi, n_hours, endpoint=False)
    cf = np.zeros((n_regions, n_hours))
    for r in range(n_regions):
        phase = rng.uniform(0, 2 * np.pi)
        base = rng.uniform(0.20, 0.30)
        amp = rng.uniform(0.10, 0.20)
        noise = rng.uniform(-0.04, 0.04, n_hours)
        cf[r] = np.clip(base + amp * np.sin(t + phase) + noise, 0.0, 1.0)
    return cf


def _solar_cf_regional(
    n_regions: int, n_hours: int, rng: np.random.Generator
) -> np.ndarray:
    """Region-specific solar CF: random peak CF and longitude phase shift."""
    t = np.linspace(0, 2 * np.pi, n_hours, endpoint=False)
    cf = np.zeros((n_regions, n_hours))
    for r in range(n_regions):
        peak = rng.uniform(0.70, 0.95)
        phase = rng.uniform(-0.2, 0.2)
        raw = np.maximum(0.0, np.cos(t - np.pi + phase))
        mx = raw.max()
        scaled = peak * raw / mx if mx > 0 else raw
        noise = rng.uniform(-0.02, 0.02, n_hours)
        cf[r] = np.clip(scaled + noise, 0.0, 1.0)
    return cf


# -----------------------------------------------------------------
# Load profile
# -----------------------------------------------------------------


def _load_profile(
    n_regions: int, n_hours: int, n_years: int, peak_mw: float, rng: np.random.Generator
) -> np.ndarray:
    t = np.linspace(0, 2 * np.pi, n_hours, endpoint=False)
    daily = 0.6 + 0.4 * np.sin(t - np.pi / 2)
    load = np.zeros((n_regions, n_hours, n_years))
    for r in range(n_regions):
        region_peak = peak_mw * rng.uniform(0.7, 1.3)
        for yt in range(n_years):
            growth = 1.0 + 0.01 * yt
            noise = rng.uniform(0.95, 1.05, n_hours)
            load[r, :, yt] = region_peak * growth * daily * noise
    return load


# -----------------------------------------------------------------
# Sparse mesh transmission topology
# -----------------------------------------------------------------


def _sparse_mesh_routes(
    regions: list[str],
    capacity_mw: float,
    rng: np.random.Generator,
    extra_neighbors: int = 2,
) -> tuple[list, dict]:
    """
    Build a sparse mesh network: ring backbone for connectivity, plus
    extra_neighbors random additional edges per node.  Each region ends
    up with roughly 2 + extra_neighbors transmission corridors.
    """
    n = len(regions)
    edges: set[tuple[int, int]] = set()
    # Ring backbone guarantees graph connectivity
    for i in range(n):
        j = (i + 1) % n
        edges.add((min(i, j), max(i, j)))
    # Random extra edges
    for i in range(n):
        added, attempts = 0, 0
        while added < extra_neighbors and attempts < 10 * extra_neighbors:
            j = int(rng.integers(0, n))
            attempts += 1
            key = (min(i, j), max(i, j))
            if j != i and key not in edges:
                edges.add(key)
                added += 1
    routes, transcap = [], {}
    for i, j in sorted(edges):
        r, rr = regions[i], regions[j]
        routes.extend([(r, rr), (rr, r)])
        transcap[(r, rr)] = capacity_mw * float(rng.uniform(0.8, 1.2))
        transcap[(rr, r)] = capacity_mw * float(rng.uniform(0.8, 1.2))
    return routes, transcap


# -----------------------------------------------------------------
# valcap generation
# -----------------------------------------------------------------


def _make_valcap(
    n_techs: int,
    n_regions: int,
    n_years: int,
    is_vre: np.ndarray,
    is_storage: np.ndarray,
    minloadfrac: np.ndarray,
    rng: np.random.Generator,
) -> np.ndarray:
    """
    Build a [n_techs, n_regions, n_years] bool mask.

    Each tech enters in a random early year and is available in a
    random subset of regions (60-100%), mimicking GAMS $valcap.

    Feasibility guarantees:
    - At least 1 dispatchable (non-VRE, non-storage) tech per (r,t).
    - At least 1 VRE tech per (r,t) so the emission cap is achievable.
    """
    valcap = np.zeros((n_techs, n_regions, n_years), dtype=bool)
    entry_year_idx = rng.integers(0, max(1, n_years // 3), size=n_techs)
    region_frac = rng.uniform(0.6, 1.0, size=n_techs)

    for i in range(n_techs):
        ey = entry_year_idx[i]
        n_avail = max(1, int(np.ceil(region_frac[i] * n_regions)))
        avail_r = rng.choice(n_regions, size=n_avail, replace=False)
        for r in avail_r:
            for t in range(n_years):
                if t >= ey:
                    valcap[i, r, t] = True

    # Dispatchable = non-VRE, non-storage
    disp_idx = np.where(~is_vre & ~is_storage)[0]
    vre_idx = np.where(is_vre)[0]
    disp_by_mingen = sorted(disp_idx, key=lambda i: minloadfrac[i])

    for r in range(n_regions):
        for t in range(n_years):
            if len(disp_by_mingen) > 0:
                valcap[disp_by_mingen[0], r, t] = True
            if not valcap[vre_idx, r, t].any() and len(vre_idx) > 0:
                forced = rng.choice(vre_idx)
                valcap[forced, r, t] = True

    return valcap


# -----------------------------------------------------------------
# Main factory
# -----------------------------------------------------------------


def make_problem(size: str = "small", seed: int = 42) -> ProblemData:
    """
    Build a ProblemData instance for the given size key.

    Parameters
    ----------
    size : "small" | "medium" | "large" | "xlarge"
    seed : random seed for reproducibility
    """
    if size not in SIZES:
        raise ValueError(f"size must be one of {list(SIZES)}")
    cfg = SIZES[size]
    rng = np.random.default_rng(seed)

    n_regions = cfg["n_regions"]
    n_hours = cfg["n_hours"]
    n_years = cfg["n_years"]
    n_techs = cfg["n_techs"]

    # ---- sets -------------------------------------------------------
    regions = [f"r{i:02d}" for i in range(n_regions)]
    techs = [TECH_PARAMS[i][0] for i in range(n_techs)]
    hours = list(range(n_hours))
    years = [2030 + 5 * y for y in range(n_years)]

    # ---- tech classification ----------------------------------------
    is_vre = np.array([TECH_PARAMS[i][4] for i in range(n_techs)], dtype=bool)
    is_storage = np.array([TECH_PARAMS[i][7] for i in range(n_techs)], dtype=bool)
    emit_rate = np.array([TECH_PARAMS[i][5] for i in range(n_techs)], dtype=float)
    startcost = np.array([TECH_PARAMS[i][6] for i in range(n_techs)], dtype=float)
    minloadfrac = np.array(
        [MINLOADFRAC.get(TECH_PARAMS[i][0], 0.0) for i in range(n_techs)], dtype=float
    )

    # min annual CF: 6% for non-VRE, non-storage techs
    min_cf_arr = np.where(~is_vre & ~is_storage, MIN_CF, 0.0)

    # ---- capacity factor [i, r, h] — region-specific VRE -----------
    cf = np.zeros((n_techs, n_regions, n_hours))
    for idx, (name, _, _, avail, vre, _, _, _) in enumerate(TECH_PARAMS[:n_techs]):
        if vre and name == "wind":
            cf[idx] = _wind_cf_regional(n_regions, n_hours, rng)
        elif vre and name == "solar":
            cf[idx] = _solar_cf_regional(n_regions, n_hours, rng)
        else:
            cf[idx, :, :] = avail if avail is not None else 1.0

    # ---- cost vectors -----------------------------------------------
    cost_inv = np.array([TECH_PARAMS[i][1] for i in range(n_techs)], float)
    cost_op = np.array([TECH_PARAMS[i][2] for i in range(n_techs)], float)

    # ---- load [r, h, t] --------------------------------------------
    peak_mw = {"small": 300.0, "medium": 1_000.0, "large": 5_000.0, "xlarge": 10_000.0}[
        size
    ]
    load = _load_profile(n_regions, n_hours, n_years, peak_mw, rng)

    avg_peak = load[:, :, 0].max(axis=1)  # [r] peak in year 0

    # ---- transmission -----------------------------------------------
    trans_cap_mw = 0.20 * float(np.mean(avg_peak))
    routes, transcap = _sparse_mesh_routes(
        regions, trans_cap_mw, rng, extra_neighbors=2
    )

    # ---- time parameters --------------------------------------------
    hours_weight = np.full(n_hours, 8760.0 / n_hours)
    pvf = np.array([1.0 / (1.07 ** (y - 2030)) for y in years])

    # ---- valcap mask [i, r, t] -------------------------------------
    valcap = _make_valcap(
        n_techs, n_regions, n_years, is_vre, is_storage, minloadfrac, rng
    )

    # ---- initial capacity: dispatchable (non-storage) only ----------
    cap_init = np.zeros((n_techs, n_regions))
    disp_techs_idx = [i for i in range(n_techs) if not is_vre[i] and not is_storage[i]]
    for r in range(n_regions):
        total = 0.55 * avg_peak[r]
        active_disp = sorted(
            [i for i in disp_techs_idx if valcap[i, r, 0]],
            key=lambda i: minloadfrac[i],
        )
        if len(active_disp) >= 1:
            cap_init[active_disp[0], r] = 0.6 * total
        if len(active_disp) >= 2:
            cap_init[active_disp[1], r] = 0.4 * total
        elif len(active_disp) == 1:
            cap_init[active_disp[0], r] = total

    # ---- emission cap [t] ------------------------------------------
    total_annual_load = np.einsum("rht,h->t", load, hours_weight)
    disp_emit = emit_rate[~is_vre & ~is_storage]
    mean_emit = float(disp_emit.mean()) if len(disp_emit) > 0 else 0.5
    emit_cap = total_annual_load * mean_emit * 0.70

    # ---- index maps -------------------------------------------------
    r_idx = {r: i for i, r in enumerate(regions)}
    i_idx = {t: i for i, t in enumerate(techs)}
    h_idx = {h: i for i, h in enumerate(hours)}
    t_idx = {t: i for i, t in enumerate(years)}

    return ProblemData(
        regions=regions,
        techs=techs,
        hours=hours,
        years=years,
        routes=routes,
        prm=0.15,
        tranloss=0.02,
        charge_eff=0.92,
        duration_h=4.0,
        load=load,
        cf=cf,
        cost_inv=cost_inv,
        cost_op=cost_op,
        startcost=startcost,
        cap_init=cap_init,
        transcap=transcap,
        pvf=pvf,
        hours_weight=hours_weight,
        valcap=valcap,
        is_vre=is_vre,
        is_storage=is_storage,
        minloadfrac=minloadfrac,
        min_cf=min_cf_arr,
        emit_rate=emit_rate,
        emit_cap=emit_cap,
        r_idx=r_idx,
        i_idx=i_idx,
        h_idx=h_idx,
        t_idx=t_idx,
    )


if __name__ == "__main__":
    for size in ("small", "medium", "large", "xlarge"):
        p = make_problem(size)
        print(f"{size:6s}: {p.summary()}")
