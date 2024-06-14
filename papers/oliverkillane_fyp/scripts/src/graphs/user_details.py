import json
from pathlib import Path
from .utils import plot_rates_fn, plot_rates_separate, process_data, split_dataframe, plot_rates_separate_lines
    
def do(bench_dir: Path, output_dir: Path) -> None:
    bench_name = "user_details"
    
    print(f"Generating {bench_name}...", end="", flush=True)
    with open(bench_dir / f"{bench_name}.json", "r") as f:
        data = json.load(f)

    processed_data = process_data(data)
    
    dataframes = split_dataframe(processed_data, 3)
        
    for index, frame in enumerate(dataframes):
        plt = plot_rates_separate_lines(frame, include_legend=index==0)
        plt.savefig(output_dir / f"{bench_name}_{index}.pgf", backend="pgf")
    print("✅")
