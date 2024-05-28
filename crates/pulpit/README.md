# Pulpit
A library for generating tablular data structures, used to support <img src="../emdb/docs/logo.drawio.svg" alt="emDB" style="vertical-align: middle;" title="emdb logo" width="70"/>.

Includes both the underlying data structures, the code generation and the macro interface.

## Table Structure
<img src="./docs/table_structure.drawio.svg" alt="alt text" title="image Title" width="1500"/>

Pulpit allows for both entirely nary (one `primary`), entirely decomposed (each column in a different associated), and combinations between.
- Indexes supported are currently limited to just the `Unique` index, kept separately from the data storage.
- Depending on the requirement for transactions, and for the deletion operation, the table structure can be chosen to improve performance.
- Tables differentiate between a mutable and immutable section for each row. This allows optimisatons such as returning references into the table for immutable data.
- Tables are accessed through a safe interface (either macro generated, or directly through a `primary`)

## Table Windows
In order to bind the lifetimes of immutable borrows from inside the table data structure to the lifetime for which the table is not moved requires building a `window` into the table to interact.
FUll explanation is included in [`crate::value`]. 
```rust
use pulpit::column::*;
let mut table = PrimaryRetain::<i32, i32, 1024>::new(1024);
{
    let mut window = table.window();
    // window (and returned references) are valid for this scope    
}  
let mut window_2 = table.window();
// ...
```

## Macro Interface
Macros to generate table implementations (using associateds, with indexes, tracked with a transaction log) are included.

See more in [./examples](./examples).
```rust
pulpit::macros::simple! {
    fields {
        name: String,
        id: usize @ unique(unique_reference_number),
        age: u8,
        fav_rgb_colour: i32,
    },
    updates {
        update_age: [age],
    },
    predicates {
        adults_only: *age > 18,
        age_cap: *age < 100,
    },
    transactions: on,
    deletions: on,
    name: bowling_club
}

let mut x = bowling_club::Table::new(1024);
let mut w = x.window();

// We can then insert some data, which is checked against the predicates and unique constraints
let bill_key = w
    .insert(bowling_club::insert::Insert {
        id: 0,
        fav_rgb_colour: 33,
        name: String::from("Bill Bob"),
        age: 50,
    })
    .unwrap();

// We can also update the data using the update method we described in the macro
w.update_age(
    bowling_club::updates::update_age::Update { age: 51 },
    bill_key,
).unwrap();

// The count is correct
assert_eq!(w.count(), 1);

// By committing the data, it can no longer be easily rolled back
w.commit();
```

## Language Limitations
This implementation could be radically simplified with variadict generics.
- Would allow the column types incide tables to be expressed without macros
- Would allow the coupling of associated tables with a primary to be expressed without macros.

But alas, it has been stuck in several closed RFCs such as [this one from 2013](https://github.com/rust-lang/rust/issues/10124).

For now we have struct generating macros like pulpit's and `tuple_impl_for(..)`.

## TODO
1. Improving performance by specifying invariants (in particular on rollback, when re-accessing indices) using [`std::hint`].
2. Adding a table that clears dropped (referenced data) when a window is dropped.
3. Adding a sorted index.
4. Adding special associated columns for sets (repetitions of the same object), and indexes
5. Fixing lack of macro provided errors for no fields, duplicate or nonexistent columns in updates
