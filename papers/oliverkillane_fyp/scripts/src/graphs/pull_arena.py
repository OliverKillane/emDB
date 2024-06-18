import json
from pathlib import Path
from matplotlib.ticker import MaxNLocator
import pandas as pd
import matplotlib.pyplot as plt
from .utils import MEAN_COLOUR_MAP, MEDIAN_COLOUR_MAP, data_rates, get_best_unit_and_scale, process_data

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
        ax.set_xlabel(f"{query}")
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

def fix_names(data):
    for bench in data.keys():
        for impl in data[bench].keys():
            new_impl = impl.split("<")[0]
            data[bench][new_impl] = data[bench].pop(impl)
    

def do(bench_dir: Path, output_dir: Path) -> None:
    bench_name = "pull_arena"
    print(f"Generating {bench_name}...", end="", flush=True)
    with open(bench_dir / f"{bench_name}.json", "r") as f:
        data = json.load(f)
    fix_names(data)
    plt = plot_rates_separate_lines(process_data(data))
    plt.savefig(output_dir / f"{bench_name}.pgf", backend="pgf")
    print("✅")
