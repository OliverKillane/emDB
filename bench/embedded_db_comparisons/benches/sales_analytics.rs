use divan::{black_box_drop, Bencher};
use embedded_db_comparisons::{
    sales_analytics::{
        duckdb_impl::DuckDB,
        emdb_impl::EmDB,
        sales_analytics::{Database, Datastore},
        sqlite_impl::SQLite,
        TableConfig,
    },
    utils::{choose, choose_internal, total},
};
use rand::{rngs::ThreadRng, Rng};

#[divan::bench(
    name = "category sales",
    types = [EmDB, SQLite, DuckDB],
    consts = [1024, 8192, 16384],
    max_time = 10
)]
fn category_sales<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let mut rng = rand::thread_rng();
            let config = TableConfig::from_size(SIZE);
            TableConfig::populate_database(&config, &mut rng)
        })
        .bench_local_values(|mut ds: DS| {
            let db = ds.db();
            black_box_drop(db.category_sales(0.2, 2.3))
        })
}

#[divan::bench(
    name = "product customers",
    types = [EmDB, SQLite, DuckDB],
    consts = [1024, 8192, 16384],
    max_time = 10
)]
fn product_customers<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher
        
        .with_inputs(|| {
            let mut rng = rand::thread_rng();
            let config = TableConfig::from_size(SIZE);
            (
                TableConfig::populate_database(&config, &mut rng),
                rng,
                config,
            )
        })
        .bench_local_values(|(mut ds, mut rng, config): (DS, ThreadRng, TableConfig)| {
            let db = ds.db();
            black_box_drop(db.product_customers(rng.gen_range(0..config.products), 0.9, 1.2))
        })
}

#[divan::bench(
    name = "customer value",
    types = [EmDB, SQLite, DuckDB],
    consts = [1024, 8192, 16384],
    max_time = 10
)]
fn customer_value<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher
        
        .with_inputs(|| {
            let mut rng = rand::thread_rng();
            let config = TableConfig::from_size(SIZE);
            (
                TableConfig::populate_database(&config, &mut rng),
                rng,
                config,
            )
        })
        .bench_local_values(|(mut ds, mut rng, config): (DS, ThreadRng, TableConfig)| {
            let db = ds.db();
            black_box_drop(db.customer_value(1.5, 8.8, rng.gen_range(0..config.customers)))
        })
}

#[divan::bench(
    name = "mixed workload",
    types = [EmDB, SQLite, DuckDB],
    consts = [1024, 8192, 16384],
    max_time = 10
)]
fn mixed_workload<DS: Datastore, const SIZE: usize>(bencher: Bencher) {
    bencher
        
        .with_inputs(|| {
            let mut rng = rand::thread_rng();
            let config = TableConfig{ customers: 0, sales: 0, products: 10 };
            (TableConfig::populate_database(&config, &mut rng), rng, config)
        })
        .bench_local_values(|(mut ds, mut rng, mut config): (DS, ThreadRng, TableConfig)| {
            let mut db = ds.db();
            for _ in 0..SIZE {
                choose! { rng
                    1 => black_box_drop(db.category_sales(0.2, 2.3)),
                    1 => black_box_drop(db.product_customers(rng.gen_range(0..config.products), 0.9, 1.2)),
                    1 => black_box_drop(db.customer_value(1.5, 8.8, rng.gen_range(0..config.customers))),
                    5 => {
                        config = TableConfig::append_database(&config, &TableConfig { customers: 1, sales: 10, products: 0 }, &mut rng, &mut db);
                    },
                }
            }
        })
}

fn main() {
    divan::main()
}
