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
