## Embedded Databases Comparisons
The duckdb and sqlite dependencies are very large as they include the respective databases.

- For duckDB compilation may fail if there is insufficient memory and swap.
- Both the duckDB and sqlite benchmarks take a very long time compared to the rust implemented comparisons.

Consider increasing swap to avoid build failures.

```bash
cargo bench
```

#### On WSL:
```toml
# In your windows home directory, in .wslconfig
# settings apply across all Linux distros running on WSL 2
# Can see memory in wsl2 with "free -m"

[wsl2]
# Limits VM memory to use no more than 48 GB, defaults to 50% of ram
memory=8GB

# Sets the VM to use 8 virtual processors
processors=8

# Sets the amount of swap storage space to 8GB, default is 25% of available RAM
swap=16GB
```
