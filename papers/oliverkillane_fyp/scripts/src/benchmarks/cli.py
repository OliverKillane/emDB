


from argparse import ArgumentParser
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from .data_logs import do as data_logs

def main() -> None:
    parser = ArgumentParser(
        prog="nenchmarks cli",
        description=f"""
        Generates graphs for benchmarks needed in the report
""",
        epilog="Have a wonderful day!",
    )
    
    
    parser.add_argument("--input-dir", required=True, type=Path, help="input containing the benchmarks json files")
    parser.add_argument("--output-dir", required=True, type=Path, help="directory to place the resulting graphs")
    
    def choose_script(choice: str) -> Callable[[Path,Path],None]:
        if choice == "datalogs":
            return data_logs
        else:
            raise ValueError(f"Invalid choice: {choice}")
        
    parser.add_argument("--script", required=True, type=choose_script, help="script to run")
    
    args = parser.parse_args()
    args.script(args.input_dir, args.output_dir)

    