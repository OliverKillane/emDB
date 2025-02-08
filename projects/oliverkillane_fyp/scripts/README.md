## Scripts for generating report files
### To Setup
```bash
# from this directory
python3.10 -m venv .venv

source .venv/bin/activate      # for bash
source .venv/bin/activate.fish # for fish

pip install -e .      # to use it
pip install -e .[dev] # for extra dev helpers (mypy, darker)
```

### Note on Implementation
The contained scripts are the among the most ugly, convoluted and decrepit python
I have ever written. Its is a mush of chatgpt, copilot and bodges on top.

### Generating Artifacts
To convert the `.drawio` images to `.pdf`:
```bash
./scripts.convert_drawio # convert all the drawio images
```
Then run the benchmarks you need with the output file as `data.json`, and gather the data.
```bash
DIVAN_WRITE_FILE="data.json" cargo bench
```

Then collect and split the benchmark data
```bash
./scripts/bench --workspace <the directory with the data.json> --output-dir "bench_data"
```

Then generate the graphs you need:
```bash
./scripts/graphs --input-dir bench_data/ --output-dir evaluation/_graphs/ --script=all
```

The paths should now match up with the required from the doc.
- If this does not work for you. Then good luck with this insanity 🫡.
