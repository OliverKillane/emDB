#!/usr/bin/env python3.10

from argparse import ArgumentParser
import os
from pathlib import Path
import subprocess
import shutil
from multiprocessing import Pool
from typing import Any, Callable

THIS_DIR = Path(os.path.dirname(os.path.abspath(__file__)))
REPORT_DIR = THIS_DIR.parent
GRAPHIFY_DIR = THIS_DIR / "graphify.py"


def get_args():
    parser = ArgumentParser(
        prog="report artifact cli",
        description="This cli manages the report artifacts (plots, tables, drawio images)",
        epilog="Have a wonderful day!",
    )
    parser.add_argument(
        "--report-dir",
        type=Path,
        default=REPORT_DIR,
        help="The directory containing the report (its report.tex file)",
    )
    subparsers = parser.add_subparsers(dest="subparser", help="sub-command help")
    drawio_parse = subparsers.add_parser(
        name="drawio", description="Drawio image -> pdf transformations"
    )
    drawio_parse.add_argument(
        "--subdir",
        type=Path,
        default=None,
        help="Convert drawios for the directory",
        metavar="<path>",
    )
    plots_parser = subparsers.add_parser(
        name="plots",
        description="Generate the pre-defined plots from graphify",
    )
    plots_parser.add_argument(
        "--subdir",
        type=Path,
        default=None,
        help="Generate plots for the directory",
        metavar="<path>",
    )
    return parser.parse_args()


def get_files(base_dir: Path, suffix: str):
    files = subprocess.check_output(
        f"find {base_dir} -name '*.{suffix}' -not -name '_*' -type f", shell=True
    )
    return [
        base_dir / Path(filepath)
        for filepath in files.decode("utf-8").split("\n")
        if len(filepath) > 0
    ]


def transform_files(
    reports_dir: Path,
    artifacts_dir: Path,
    subdir: Path | None,
    suffix: str,
    worker: Callable[[tuple[Path, Path, Path, Any]], None],
    aux: Any = None,
):
    # Find all paths within the base_dir that are drawio files
    base_path = reports_dir if subdir is None else reports_dir / subdir
    print(f"Generating {suffix} for {base_path}")
    drawio_base = artifacts_dir if subdir is None else artifacts_dir / subdir
    files = [
        (base_path, drawio_base, path, aux) for path in get_files(base_path, suffix)
    ]
    print(files)
    shutil.rmtree(drawio_base, ignore_errors=True)
    Pool().map(worker, files)


def run_benchmarks(
    experiments_dir: Path, bench_dir: Path, name: str, cache: bool = True
):
    # run the benchmarks wth output
    bench_dir.mkdir(exist_ok=True)
    tables_dest = bench_dir / f"{name}.json"
    if cache and tables_dest.exists():
        print(f"Cached benchmark results for {name} used")
    else:
        print(f"Running benchmark for {name}")
        shutil.rmtree(tables_dest, ignore_errors=True)
        subprocess.check_output(
            ["cargo", "bench", name],
            env=dict(os.environ, DIVAN_WRITE_FILE=str(tables_dest)),
            cwd=experiments_dir,
        )


def drawio_transform(data: (Path, Path, Path, Path)):
    base_path, drawio_base, path, _ = data
    old = base_path / path
    new = (drawio_base / os.path.relpath(path, base_path)).with_suffix(".pdf")
    new.parent.mkdir(parents=True, exist_ok=True)
    subprocess.check_output(
        ["drawio", old, "-x", "--format", "pdf", "-t", "--crop", "-o", new], stderr=subprocess.DEVNULL
    )
    print(f"✅ transformed {new}")


def plot_transform(data: (Path, Path, Path, Path)):
    base_path, plots_base, path, bench_path = data
    graph = path.stem
    new = (plots_base / os.path.relpath(path, base_path)).with_suffix(".pdf")
    new.parent.mkdir(parents=True, exist_ok=True)
    subprocess.check_output(
        [
            f"{GRAPHIFY_DIR} --input {bench_path} --output {new} --format pdf --graph {graph}"
        ],
        shell=True,
    )
    print(f"✅ transformed plot {new}")


def main():
    args = get_args()

    report_dir = args.report_dir
    drawio_dir = report_dir / "_drawio"
    graphs_dir = report_dir / "_plots"
    bench_dir = report_dir / "_bench"
    experiments_dir = report_dir.parent / "experiments"

    if args.subparser == "drawio":
        transform_files(report_dir, drawio_dir, args.subdir, "drawio", drawio_transform)
    if args.subparser == "plots":
        run_benchmarks(experiments_dir, bench_dir, "tables")
        transform_files(
            report_dir,
            graphs_dir,
            args.subdir,
            "graph",
            plot_transform,
            bench_dir / "tables.json",
        )


main()
