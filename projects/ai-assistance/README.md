## Tuning LLMs for a codebase
1. Scrape a codebase for relevant info:
 - Code, file locations, git history, etc
2. Tune an existing open source LLM on the code.
3. Use retreival augmented generation using periodically scraped database.

### Develop
Create an env file at [.env](./.env) with a github personal access token ([instructions](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens))

To just create a venv
```bash
uv venv --python=3.12
source .venv/bin/activate
```

To install dev deps (for the notebooks)
```bash
uv pip install -r pyproject.toml --extra dev
```
In vscode can then open the [notebooks](./notebooks/) and select the python venv to use.

### Notebooks
```bash
uv run --with jupyter jupyter lab 
```

> NOTE: Check GPUs
> See guide [here for pytorch](https://docs.astral.sh/uv/guides/integration/pytorch/#installing-pytorch), using nvidia-smi command to check driver from within wsl.

### Resources
 - [Huggingface 'Fine Tuning on a Single GPU'](https://huggingface.co/learn/cookbook/fine_tuning_code_llm_on_single_gpu)
 - [Huggingface RAG](https://huggingface.co/blog/ray-rag#:~:text=Huggingface%20Transformers%20recently%20added%20the%20Retrieval%20Augmented%20Generation,state%20of%20the%20art%20results%20on%20knowledge-intensive%20tasks)
 - [Huggingface Open Source LLM leaderboard](https://huggingface.co/spaces/open-llm-leaderboard/open_llm_leaderboard#/)
