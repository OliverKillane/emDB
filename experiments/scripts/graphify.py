#!/usr/bin/env python3.10
from argparse import ArgumentParser
import matplotlib.pyplot as plt
import json
from pathlib import Path
from typing import Any, Callable
from utils import common_line, common_boxplot
import tikzplotlib

GRAPH_FUNCTIONS: dict[str, Callable[[Any], None]] = {
    "random_inserts" : common_line("random_inserts", "Time taken to insert rows", "Number of rows"),
    "random_get_ids" : common_line("random_get_ids", "Time taken to retrieve all rows in random order", "Number of rows"),
    "snapshot" : common_line("snapshot", "Time taken to return a snapshot of the all rows in the database", "Number of rows"),
    "get_total_prem_credits" : common_line("get_total_prem_credits", "Time taken to filter and sum the credits of all premium users", "Number of rows"),
    "reward_premium_users" : common_line("reward_premium_users", "Time taken to update all premium users credits, and return the change in credits", "Number of rows"),
    "random_workloads" : common_line("random_workloads", "Time taken to perform a number of random actions on the database", "Number of actions"),
    "random_insert_boxplot" : common_boxplot("random_inserts", "Comparative time for inserts of 128 rows", 128),
}
PARAM_PDF = "pdf"
PARAM_TEX = "tex"
PARAM_SHOW = "show"

def main():
    args = ArgumentParser(
        prog="divan benchmark processing and graph creation",
        description="Takes json output from divan and converts it into graphs for latex",
        epilog="Have a wonderful day!"
    )
    args.add_argument("--input", required=True, type=Path, help="The input divan json file to process")
    args.add_argument("--output", required=True, type=Path, help="The output tex file to write to")
    args.add_argument("--graph", required=True, type=str, choices=GRAPH_FUNCTIONS.keys(), help="Choose the experiment graph you")
    args.add_argument("--format", type=str, default=PARAM_TEX,choices=[PARAM_SHOW, PARAM_TEX, PARAM_PDF], help="Determine the graph output (file or just show/display)")
    
    params = args.parse_args()
    input_file = json.load(params.input.open())
    GRAPH_FUNCTIONS[params.graph](input_file)
    
    if params.format == PARAM_PDF:
        plt.savefig(params.output, bbox_inches='tight')
    elif params.format == PARAM_TEX:
        tikzplotlib.save(params.output)
    else:
        plt.show()

main()
