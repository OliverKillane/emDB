import math
from typing import Callable
import pandas as pd
import json
from pathlib import Path
import numpy as np
import pandas as pd
from matplotlib.ticker import MaxNLocator
import matplotlib.pyplot as plt


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


def process_data(bench_data: dict) -> pd.DataFrame:
    benchmark_list = []
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
                            "mean": result["time"]["mean"],
                            "median": result["time"]["median"],
                            "fastest": result["time"]["fastest"],
                            "slowest": result["time"]["slowest"],
                        }
                    )

    return pd.DataFrame(benchmark_list)


def data_rates(data: pd.DataFrame):
    columns_to_update = ["mean", "median", "fastest", "slowest"]
    for col in columns_to_update:
        name = f"{col}_rate"
        data[name] = data["scale_factor"] / data[col]


MEAN_COLOUR_MAP = {
    "EmDB": "purple",
    "EmDBCopy": "green",
    "EmDBRef" : "red",
    "DuckDB": "yellow",
    "SQLite": "blue",
}

MEDIAN_COLOUR_MAP = {
    "EmDB": "indigo",
    "EmDBCopy": "darkgreen",
    "EmDBRef" : "darkred",
    "DuckDB": "goldenrod",
    "SQLite": "darkcyan",
}


def plot_rates(data_unscaled: pd.DataFrame) -> plt:
    data_rates(data_unscaled)
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
        return (
            (query_idx * QUERY_SPACING)
            + ((query_idx * scale_factors + scale_factor_idx) * SCALE_SPACING)
            + (
                (
                    ((query_idx * scale_factors + scale_factor_idx) * databases)
                    + db_index
                )
                * (WIDTH + SPACING)
            )
        )

    bar_positions = []
    for q_idx, query in enumerate(queries):
        for sf_idx, sf in enumerate(scale_factors):
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
                        color=MEAN_COLOUR_MAP.get(db, "black"),
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
                [
                    position(query_idx, scale_factor_idx, 0),
                    position(query_idx, scale_factor_idx, len(data["db"].unique()) - 1),
                ]
            )
            for query_idx in range(len(queries))
            for scale_factor_idx in range(len(scale_factors))
        ]
    )

    ax.set_xticklabels(
        [f"{sf:,}" for _ in queries for sf in scale_factors], ha="center"
    )

    ax2 = ax.secondary_xaxis(-0.2)
    ax2.set_xlim(ax.get_xlim())
    ax2.set_xticks(
        [
            np.mean(
                [
                    position(query_idx, 0, 0),
                    position(
                        query_idx,
                        len(data["scale_factor"].unique()) - 1,
                        len(data["db"].unique()) - 1,
                    ),
                ]
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


def plot_rates_separate(data_unscaled: pd.DataFrame, include_legend=True) -> plt:    
    WIDTH = 0.1

    def position(scale_factor_idx: int, db_index: int) -> float:
        SPACING = 0  # Spacing between groups of bars
        SCALE_SPACING = 0.05  # Scale factor between queries
        databases = len(data["db"].unique())
        return (
            +(scale_factor_idx * SCALE_SPACING)
            + (databases * (WIDTH + SPACING)) * scale_factor_idx
            + (db_index * (WIDTH + SPACING))
        )

    data_rates(data_unscaled)
    data_unscaled = data_unscaled.sort_values(by="scale_factor")

    queries = data_unscaled["query"].unique()
    num_queries = len(queries)

    fig, axes = plt.subplots(
        nrows=1, ncols=num_queries, figsize=(10, 4), sharex=True
    )
    if num_queries == 1:
        axes = [axes]
    
    for ax, query in zip(axes, queries):
        query_data = data_unscaled[data_unscaled["query"] == query]
        max_value = query_data[["slowest_rate"]].max().max()
        unit, scale = get_best_unit_and_scale(max_value)

        data = query_data.copy()
        data[["mean_rate", "median_rate", "fastest_rate", "slowest_rate"]] = (
            data[["mean_rate", "median_rate", "fastest_rate", "slowest_rate"]] * scale
        )

        scale_factors = data["scale_factor"].unique()
        x = range(len(scale_factors))

        for sf_idx, sf in enumerate(scale_factors):
            for i, db in enumerate(data["db"].unique()):
                db_data = data[
                    (data["db"] == db)
                    & (data["query"] == query)
                    & (data["scale_factor"] == sf)
                ]
                pos = position(sf_idx, i)
                if not db_data.empty:
                    ax.bar(
                        pos,
                        db_data["mean_rate"].values[0],
                        width=WIDTH,
                        label=db if sf_idx == 0 else "",
                        color=MEAN_COLOUR_MAP.get(db, "black"),
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
                    [
                        position(scale_factor_idx, 0),
                        position(scale_factor_idx, len(data["db"].unique()) - 1),
                    ]
                )
                for scale_factor_idx in range(len(scale_factors))
            ]
        )
        ax.set_xticklabels([f"{sf}" for sf in scale_factors], ha="center")
        ax.set_xlabel("Scale Factor")
        ax.yaxis.set_label_coords(-0.1, 1.02)
        ax.set_ylabel(f"${unit}^{{-1}}$", rotation=0, fontsize=10)
        ax.yaxis.set_major_locator(MaxNLocator(nbins="auto", steps=[1, 2, 4, 5, 10]))
        ax.yaxis.grid(True, color="black")
        ax.yaxis.set_label_coords(-0.0, -0.09)
        ax.set_axisbelow(True)
        ax.set_title(query)
    
    leftmost_ax = axes[0]
    bbox = leftmost_ax.get_position()
    fig.text(bbox.x0 - 0.1, 0.5, 'Scale Factor / Bench Time', va='center', rotation='vertical', fontsize=10)
    
    if include_legend:
        handles, labels = axes[0].get_legend_handles_labels()
        fig.legend(
            handles,
            labels,
            title="Database",
            frameon=True,
            facecolor="white",
            framealpha=1,
            edgecolor="white",
            loc="upper center",
            bbox_to_anchor=(0.5, 1.0),
            ncol=len(labels),
        )

    for ax in axes:
        for spine in ax.spines.values():
            spine.set_visible(False)

    # Adjust the layout to minimize whitespace
    # plt.tight_layout(pad=0.1)
    plt.subplots_adjust(left=0.12, right=0.95, top=0.9, bottom=0.1, hspace=0.2)

    return plt

def plot_rates_separate_lines(data_unscaled: pd.DataFrame, include_legend=True) -> plt:    
    WIDTH = 0.1
    BAR_WIDTH=0.01

    def position(scale_factor_idx: int) -> float:
        SCALE_SPACING = 0.05  # Scale factor between queries
        return (
            (scale_factor_idx * SCALE_SPACING)
        )

    data_rates(data_unscaled)
    data_unscaled = data_unscaled.sort_values(by="scale_factor")

    queries = data_unscaled["query"].unique()
    num_queries = len(queries)

    fig, axes = plt.subplots(
        nrows=1, ncols=num_queries, figsize=(10, 4), sharex=True
    )
    if num_queries == 1:
        axes = [axes]
    
    for index, (ax, query) in enumerate(zip(axes, queries)):
        
        query_data = data_unscaled[data_unscaled["query"] == query]
        max_value = query_data[["slowest_rate"]].max().max()
        unit, scale = get_best_unit_and_scale(max_value)

        data = query_data.copy()
        data[["mean_rate", "median_rate", "fastest_rate", "slowest_rate"]] = (
            data[["mean_rate", "median_rate", "fastest_rate", "slowest_rate"]] * scale
        )
        scale_factors = data["scale_factor"].unique()

        mean_values = {db: [] for db in data["db"].unique()}
        for sf_idx, sf in enumerate(scale_factors):
            for db in data["db"].unique():
                db_data = data[
                    (data["db"] == db)
                    & (data["query"] == query)
                    & (data["scale_factor"] == sf)
                ]
                pos = position(sf_idx)
                colour = MEAN_COLOUR_MAP.get(db, "black")
                med_colour = MEDIAN_COLOUR_MAP.get(db, "black")
                if not db_data.empty:
                    mean = db_data["mean_rate"].values[0]
                    fastest = db_data["fastest_rate"].values[0]
                    slowest = db_data["slowest_rate"].values[0]
                    median = db_data["median_rate"].values[0]
                    
                    mean_values[db].append((pos, mean))
                    
                    ax.plot(
                        [pos, pos], [fastest, slowest], color=colour
                    )  # Vertical line
                    ax.plot(
                        [pos - BAR_WIDTH / 2, pos + BAR_WIDTH / 2],
                        [fastest, fastest],
                        color=colour,
                    )  # Fastest time as a horizontal line
                    ax.plot(
                        [pos - BAR_WIDTH / 2, pos + BAR_WIDTH / 2],
                        [slowest, slowest],
                        color=colour,
                    )  # Slowest time as a horizontal line
                    ax.plot(pos, median, color=med_colour, marker='D', linestyle='None')
                    # ax.plot(
                    #     [pos - BAR_WIDTH / 2, pos + BAR_WIDTH / 2],
                    #     [median, median],
                    #     color=colour,
                    # )  # Median as dotted line
                    ax.scatter(pos, mean, color=colour, marker='o')

        for db, values in mean_values.items():
            positions, means = zip(*values)
            ax.plot(positions, means, label=db, color=MEAN_COLOUR_MAP[db])
    
        ax.set_xticks(
            [
                position(scale_factor_idx)
                for scale_factor_idx in range(len(scale_factors))
            ]
        )
        ax.set_xticklabels([f"{sf}" for sf in scale_factors], ha="center")
        ax.set_xlabel(f"{query}: scale factor")
        ax.set_ylabel(f"Scale Factor / Time $\\left({unit}^{{-1}}\\right)$", rotation=90, fontsize=8)
        ax.yaxis.set_major_locator(MaxNLocator(nbins="auto", steps=[1, 2, 4, 5, 10]))
        ax.yaxis.grid(True, color="black")
        ax.set_axisbelow(True)
        # ax.set_title(query)
    
    if include_legend:
        handles, labels = axes[0].get_legend_handles_labels()
        fig.legend(
            handles,
            labels,
            title="Database",
            frameon=True,
            facecolor="white",
            framealpha=1,
            edgecolor="white",
            loc="upper center",
            # bbox_to_anchor=(0.5, 1.1),
            ncol=len(labels),
        )

    for ax in axes:
        for spine in ax.spines.values():
            spine.set_visible(False)

    # Adjust the layout to minimize whitespace
    plt.tight_layout(pad=0.1)
    plt.subplots_adjust(left=0.07, right=0.95, top=0.9, bottom=0.1, hspace=0.2)

    return plt

def split_dataframe(df, queries_per_chunk):
    unique_queries = df['query'].unique()
    # Create a list of DataFrames each containing at most `queries_per_chunk` unique queries
    dataframes = [df[df['query'].isin(unique_queries[i:i + queries_per_chunk])]
                  for i in range(0, len(unique_queries), queries_per_chunk)]
    return dataframes

def plot_rates_fn(bench_name: str) -> Callable[[Path, Path], None]:
    def do(bench_dir: Path, output_dir: Path) -> None:
        print(f"Generating {bench_name}...", end="", flush=True)
        with open(bench_dir / f"{bench_name}.json", "r") as f:
            data = json.load(f)
        plt = plot_rates_separate_lines(process_data(data))
        plt.savefig(output_dir / f"{bench_name}.pgf", backend="pgf")
        print("✅")

    return do
