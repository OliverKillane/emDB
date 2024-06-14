from argparse import ArgumentParser
from pathlib import Path
from typing import Callable
import warnings

from .data_logs import DO as data_logs
from .string_copy import DO as string_copy
from .sales_analytics import DO as sales_analytics
from .user_details import do as user_details


def main() -> None:
    parser = ArgumentParser(
        prog="graphs cli",
        description=f"""Generates graphs for benchmarks needed in the report""",
        epilog="Have a wonderful day!",
    )

    parser.add_argument(
        "--input-dir",
        required=True,
        type=Path,
        help="input containing the benchmarks json files",
    )
    parser.add_argument(
        "--output-dir",
        required=True,
        type=Path,
        help="directory to place the resulting graphs",
    )

    def choose_script(choice: str) -> Callable[[Path, Path], None]:
        match choice:
            case "data_logs":
                return data_logs
            case "string_copy":
                return string_copy
            case "sales_analytics":
                return sales_analytics
            case "user_details":
                return user_details
            case "all":

                def do(input_dir: Path, output_dir: Path) -> None:
                    data_logs(input_dir, output_dir)
                    string_copy(input_dir, output_dir)
                    sales_analytics(input_dir, output_dir)
                    user_details(input_dir, output_dir)

                return do
            case _:
                raise ValueError(f"Invalid choice: {choice}")

    parser.add_argument(
        "--script", required=True, type=choose_script, help="script to run"
    )

    args = parser.parse_args()

    warnings.filterwarnings("ignore")

    args.script(args.input_dir, args.output_dir)
