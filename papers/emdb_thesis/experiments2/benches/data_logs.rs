use divan::{self, black_box_drop, Bencher};
use experiments2::data_logs::{
    data_logs::{Database, Datastore},
    duckdb_impl::DuckDB,
    emdb_impl::EmDB,
    emdb_inlined_impl::EmDBInlined,
    populate_table,
    sqlite_impl::SQLite,
};

const TABLE_SIZES: [usize; 1] = [2048]; //[1048576, 2097152, 4194304, 8388608, 16777216],

#[divan::bench(
    name = "demote_errors_data_cleaning",
    types = [SQLite, EmDB, EmDBInlined, DuckDB],
    consts = TABLE_SIZES,
)]
fn demote_errors_data_cleaning<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher
        .with_inputs(|| populate_table(&mut rand::thread_rng(), SIZE))
        .bench_local_values(|mut ds: DS| {
            let mut db = ds.db();
            black_box_drop(db.demote_error_logs());
        })
}

#[divan::bench(
    name = "get_errors_per_minute",
    types = [SQLite, EmDB, EmDBInlined, DuckDB],
    consts = TABLE_SIZES,
)]
fn get_errors_per_minute<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher
        .with_inputs(|| populate_table(&mut rand::thread_rng(), SIZE))
        .bench_local_values(|mut ds: DS| {
            let db = ds.db();
            black_box_drop(db.get_errors_per_minute());
        })
}

#[divan::bench(
    name = "get_comment_summaries",
    types = [SQLite, EmDB, EmDBInlined, DuckDB],
    consts = TABLE_SIZES,
)]
fn get_comment_summaries<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher
        .with_inputs(|| populate_table(&mut rand::thread_rng(), SIZE))
        .bench_local_values(|mut ds: DS| {
            let db = ds.db();
            // for the entire database
            black_box_drop(db.get_comment_summaries(0, SIZE));
        })
}

fn main() {
    divan::main()
}
