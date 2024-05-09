//! Needs to specialise based on mutable or immutable keys and the column they are in.
//!
//! For example

// TODO: read https://www.idryman.org/blog/2017/05/03/writing-a-damn-fast-hash-table-with-tiny-memory-footprints/

struct ConstUniqueIndex<ImmDataField> {
    // mapping: HashMap<()>
    something: ImmDataField,
}
