use divan::{black_box, black_box_drop, Bencher};
use embedded_db_comparisons::{
    user_details::{
        duckdb_impl::DuckDB,
        emdb_iter_impl::EmDBIter,
        random_table, random_user,
        sqlite_impl::SQLite,
        user_details::{Database, Datastore},
        GetNewUserKey,
    },
    utils::{choose, choose_internal, total},
};
use rand::Rng;

// const TABLE_SIZES: [usize; 9] = [1, 8, 64, 128, 512, 4096, 16384, 65536, 262144];
const TABLE_SIZES: [usize; 4] = [1, 8, 16, 512];

fn main() {
    divan::main();
}

/// Time taken for a number of inserts of random premium/non-premium
#[divan::bench(
    name = "random_inserts",
    types = [EmDBIter, SQLite, DuckDB],
    consts = TABLE_SIZES
)]
fn inserts<T, const N: usize>(bencher: Bencher)
where
    T: Datastore,
{
    bencher
        .with_inputs(|| {
            let db = T::new();
            let mut rng = rand::thread_rng();

            (
                (0..N).map(|i| random_user(&mut rng, i)).collect::<Vec<_>>(),
                db,
            )
        })
        .bench_local_values(|(users, mut ds)| {
            let mut db = ds.db();
            for (name, prem, initial) in users {
                black_box_drop(db.new_user(name, prem, initial));
            }
        })
}

/// Time taken to get ids in random order
#[divan::bench(
    name = "random_get_ids",
    types = [EmDBIter, SQLite, DuckDB],
    consts = TABLE_SIZES
)]
fn gets<T, const N: usize>(bencher: Bencher)
where
    T: Datastore + GetNewUserKey,
{
    bencher
        .with_inputs(random_table::<N, T>)
        .bench_local_refs(|(ids, ds)| {
            let db = ds.db();
            for id in ids {
                black_box_drop(db.get_info(*id));
            }
        })
}

/// Time taken to get a snapshot
#[divan::bench(
    name = "snapshot",
    types = [EmDBIter, SQLite, DuckDB],
    consts = TABLE_SIZES
)]
fn snapshot<T, const N: usize>(bencher: Bencher)
where
    T: Datastore + GetNewUserKey,
{
    bencher
        .with_inputs(random_table::<N, T>)
        .bench_local_refs(|(_, ds)| {
            let db = ds.db();
            black_box_drop(db.get_snapshot())
        })
}

/// Time taken to get the total credits of premium users
#[divan::bench(
    name = "get_total_prem_credits",
    types = [EmDBIter, SQLite, DuckDB],
    consts = TABLE_SIZES,
    max_time = 1
)]
fn premium_credits<'a, T, const N: usize>(bencher: Bencher)
where
    T: Datastore + GetNewUserKey,
{
    bencher
        .with_inputs(random_table::<N, T>)
        .bench_local_refs(|(_, ds)| {
            let db = ds.db();
            black_box_drop(db.total_premium_credits())
        })
}

/// Time taken to reward premium users
#[divan::bench(
    name = "reward_premium_users",
    types = [EmDBIter, SQLite, DuckDB],
    consts = TABLE_SIZES,
    max_time = 1
)]
fn reward_premium<T, const N: usize>(bencher: Bencher)
where
    T: Datastore + GetNewUserKey,
{
    bencher
        .with_inputs(random_table::<N, T>)
        .bench_local_refs(|(_, ds)| {
            let mut db = ds.db();
            black_box_drop(db.reward_premium(2f32))
        })
}

/// Random workload of N actions
#[divan::bench(
    name = "random_workloads",
    types = [EmDBIter, SQLite, DuckDB],
    consts = [1024, 2048, 4096],
    max_time = 100
)]
fn mixed_workload<DS, const N: usize>(bencher: Bencher)
where
    DS: Datastore + GetNewUserKey,
{
    bencher.bench_local(|| {
        let mut ds = DS::new();
        let mut db = ds.db();
        let mut rng = rand::thread_rng();

        // avoid reallocations
        let mut ids = Vec::with_capacity(N);
        ids.push(DS::new_user_wrap(&mut db, String::from("bob"), true, Some(3)));

        for _ in 0..N {
            choose! { rng
                10 => { ids.push(DS::new_user_wrap(&mut db, String::from("bob"), true, Some(3))); },
                20 => { black_box(db.get_info(ids[rng.gen_range(0..ids.len())])); },
                1 => { black_box(db.get_snapshot()); },
                2 => { black_box(db.total_premium_credits()); },
                1 => { let _ = black_box(db.reward_premium(1.2f32)); },
                20 => { let _ = black_box(db.add_credits(ids[rng.gen_range(0..ids.len())], rng.gen_range(2..100))); },
            }
        }
    })
}
