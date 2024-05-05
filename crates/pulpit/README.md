# Pulpit
A library for generating code for very fast tables.

The library is split into separate modules for:
- Underlying data structures
- Data structure selection code
- Macro interface


## Usage
```rust
use pulpit::arena;

arena!{
    struct 


}


#[arena(
    // checks to enforce, failing operations if encountered
    check [
        pred(my_predicate) as constraint
        unique(name) as uniqueness,
        limit(2300) as capacity_check,
    ],
    // accesses through an index to generate
    access [
        insert() as insert_all_fn,
        update(name) as update_fn,
        delete() as do_thing
    ],
    // ways to get references to rows
    index [
        scan() as get_idx,
        unique(name) as ,
    ]
)]
struct myarena {
    name: String,
    cool: i32
}
```
