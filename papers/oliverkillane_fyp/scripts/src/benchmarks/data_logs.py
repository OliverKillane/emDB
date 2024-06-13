#!/usr/bin/env python
import json
from pathlib import Path
from matplotlib.ticker import MaxNLocator
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib


def get_best_unit_and_scale(min_value) -> tuple[str, float]:
    TIME_UNITS = ["ps", "ns", "\mu s", "ms"]
    scale = 1
    for unit in TIME_UNITS:
        if min_value >= 1:
            return unit, scale
        else:
            min_value *= 1000
            scale *= 1000
    return "s", scale


def process_data(data: dict, bench_name: str) -> pd.DataFrame:
    benchmark_list = []
    bench_data = data["benchmarks"][bench_name]
    for query, db_data in bench_data.items():
        for db, results in db_data.items():
            for scale_factor, result in results.items():
                if "time" in result:
                    scale_factor = int(scale_factor)
                    benchmark_list.append(
                        {
                            "query": query,
                            "db": db,
                            "scale_factor": scale_factor,
                            "mean_rate": float(
                                scale_factor / float(result["time"]["mean"])
                            ),  # Convert picoseconds to microseconds
                            "median_rate": float(
                                scale_factor / float(result["time"]["median"])
                            ),
                            "fastest_rate": float(
                                scale_factor / float(result["time"]["fastest"])
                            ),
                            "slowest_rate": float(
                                scale_factor / float(result["time"]["slowest"])
                            ),
                        }
                    )

    return pd.DataFrame(benchmark_list)


COLOUR_MAP = {
    "EmDBIter": "purple",
    "EmDBThunderdome": "green",
    "DuckDB": "yellow",
    "SQLite": "blue",
}


def plot_data(data_unscaled: pd.DataFrame) -> plt:
    data_unscaled = data_unscaled.sort_values(by="scale_factor")

    max_value = data_unscaled[["slowest_rate"]].max().max()
    unit, scale = get_best_unit_and_scale(max_value)

    # Scale the data
    data = data_unscaled.copy()
    data[["mean_rate", "median_rate", "fastest_rate", "slowest_rate"]] = (
        data[["mean_rate", "median_rate", "fastest_rate", "slowest_rate"]] * scale
    )

    # Generate the bar graph
    fig, ax = plt.subplots(figsize=(10, 4))  # Adjusted figure size to be smaller
    # Plot each database's benchmark results
    queries = data["query"].unique()
    scale_factors = data["scale_factor"].unique()
    x = range(len(queries) * len(scale_factors))

    WIDTH = 0.1  # Width of the bars

    def position(query_idx: int, scale_factor_idx: int, db_index: int) -> float:
        SPACING = 0  # Spacing between groups of bars
        SCALE_SPACING = 0.05  # Scale factor between queries
        QUERY_SPACING = 0.3

        scale_factors = len(data["scale_factor"].unique())
        databases = len(data["db"].unique())
        return (query_idx * QUERY_SPACING) + (
            (query_idx * scale_factors + scale_factor_idx) * SCALE_SPACING
            ) + (
                (((query_idx * scale_factors + scale_factor_idx) * databases) +
                db_index) * (WIDTH + SPACING)
                
            )

    bar_positions = []
    for q_idx, query in enumerate(queries):
        for sf_idx, sf in enumerate(scale_factors):
            base_pos = q_idx * len(scale_factors) + sf_idx
            for i, db in enumerate(data["db"].unique()):
                db_data = data[
                    (data["db"] == db)
                    & (data["query"] == query)
                    & (data["scale_factor"] == sf)
                ]
                pos = position(q_idx, sf_idx, i)
                bar_positions.append(pos)
                if not db_data.empty:
                    ax.bar(
                        pos,
                        db_data["mean_rate"].values[0],
                        width=WIDTH,
                        label=db if q_idx == 0 and sf_idx == 0 else "",
                        color=COLOUR_MAP.get(db, "black"),
                    )
                    mean = db_data["mean_rate"].values[0]
                    fastest = db_data["fastest_rate"].values[0]
                    slowest = db_data["slowest_rate"].values[0]
                    median = db_data["median_rate"].values[0]
                    ax.plot(
                        [pos, pos], [fastest, slowest], color="grey"
                    )  # Vertical line
                    ax.plot(
                        [pos - WIDTH / 2, pos + WIDTH / 2],
                        [fastest, fastest],
                        color="grey",
                    )  # Fastest time as a horizontal line
                    ax.plot(
                        [pos - WIDTH / 2, pos + WIDTH / 2],
                        [slowest, slowest],
                        color="grey",
                    )  # Slowest time as a horizontal line
                    ax.plot(
                        [pos - WIDTH / 2, pos + WIDTH / 2],
                        [median, median],
                        color="orange",
                    )  # Median as dotted line
                    ax.plot(
                        [pos - WIDTH / 2, pos + WIDTH / 2], [mean, mean], color="black"
                    )  # Mean as black line

    ax.set_xticks(
        [
            np.mean(
                [position(query_idx, scale_factor_idx, 0),
                position(query_idx, scale_factor_idx, len(data["db"].unique()) - 1)]
            )
            for query_idx in range(len(queries))
            for scale_factor_idx in range(len(scale_factors))
        ]
    )
    
    ax.set_xticklabels([f"{sf:,}" for _ in queries for sf in scale_factors], ha="center")

    ax2 = ax.secondary_xaxis(-0.2)
    ax2.set_xlim(ax.get_xlim())
    ax2.set_xticks(
        [
            np.mean(
                [position(query_idx, 0, 0),
                position(query_idx, len(data["scale_factor"].unique()) - 1, len(data["db"].unique()) - 1)]
            )
            for query_idx in range(len(queries))
        ]
    )
    ax2.set_xticklabels(queries, ha="center")
    ax2.tick_params(bottom=False, labelbottom=False, top=True, labeltop=True)

    ax.yaxis.grid(True, color="black")
    ax.set_axisbelow(True)
    ax2.set_xlabel("Query and Scale Factor")
    ax.set_ylabel(f"Scale Factor / Query Time (${unit}^{{-1}}$)")
    ax.yaxis.set_major_locator(MaxNLocator(nbins="auto", steps=[1, 2, 4, 5, 10]))
    handles, labels = ax.get_legend_handles_labels()
    unique_labels = {label: handle for handle, label in zip(handles, labels)}
    ax.legend(
        unique_labels.values(),
        unique_labels.keys(),
        title="Database",
        frameon=True,
        facecolor="white",
        framealpha=1,
        edgecolor="white",
        loc="upper center",
        bbox_to_anchor=(0.5, 1.2),
        ncol=len(unique_labels),
    )

    for spine in ax.spines.values():
        spine.set_visible(False)

    plt.tight_layout(pad=0.0)
    return plt


def do(bench_dir: Path, output_dir: Path) -> None:
    with open(bench_dir / "data_logs.json", "r") as f:
        data = json.load(f)
    plt = plot_data(process_data(data, "data_logs"))
    plt.savefig(output_dir / "data_logs.pgf", backend="pgf")
