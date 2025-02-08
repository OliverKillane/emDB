import json
from pathlib import Path
import pandas as pd
import matplotlib.pyplot as plt
from .utils import process_data


def plot(data: dict):
    # Define the specific sizes for each benchmark
    sizes_per_benchmark = {
        "long sequence": ["100"],
        "recursive ident": ["64"],
        "parse nothing": ["()"],
        "large groups": ["Size { depth: 4, branch_width: 4, leaf_width: 8096 }"],
    }

    # Function to calculate speedup
    def calculate_speedup(benchmark, size):
        speeds = {}
        chumsky_time = data[benchmark]["ChumskyProc"][size]["time"]["mean"]
        for parser in data[benchmark]:
            if size in data[benchmark][parser]:
                parser_time = data[benchmark][parser][size]["time"]["mean"]
                speeds[parser] = chumsky_time / parser_time
        return speeds

    bar_charts = [
            ("sequence(100)", "long sequence", "100"),
            ("sequence(1M)", "long sequence", "1000000"),
            ("recursive(1)", "recursive ident", "1"),
            ("recursive(2)", "recursive ident", "2"),
            ("recursive(64)", "recursive ident", "64"),
            ("nothing", "parse nothing", "()"),
            ("tree", "large groups", "Size { depth: 16, branch_width: 2, leaf_width: 1 }"),
            ("large tree", "large groups", "Size { depth: 4, branch_width: 4, leaf_width: 8096 }"),
        ]
    # Prepare data for plotting
    plot_data = {
        name: calculate_speedup(benchmark, size)
        for name, benchmark, size in bar_charts
    }
    
    # print(plot_data)

    # Create a single figure with subplots to place the graphs side by side with thinner bars
    fig, axs = plt.subplots(1, len(bar_charts), figsize=(10, 4))

    colors = plt.cm.get_cmap("tab10", len(next(iter(plot_data.values()))))

    # Plot each benchmark's data in its subplot with thinner bars
    for idx, (benchmark, speeds) in enumerate(plot_data.items()):
        df = pd.DataFrame(list(speeds.items()), columns=["Parser", "Speedup"])
        # df.plot(
        #     kind="bar", x="Parser", y="Speedup", ax=axs[idx], legend=False, width=0.4
        # )  # Adjusting the width to make bars thinner
        bars = axs[idx].bar(
            df["Parser"],
            df["Speedup"],
            color=[colors(i) for i in range(len(df))],
            width=1,
        )  # Adjusting the width to make bars thinner
        axs[idx].set_title(f"   ")
        if idx == 0:
            axs[idx].set_ylabel("Relative Speedup against Chumsky-Proc")
        axs[idx].set_xlabel(f"{benchmark}")
        # axs[idx].tick_params(axis='x', rotation=45)
        axs[idx].tick_params(axis="x", bottom=False, labelbottom=False)

    fig.legend(
        bars,
        df["Parser"],
        loc="upper center",
        ncol=len(df["Parser"]),
        facecolor="white",
        framealpha=1,
        edgecolor="white",
    )

    plt.tight_layout()
    return plt


def do(bench_dir: Path, output_dir: Path) -> None:
    bench_name = "tokens"
    print(f"Generating {bench_name}...", end="", flush=True)
    with open(bench_dir / f"{bench_name}.json", "r") as f:
        data = json.load(f)

    plt = plot(data)
    plt.savefig(output_dir / f"{bench_name}.pgf", backend="pgf")
    print("✅")
