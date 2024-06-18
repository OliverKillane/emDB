from argparse import ArgumentParser
from dataclasses import dataclass
from pathlib import Path
import json
import os
from pathlib import Path
import subprocess


def run_workspace(path: Path, output_file: Path):
    with open(output_file, "w") as file:
        file.truncate(0)
    subprocess.check_output(
        ["cargo", "bench"],
        env=dict(os.environ, DIVAN_WRITE_FILE=str(output_file)),
        cwd=path,
    )


def split_benchmarks(all_bench_file: Path, output_dir: Path):
    output_dir.mkdir(parents=True, exist_ok=True)
    with open(all_bench_file, "r") as file:
        for line in file.readlines():
            json_data = json.loads(line.strip())
            for name, val in json_data["benchmarks"].items():
                print(name)

            output_file = output_dir / f"{name}.json"
            with open(output_file, "w") as out_file:
                json.dump(val, out_file)
            print(f"✅Written {output_file}")


def main():
    parser = ArgumentParser(
        prog="benchmarking cli",
        description=f"Runs benchmarks and splits the resulting files",
        epilog="Have a wonderful day!",
    )

    parser.add_argument(
        "--workspace",
        required=True,
        type=Path,
        help="workspace to run the benchmarks in",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        type=Path,
        help="directory to place the resulting files",
    )
    parser.add_argument("--rerun", action="store_true", help="rerun the benchmarks")
    args = parser.parse_args()

    ALL_DATA_NAME = "data.json"
    bench_json = args.workspace / ALL_DATA_NAME
    if args.rerun:
        run_workspace(args.workspace, bench_json)
    split_benchmarks(bench_json, args.output_dir)
