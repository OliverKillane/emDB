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


struct Table {
    groups: (Column,Column,Column),
    index: Index,
    hooks: Predicate, Scan, Unique, ColPredicate,
}

struct TableWindow<'imm> {
    table: &'imm mut Table,
}

impl TableWindow<'imm> {

    fn scan(&self) -> impl Iterator<Item=TableIndex> { todo!() }
    fn update_x(&mut self, field1: i32, field2: String) { todo!() }
    fn update_y(&mut self, field3: i32, field2: String) { todo!() }

    fn get(&self, index: TableIndex) -> (i32, String, &'imm String, i32) { todo!() }

    fn insert() -> ...

}
*/
