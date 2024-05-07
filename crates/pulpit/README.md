# Pulpit
A library for generating code for very fast tables.

The library is split into separate modules for:
- Underlying data structures
- Data structure selection code
- Macro interface


## Usage
```rust
pulpit::table! {
    my_table {
        fields {
            [
                name: String
            ],[
                other: i32,
                other2: usize
            ],
        },
        actions {
            insert() as insert,
            update(name) as update_name,
            delete() as delete,
            update(name, other) as update_all,
            get(name) as get_name 
        },
        hooks {
            predicate(other > 3) as other_size_cons,
            limit(70),
            unique(name) as by_name
        }
    }
}

fn test() {
    let mut t = my_table::Table::new();
    let mut w = t.window();


    let ind = w.insert("Cool".to_owned(), 4, 7).unwrap();
    w.get_name(ind).unwrap()
}
```
