# <img src="./emdb/docs/logo.drawio.svg" alt="emDB" style="vertical-align: middle;" title="emdb logo" width="100"/> Crates
This workspace contains the usable libraries from <img src="./emdb/docs/logo.drawio.svg" alt="emDB" style="vertical-align: middle;" title="emdb logo" width="50"/>.

## Setup
### Basic Development
1. [Get rust](https://www.rust-lang.org/tools/install)
2. Use [cargo](https://doc.rust-lang.org/cargo/) in this directory to build, test, benchmark and create docs
3. Get an IDE supporting rust analyzer (vscode, [rustrover](https://www.jetbrains.com/rust/), clion, zed, nvim, etc. )

### Additional Tools
A basic toolchain is setup ayutomatically when running `cargo` by [`./rust-toolchain.toml`](./rust-toolchain.toml), however some useful tools to additionally install are...
#### [`cargo expand`](https://github.com/dtolnay/cargo-expand)
Expands procedural macros and outputs highlighted expansion to the terminal.
```bash
cd emdb
cargo expand --test scratch
```
*Note for single file expansion (e.g. of a macro rules) we can also use `rustc -Zunpretty=expanded <file>.rs`*

#### [`cargo asm`](https://github.com/pacak/cargo-show-asm)
View intermediate results (mir, llvm, asm) generated by rustc. Allows easy scoping down to the level of individual functions.
```bash
# view the available objects in the dereferencing example
cargo asm -p emdb --example dereferencing

# for the 2nd object option
cargo asm -p emdb --example dereferencing --mir 1 # view the MIR code
cargo asm -p emdb --example dereferencing --mca 1 # view the llvm mca analysis
```

#### [`cargo flamegraph`](https://github.com/flamegraph-rs/flamegraph)
Generates flame graphs using `perf`, from tests and benchmarks. 
```bash
CARGO_PROFILE_BENCH_DEBUG=true cargo flamegraph -p combi --bench tokens
```

#### [`cargo kani`](https://github.com/model-checking/kani)
`kani` is a bit-precise model checker using [CBMC](https://github.com/diffblue/cbmc), it can be used to verify memory 
safety, panic safety (e.g. on asserts) and other behaviour (e.g. arithmetic 
overflows).

```bash
cargo kani
```

`kani` is used to verify the correctness of unsafe code in this project. While Kani 
is sound (no false negatives - `VERIFIED` means proved no errors), verification 
can take a long time, and it is not complete (has false positives - can fail to 
verify correct code).

Furthermore coverage of proofs is important, `kani` only analyses proofs.

### [`cargo pgo`](https://github.com/Kobzol/cargo-pgo)
Profile guided optimisation addon for cargo.

## Develop
### Workspace
All crates part of this project are contained in a single [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).
- *Beware: The project relies on a [fork of divan](https://github.com/OliverKillane/divan) for benchmarks from outside this repo*

```bash
cargo test
cargo bench
```

For test output from prints, and getting panic backtraces, a handy command is
```bash
RUST_BACKTRACE=1 cargo test -- --nocapture
```

### Lockfile
[`Cargo.lock`](./Cargo.lock) is tracked by version control for reproducability ([see this justification](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control)).

### Documentation
```bash
cargo doc                          # public docs
cargo doc --document-private-items # include private documentation
```

If using vscode, the [live preview](vscode:extension/ms-vscode.live-server) can 
be used to view documentation built in the [target directory](../target/doc/emdb/).

## Other Resources
### [Rustonomicon](https://doc.rust-lang.org/nomicon/)
### [Rust Performance Book](https://nnethercote.github.io/perf-book/introduction.html)

## Issues
- For rare failure, see the [./emdb/tests/valid/fixme](./emdb/tests/valid/fixme) directory.
- Nondeterministic compile behaviour for [./emdb/tests/valid/complex/favourite_colours.rs](./emdb/tests/valid/complex/favourite_colours.rs)
