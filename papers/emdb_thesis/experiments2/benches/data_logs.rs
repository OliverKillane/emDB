use experiments2::data_logs::{populate_table, emdb_impl::EmDB, data_logs::{Database, Datastore}};
use divan::{self, black_box_drop, Bencher};

#[divan::bench(
    name = "demote_errors_data_cleaning",
    types = [EmDB],
    consts = [1048576, 2097152, 4194304, 8388608, 16777216],
    max_time = 10
)]
fn demote_errors_data_cleaning<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher.with_inputs(|| {
        populate_table(&mut rand::thread_rng(), SIZE)
    }).bench_local_values(|mut ds: DS| {
        let mut db = ds.db();
        black_box_drop(db.demote_error_logs());
    })
}

fn main() {
    divan::main()
}