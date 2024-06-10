use divan::{black_box_drop, Bencher};
use embedded_db_comparisons::{
    sales_analytics::{
        duckdb_impl::DuckDB,
        emdb_basic_impl::EmDBBasic,
        emdb_parallel_impl::EmDBParallel,
        emdb_iter_impl::EmDBIter,
        emdb_chunk_impl::EmDBChunk,
        sales_analytics::{Database, Datastore},
        sqlite_impl::SQLite,
        TableConfig,
    },
    utils::{choose, choose_internal, total},
};
use rand::{rngs::ThreadRng, Rng};

#[divan::bench(
    name = "random_workloads",
    types = [SQLite, EmDBParallel, EmDBBasic, EmDBIter, EmDBChunk, DuckDB],
    consts = [1024, 8192, 16384],
    max_time = 10
)]
fn mixed_workload<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let mut rng = rand::thread_rng();
            let config = TableConfig::from_size(SIZE);
            (TableConfig::populate_database(&config, &mut rng), rng, config)
        })
        .bench_local_values(|(mut ds, mut rng, config): (DS, ThreadRng, TableConfig)| {
            let db = ds.db();
            for _ in 0..1000 {
                choose! { rng
                    1 => black_box_drop(db.category_sales(0.2, 2.3)),
                    1 => black_box_drop(db.product_customers(rng.gen_range(0..config.products), 0.9, 1.2)),
                    1 => black_box_drop(db.customer_value(1.5, 8.8, rng.gen_range(0..config.customers))),
                }
            }
        })
}

fn main() {
    divan::main()
}
