use divan::{self, black_box_drop, Bencher};
use embedded_db_comparisons::data_logs::{
    copy_selector::EmDBCopy,
    data_logs::{Database, Datastore},
    duckdb_impl::DuckDB,
    emdb_columnar_impl::EmDBColumnar,
    emdb_impl::EmDB,
    populate_table,
    sqlite_impl::SQLite,
};

const TABLE_SIZES: [usize; 6] = [2048, 8192, 32768, 65536, 131072, 262144];

#[divan::bench(
    name = "data cleaning",
    types = [EmDB, EmDBCopy, DuckDB, SQLite, EmDBColumnar],
    consts = TABLE_SIZES,
    sample_size = 5,
    sample_count = 3,
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
    name = "errors per minute",
    types = [EmDB, EmDBCopy, DuckDB, SQLite, EmDBColumnar],
    consts = TABLE_SIZES,
    sample_size = 5,
    sample_count = 3,
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
    name = "comment summaries",
    types = [EmDB, EmDBCopy, DuckDB, SQLite, EmDBColumnar],
    consts = TABLE_SIZES,
    sample_size = 5,
    sample_count = 3,
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
