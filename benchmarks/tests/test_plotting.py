from __future__ import annotations

import pandas as pd

from arco_benchmarks.plotting import plot_all


def test_plot_all_emits_per_phase_outputs(tmp_path) -> None:
    csv_path = tmp_path / "points.csv"
    pd.DataFrame(
        [
            {
                "tool": "arco",
                "phase": "build",
                "n": 10,
                "num_variables": 200,
                "wall_time_seconds": 0.01,
                "peak_memory_gb": 0.001,
                "memory_source": "raw",
            },
            {
                "tool": "arco",
                "phase": "materialize",
                "n": 10,
                "num_variables": 200,
                "wall_time_seconds": 0.02,
                "peak_memory_gb": 0.002,
                "memory_source": "raw",
            },
            {
                "tool": "arco",
                "phase": "solve",
                "n": 10,
                "num_variables": 200,
                "wall_time_seconds": 0.03,
                "peak_memory_gb": 0.003,
                "memory_source": "raw",
            },
        ]
    ).to_csv(csv_path, index=False)

    outputs = plot_all(csv_path=csv_path, output_dir=tmp_path / "plots")

    names = sorted(path.name for path in outputs)
    assert names == [
        "memory_vs_variables_build.png",
        "memory_vs_variables_materialize.png",
        "memory_vs_variables_solve.png",
        "time_vs_variables_build.png",
        "time_vs_variables_materialize.png",
        "time_vs_variables_solve.png",
    ]
