# <img src="./emdb/docs/logo.drawio.svg" alt="emDB" style="vertical-align: middle;" title="emdb logo" width="100"/> Crates
This workspace contains the usable libraries from <img src="./emdb/docs/logo.drawio.svg" alt="emDB" style="vertical-align: middle;" title="emdb logo" width="50"/>.

## Setup
1. [Get rust](https://www.rust-lang.org/tools/install)
2. Use [cargo](https://doc.rust-lang.org/cargo/) in this directory to build, test, benchmark and create docs

### Workspace
All crates part of this project are contained in a single [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).
- *Beware: The project relies on a [fork of divan](https://github.com/OliverKillane/divan) for benchmarks from outside this repo*

### Lockfile
[`Cargo.lock`](./Cargo.lock) is tracked by version control for reproducability ([see this justification](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control)).

### Documentation
```bash
cargo doc                          # public docs
cargo doc --document-private-items # include private documentation
```

If using vscode, the [live preview](vscode:extension/ms-vscode.live-server) can 
be used to view documentation built in the [target directory](../target/doc/emdb/).
