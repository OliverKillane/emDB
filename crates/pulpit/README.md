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
            colmap() [
                { name: String }
            ],
            ref colvec() => [
                mut { 
                    other: i32,
                    other2: usize
                },
                {
                    other3: usize
                }
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

pulpit::auto_table! {
    my_auto_table {
        fields {
            mut {
                thing: String,
                lol: i32
            },
            {
                blagh: [u8;100]
                surname: String,
            }
        },
        actions {
            update() as something: x
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

TODO:
1. Clean up primary, add docs
2. Setup the associated unique index (efficiently) for imm and mut keys
3. Create verification checks and benchmark against generational arena
4. Change the gen to support this change in structure
5. Generate basic structures
6. Transaction generation
7. Macro interface
8. Stream operator library
