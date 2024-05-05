//! # Interface for PulPit Tables

/*
Capabilities
Insert(fields)
Write(index, fields)
Read(index, fields)
Borrow(index, fields)
Delete
*/

/*

table
(
    hooks,
    access,
    index,
    group(
        mut (
            field,
        )
        immut (
            field,
            field,
        )
    )
    ...
)

*/

/*

hooks

- interface
- scan
- unique
- transaction
- predicate
- limit

*/

/*

Access {
    hooks:
    blagh:
}
*/
