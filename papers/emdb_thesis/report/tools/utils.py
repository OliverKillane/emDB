from dataclasses import dataclass
from typing import Any
from matplotlib.ticker import FuncFormatter, LogLocator
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D


@dataclass
class StatRange:
    fastest: int
    mean: int
    median: int
    slowest: int


@dataclass
class Benchmark:
    benchmarks: dict[str, dict[int, StatRange]]  # goes to T = dict[str, T | StatRange]


def process_benchmark(data: Any) -> Benchmark | None:
    def toStatRange(stats: dict[str, int]) -> StatRange:
        return StatRange(
            stats["fastest"], stats["mean"], stats["median"], stats["slowest"]
        )

    try:
        return Benchmark(
            {
                name: {
                    int(param): toStatRange(stats["time"])
                    for (param, stats) in values.items()
                }
                for (name, values) in data.items()
            }
        )
    except Exception as e:
        return None


def plot_line(title: str, param_name: str, bench: Benchmark):
    def time_format(value: int, _):
        units = ["ps", "ns", "μs", "ms"]
        for unit in units:
            if value < 1000:
                return f"{value} {unit}"
            value /= 1000
        return f"{value} s"

    def param_format(value, _):
        return f"{value:.0f}"

    data = bench.benchmarks

    # Create a color map
    colors = plt.cm.viridis([i * (1 / len(data)) for i in range(len(data))])
    marked_params = set()

    plt.figure(figsize=(20, 10))

    for i, (name, values) in enumerate(data.items()):
        # Sort the values by the parameter
        sorted_values = sorted(values.items())
        parameters, stats = zip(*sorted_values)
        mean_values, min_values, max_values = zip(
            *[(stat.mean, stat.fastest, stat.slowest) for stat in stats]
        )

        plt.plot(parameters, mean_values, color=colors[i], label=name)
        plt.fill_between(parameters, min_values, max_values, color=colors[i], alpha=0.2)

        for param in parameters:
            marked_params.add(param)

    for param in marked_params:
        plt.axvline(x=param, color="b", linestyle="--")

    # Add labels, title, and legend
    plt.xlabel(param_name)
    plt.ylabel("Time")
    plt.title(title)
    ax = plt.gca()
    ax.set_xscale("log")
    ax.xaxis.set_major_locator(LogLocator(base=2))
    ax.xaxis.set_major_formatter(FuncFormatter(param_format))
    ax.yaxis.set_major_formatter(FuncFormatter(time_format))
    ax.set_xlim(left=1)
    # plt.legend(bbox_to_anchor=(1.05, 1), loc='upper left')
    plt.legend()


def common_line(name: str, title: str, param: str):
    def plot(data: Any):
        bench = process_benchmark(data["benchmarks"]["tables"][name])
        # del bench.benchmarks["DuckDB"]
        plot_line(title, param, bench)

    return plot


def plot_rates(title: str, param_name: str, bench: Benchmark):
    def param_format(value, _):
        return f"{value:.0f}"

    data = bench.benchmarks

    # Create a color map
    colors = plt.cm.viridis([i * (1 / len(data)) for i in range(len(data))])
    marked_params = set()

    plt.figure(figsize=(20, 10))

    for i, (name, values) in enumerate(data.items()):
        # Sort the values by the parameter
        sorted_values = sorted(values.items())
        parameters, mean_values, min_values, max_values = zip(
            *[
                (
                    param,
                    param / (stat.mean / 10**12),
                    param / (stat.fastest / 10**12),
                    param / (stat.slowest / 10**12),
                )
                for (param, stat) in sorted_values
            ]
        )

        plt.plot(parameters, mean_values, color=colors[i], label=name)
        plt.fill_between(parameters, min_values, max_values, color=colors[i], alpha=0.2)

        for param in parameters:
            marked_params.add(param)

    for param in marked_params:
        plt.axvline(x=param, color="b", linestyle="--")

    # Add labels, title, and legend
    plt.xlabel(param_name)
    plt.ylabel("Rows / s")
    plt.title(title)
    ax = plt.gca()
    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.xaxis.set_major_locator(LogLocator(base=2))
    ax.yaxis.set_major_locator(LogLocator(base=10))
    ax.xaxis.set_major_formatter(FuncFormatter(param_format))
    ax.set_xlim(left=1)
    plt.legend()


def common_rates(name: str, title: str, param: str):
    def plot(data: Any):
        bench = process_benchmark(data["benchmarks"]["tables"][name])
        # del bench.benchmarks["DuckDB"]
        plot_rates(title, param, bench)

    return plot


def plot_speedup(title: str, param_val: int, bench: Benchmark):
    def speedup_format(value, _):
        return f"{value:.0f}x"

    data = bench.benchmarks

    plt.figure(figsize=(12, 4))

    min_val = min(value[param_val].mean for value in data.values())
    max_val = max(value[param_val].slowest for value in data.values())
    labels = []

    data_items = sorted(list(data.items()), key=lambda item: item[1][param_val].mean)

    HEIGHT = 0.2

    for i, (name, values) in enumerate(data_items):
        pval = values[param_val]
        mean_val = pval.mean / min_val
        fastest_val = pval.fastest / min_val
        slowest_val = pval.slowest / min_val
        plt.fill_betweenx(
            [i - HEIGHT, i + HEIGHT], fastest_val, slowest_val, color="lightgray"
        )
        plt.plot([mean_val, mean_val], [i - HEIGHT, i + HEIGHT], color="red")
        plt.plot([fastest_val, fastest_val], [i - HEIGHT, i + HEIGHT], color="blue")
        plt.plot([slowest_val, slowest_val], [i - HEIGHT, i + HEIGHT], color="green")
        plt.text(
            mean_val,
            i + HEIGHT,
            f" {mean_val:.2f}x",
            color="red",
            ha="center",
            va="bottom",
        )
        plt.text(
            mean_val,
            i - HEIGHT,
            f"{round((param_val / (pval.mean / 10**12)))} /s",
            color="red",
            ha="center",
            va="top",
        )
        plt.text(
            fastest_val,
            i,
            f"{fastest_val:.2f}x ",
            color="blue",
            ha="right",
            va="center",
        )
        plt.text(
            slowest_val,
            i,
            f" {slowest_val:.2f}x",
            color="green",
            ha="left",
            va="center",
        )
        labels.append(name)

    custom_lines = [
        Line2D([0], [0], color="red", lw=2),
        Line2D([0], [0], color="blue", lw=2),
        Line2D([0], [0], color="green", lw=2),
    ]
    plt.legend(custom_lines, ["Mean", "Fastest", "Slowest"], loc="lower right")

    plt.yticks(range(len(labels)), labels)
    ax = plt.gca()
    ax.set_xscale("log")
    ax.xaxis.set_major_formatter(FuncFormatter(speedup_format))
    ax.set_xlim(right=max_val / min_val * 3)
    ax.grid(which="major", linestyle="-", linewidth="0.5", color="black", axis="x")
    ax.set_axisbelow(True)
    plt.title(title)
    plt.xlabel("Time Multiplier")


def common_boxplot(name: str, title: str, param: int):
    def plot(data: Any):
        bench = process_benchmark(data["benchmarks"]["tables"][name])
        plot_speedup(title, param, bench)

    return plot
