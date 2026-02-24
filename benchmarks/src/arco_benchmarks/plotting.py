from __future__ import annotations

from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns


def plot_all(*, csv_path: Path, output_dir: Path) -> list[Path]:
    df = pd.read_csv(csv_path)
    required = {
        "tool",
        "phase",
        "num_variables",
        "peak_memory_gb",
        "wall_time_seconds",
    }
    missing = required.difference(df.columns)
    if missing:
        raise ValueError(f"Missing required columns: {sorted(missing)}")

    df = df[df["phase"] == "build"].copy()
    if df.empty:
        raise ValueError("No build-phase rows available for plotting")

    output_dir.mkdir(parents=True, exist_ok=True)
    sns.set_theme(style="whitegrid")

    mem_path = output_dir / "memory_vs_variables_build_logx.png"
    fig, ax = plt.subplots(figsize=(8, 5))
    sns.lineplot(
        data=df,
        x="num_variables",
        y="peak_memory_gb",
        hue="tool",
        markers=True,
        dashes=False,
        ax=ax,
    )
    ax.set_xscale("log")
    ax.set_xlabel("Number of variables")
    ax.set_ylabel("Peak memory [GB]")
    ax.set_title("Model build peak memory")
    fig.tight_layout()
    fig.savefig(mem_path, dpi=150)
    plt.close(fig)

    time_path = output_dir / "time_vs_variables_build_logx.png"
    fig, ax = plt.subplots(figsize=(8, 5))
    sns.lineplot(
        data=df,
        x="num_variables",
        y="wall_time_seconds",
        hue="tool",
        markers=True,
        dashes=False,
        ax=ax,
    )
    ax.set_xscale("log")
    ax.set_xlabel("Number of variables")
    ax.set_ylabel("Build time [s]")
    ax.set_title("Model build wall time")
    fig.tight_layout()
    fig.savefig(time_path, dpi=150)
    plt.close(fig)

    return [mem_path, time_path]
