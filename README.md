<img src="./emdb/docs/logo/simple.drawio.svg" alt="emDB" title="emdb logo" width="300"/>

Composed of [combi](./combi/), [enumtrait](./enumtrait/) and [emDB](./emdb/)

## What is this?
A final year project to develop an embedded database using schema compilation.

### [Design Proposal](./docs/)
### Contributing
As a final year project to be marked, no code contributions can be accepted at this time.

### Setup
1. [Get rust](https://www.rust-lang.org/tools/install)
2. Clone this repo, and use [cargo](https://doc.rust-lang.org/cargo/) to build, test, benchmark

### Workspace
All crates part of this project are contained in a single [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).
- *Beware: The experiments require building duckDB and sqlite, to avoid this code, `cargo build` from within the crates you need*
- *Beware: The project relies on a [fork of divan](https://github.com/OliverKillane/divan) for benchmarks from outside this repo*

### Documentation
```bash
cargo doc                          # public docs
cargo doc --document-private-items # include private documentation
```

If using vscode, the [live preview](vscode:extension/ms-vscode.live-server) can 
be used to view documentation built in the [target directory](../target/doc/emdb/).
