use divan;
use embedded_db_comparisons::string_copy::{
    copy_string::{self as interface, Database},
    emdb_copy_impl::EmDBCopy,
    emdb_ref_impl::EmDBRef,
    populate_database,
};

const STR_LEN_SIZES: [usize; 3] = [512, 1024, 4096];
const DB_SIZE: usize = 262144; // 2**18

#[divan::bench(
    name = "count values",
    types = [EmDBRef, EmDBCopy],
    consts = STR_LEN_SIZES,
)]
fn count_values<DS: interface::Datastore, const STR_LEN: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| divan::black_box(populate_database(DB_SIZE, STR_LEN)))
        .bench_local_values(|mut ds: DS| {
            {
                let db = ds.db();
                db.count_values();
            }
            ds
        })
}

#[divan::bench(
    name = "count refs",
    types = [EmDBRef, EmDBCopy],
    consts = STR_LEN_SIZES,
)]
fn count_refs<DS: interface::Datastore, const STR_LEN: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| divan::black_box(populate_database(DB_SIZE, STR_LEN)))
        .bench_local_values(|mut ds: DS| {
            {
                let db = ds.db();
                db.count_refs();
            }
            ds
        })
}

#[divan::bench(
    name = "count refs with value deref",
    types = [EmDBRef, EmDBCopy],
    consts = STR_LEN_SIZES,
)]
fn count_values_ignore<DS: interface::Datastore, const STR_LEN: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| divan::black_box(populate_database(DB_SIZE, STR_LEN)))
        .bench_local_values(|mut ds: DS| {
            {
                let db = ds.db();
                db.count_values_ignore();
            }
            ds
        })
}

fn main() {
    divan::main()
}
