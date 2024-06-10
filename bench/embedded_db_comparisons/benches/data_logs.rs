use divan::{self, black_box_drop, Bencher};
use embedded_db_comparisons::data_logs::{
    data_logs::{Database, Datastore},
    duckdb_impl::DuckDB,
    emdb_table_thunderdome_impl::EmDBThunderdome,
    emdb_iter_impl::EmDBIter,
    populate_table,
    sqlite_impl::SQLite,
};

const TABLE_SIZES: [usize; 1] = [32384]; // [524288, 1048576, 2097152];

#[divan::bench(
    name = "demote_errors_data_cleaning",
    types = [EmDBIter, EmDBThunderdome, SQLite, DuckDB],
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
    name = "get_errors_per_minute",
    types = [EmDBIter, EmDBThunderdome, SQLite, DuckDB],
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
    name = "get_comment_summaries",
    types = [EmDBIter, EmDBThunderdome, SQLite, DuckDB],
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
