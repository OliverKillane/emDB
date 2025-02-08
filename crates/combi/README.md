# Combi
## What is this?

A simple combinator library, allowing functions that succeed, fail or can be _continued_/ignored to be composed.

<img src="./docs/combi_trait.drawio.svg" alt="alt text" title="image Title" width="1500"/>

Before you ask: _"Isn't this just functions with extra steps"_, no! Its monads with extra steps.

Different specialised combinators can be built atop combi, typically parsing logic.
- *Combi*s are similar to typical parser combinators, but the addition of continuations enable them to support multiple syntax errors, without requiring error nodes in a more complex AST
- The *Combi*s make no assumptions about input structure, allowing them to compute on buffers (like regular parsers) but also streams (with no backtracking), or trees (such as rust [`proc_macro2::TokenTree`]s)

This was build as part of the [emDB project](./../emdb).

## Included Specialisations
### Tokens
Parser combinators for tokenstreams, allowing LL(1) grammars (recursive descent, no backtracking).
- Produce compiler diagnostics using [`proc_macro_error2`]
- Can recover from failures easily, with custom recovery logic

It is implemented simply (explicit peeking for lookahead, explitit recovery), and is intended to be as close to zero-cost over a comparably features hand-rolled parser.

For maximum size cases of the tokens benchmarks:
```
Timer precision: 15 ns
tokens              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ large_groups                   │               │               │               │         │
│  ├─ ChumskyProc   268.3 ms      │ 336.1 ms      │ 284.2 ms      │ 287.1 ms      │ 100     │ 100
│  ├─ CombiParser   42.38 ms      │ 54.22 ms      │ 45.18 ms      │ 45.44 ms      │ 100     │ 100
│  ╰─ HandRolled    13.54 ms      │ 18.11 ms      │ 15.16 ms      │ 15.49 ms      │ 100     │ 100
├─ long_sequence                  │               │               │               │         │
│  ├─ ChumskyProc   109.2 ms      │ 162.8 ms      │ 120.3 ms      │ 123 ms        │ 100     │ 100
│  ├─ CombiParser   36.48 ms      │ 55.93 ms      │ 42.9 ms       │ 43.44 ms      │ 100     │ 100
│  ╰─ HandRolled    27.22 ms      │ 54.56 ms      │ 32.25 ms      │ 33.85 ms      │ 100     │ 100
├─ parse_nothing                  │               │               │               │         │
│  ├─ ChumskyProc   64.8 ns       │ 12.59 µs      │ 121.6 ns      │ 238.6 ns      │ 100     │ 1600
│  ├─ CombiParser   28.82 ns      │ 28.97 ns      │ 28.9 ns       │ 28.9 ns       │ 100     │ 6400
│  ╰─ HandRolled    7.177 ns      │ 7.22 ns       │ 7.199 ns      │ 7.199 ns      │ 100     │ 25600
╰─ recursive_ident                │               │               │               │         │
   ├─ ChumskyProc   83.55 µs      │ 254.3 µs      │ 87.62 µs      │ 93.95 µs      │ 100     │ 100
   ├─ CombiParser   9.704 µs      │ 11.2 µs       │ 10.17 µs      │ 10.21 µs      │ 100     │ 100              
   ╰─ HandRolled    4.883 µs      │ 5.794 µs      │ 5.122 µs      │ 5.127 µs      │ 100     │ 100
```

## Potential Improvements
- [ ] parse `seq(recovgroup(P1), P2)` in parallel (tree based data structure is naturally parallel) as `treepar(P1, P2)`, need to get around issues with TokenStream being `!Send` and `!Sync`
- [ ] Optimise the `recursive` parser to remove heap allocation

## Inlining
When aggressively inlining, but not optimising, stack frames can get huge.
 - This can result in stackoverflows during compilation (as the proc macro is run after being dynamically linked with rustc).
 - [scripts/extra.gdb](./scripts/extra.gdb) has some additional helpers for debugging this
 - Can also set relevant flags in [../.argo/config.toml](./../.cargo/config.toml)
 