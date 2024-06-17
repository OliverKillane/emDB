from argparse import ArgumentParser
from pathlib import Path
from typing import Callable
import warnings

from .data_logs import DO as data_logs
from .string_copy import DO as string_copy
from .sales_analytics import DO as sales_analytics
from .user_details import do as user_details
from .tokens import do as tokens
from .rc_vs_brw import DO as rc_vs_brw
from .pull_arena import do as pull_arena
from .iterators import DO as iterators

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
        names = {
            "data_logs": data_logs,
            "pull_arena": pull_arena,
            "string_copy": string_copy,
            "sales_analytics": sales_analytics,
            "user_details": user_details,
            "tokens": tokens,
            "iterators": iterators,
            "rc_vs_brw": rc_vs_brw,
        }
        
        def do_all(input_dir: Path, output_dir: Path) -> None:
            for fn in names.values():
                fn(input_dir, output_dir)
        
        if choice == "all":
            return do_all
        else:
            return names.get(choice)
    
    
    parser.add_argument(
        "--script", required=True, type=choose_script, help="script to run"
    )

    args = parser.parse_args()

    warnings.filterwarnings("ignore")

    args.script(args.input_dir, args.output_dir)
