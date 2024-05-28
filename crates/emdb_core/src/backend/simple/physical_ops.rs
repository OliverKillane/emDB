
use std::cmp::Ordering;

use super::buffers::{Buffer, BufferUnion, VectorBuffer};

trait Table {
    type Ref;
}

fn update<InputRecord: Clone, UpdateTable: Table, UpdateRecord, Error>(
    input: impl Buffer<InputRecord>, 
    table: &mut UpdateTable, 
    update_expr: impl Fn(&InputRecord) -> UpdateRecord,
    update_access: impl Fn(&mut UpdateTable, UpdateRecord) -> Result<(), Error>
) -> Result<impl Buffer<InputRecord>, Error> {
    input.scan_move().map(
        |rec| {
            let update_rec = update_expr(&rec);
            update_access(table, update_rec)?;
            Ok(rec)
        }
    ).collect::<Result<VectorBuffer<InputRecord>, _>>()
}

fn insert<InputRecord, OutputRecord: Clone, UpdateTable: Table, Error>(
    input: impl Buffer<InputRecord>, 
    table: &mut UpdateTable,
    insert_expr: impl Fn(&mut UpdateTable, InputRecord) -> Result<OutputRecord, Error>,
) -> Result<impl Buffer<OutputRecord>, Error> {
    input.scan_move().map(
        |rec| {
            insert_expr(table, rec)
        }
    ).collect::<Result<VectorBuffer<OutputRecord>, _>>()
}

fn delete<InputRecord: Clone, UpdateTable: Table, Error>(
    input: impl Buffer<InputRecord>,
    table: &mut UpdateTable,
    delete_ref: impl Fn(&InputRecord) -> UpdateTable::Ref,
    delete_expr: impl Fn(&mut UpdateTable, UpdateTable::Ref) -> Result<(), Error>
) -> Result<impl Buffer<InputRecord>, Error> {
    input.scan_move().map(
        |rec| {
            let ref_rec = delete_ref(&rec);
            delete_expr(table, ref_rec)?;
            Ok(rec)
        }
    ).collect::<Result<VectorBuffer<InputRecord>, _>>()
}

fn unique_ref<InputRecord, OutputRecord: Clone, UpdateTable: Table, Field, Error>(
    input: impl Buffer<InputRecord>,
    table: &UpdateTable,
    unique_ref: impl Fn(&InputRecord) -> &Field,
    find_expr: impl Fn(&UpdateTable, &Field) -> Result<UpdateTable::Ref, Error>,
    convert_rec: impl Fn(InputRecord, UpdateTable::Ref) -> OutputRecord,
) -> Result<impl Buffer<OutputRecord>, Error> {
    input.scan_move().map(
        |rec| {
            let field = unique_ref(&rec);
            let ref_rec = find_expr(table, field)?;
            Ok(convert_rec(rec, ref_rec))
        }
    ).collect::<Result<VectorBuffer<OutputRecord>, _>>()
}

fn scan_refs<Record: Clone, ScanTable: Table, Error, Refs: Iterator<Item=ScanTable::Ref>>(
    table: &ScanTable,
    scan_expr: impl Fn(&ScanTable) -> Result<Refs, Error>,
    convert_rec: impl Fn(ScanTable::Ref) -> Record,
) -> Result<impl Buffer<Record>, Error> {
    Ok(scan_expr(table)?.map(convert_rec).collect::<VectorBuffer<_>>())
}

fn deref<InputRecord, ReadRecord, OutputRecord: Clone, DerefTable: Table, Error>(
    input: impl Buffer<InputRecord>,
    table: &DerefTable,
    deref_expr: impl Fn(&InputRecord) -> DerefTable::Ref,
    read_expr: impl Fn(&DerefTable, DerefTable::Ref) -> Result<ReadRecord, Error>,
    convert_rec: impl Fn(InputRecord, ReadRecord) -> OutputRecord,
) -> Result<impl Buffer<OutputRecord>, Error> {
    input.scan_move().map(
        |rec| {
            let ref_rec = deref_expr(&rec);
            let read_rec = read_expr(table, ref_rec)?;
            Ok(convert_rec(rec, read_rec))
        }
    ).collect::<Result<VectorBuffer<OutputRecord>, _>>()
}

fn map<InputRecord, OutputRecord: Clone>(
    input: impl Buffer<InputRecord>,
    mapping: fn(InputRecord) -> OutputRecord
) -> impl Buffer<OutputRecord> {
    input.scan_move().map(mapping).collect::<VectorBuffer<_>>()
}

fn fold<InputRecord, OutputRecord: Clone>(
    input: impl Buffer<InputRecord>,
    init: OutputRecord,
    fold_fn: fn(OutputRecord, InputRecord) -> OutputRecord
) -> OutputRecord {
    input.scan_move().fold(init, fold_fn)
}

fn filter<Record: Clone>(
    input: impl Buffer<Record>,
    filter_fn: fn(&Record) -> bool
) -> impl Buffer<Record> {
    input.scan_move().filter(filter_fn).collect::<VectorBuffer<_>>()
}

fn sort<Record: Clone>(
    input: impl Buffer<Record>,
    sort_by: impl FnMut(&Record, &Record) -> Ordering,
) -> impl Buffer<Record>{
    let mut s = input.scan_move().collect::<Vec<_>>();
    s.sort_by(sort_by);
    VectorBuffer::from(s)
}

fn assert<Record: Clone, Error>(
    input: impl Buffer<Record>,
    assert_fn: impl Fn(&Record) -> bool,
    gen_error: impl Fn(&Record) -> Error
) -> Result<impl Buffer<Record>, Error> {
    for rec in input.scan_borrow() {
        if !assert_fn(rec) {
            return Err(gen_error(rec))
        }
    }
    Ok(input)
}

fn collect<InputRecord, OutputRecord: Clone>(
    input: impl Buffer<InputRecord>,
    collect_fn: impl Fn(InputRecord) -> OutputRecord,
) -> impl Buffer<OutputRecord> {
    input.scan_move().map(collect_fn).collect::<VectorBuffer<_>>()
}

fn take<Record: Clone>(
    input: impl Buffer<Record>,
    n: usize
) -> impl Buffer<Record>{
    input.scan_move().take(n).collect::<VectorBuffer<_>>()
}



fn union<Inner: Clone>(left: impl Buffer<Inner>, right: impl Buffer<Inner>) -> impl Buffer<Inner> {
    BufferUnion::from((left, right))
}

