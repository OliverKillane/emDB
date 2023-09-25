## Fake Examples

Fake code examples to explore the language design of emDB.

The language design is still early (draft v3) so much is likely to change. Was heavily inspired by the flux language.

- `ref` kinds (one way, two way, to row rather than specified column) (inspired by surrealDB)
- `row` and `stream` types (all operators work on either, a row can contain a stream), streams are lazy and for the basis for all operators (volcano).
- `ref` streams / determining where a stream is of values, or references.

  ```rust
  // the first part is a stream of references
  some_table |> where(even(it[some_col])) |< get_delete() |> return;

  // returning a stream of values
  some_table |> where(even(it[some_col])) |> return;
  ```

- auto derive common functions (can derive insert, size, etc) - not sure if this is a good idea?

- Also need to avoid over-expanding the language, anything complex should be left to rust.

- Task based concurrency? Rather than deciding between (single thraded queries, many queries Versus multithreaded query, one query), we just use tasks, tokio spreads them across all available kernel & thus hardware threads.

### Tasks

- [ ] Work out minimum set of high-level operators and constraints required. Specify semantics. Only baked in operators supported, no custom operators.
- [ ] Research other similar concepts
- [ ] Update examples to new specification for use in testing.
- [ ] Write language documentation.
