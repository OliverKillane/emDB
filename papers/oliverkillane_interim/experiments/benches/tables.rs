use divan::{black_box, Bencher};
use experiments::tables::{DuckDBDatabase as DuckDB, SQLiteDatabase as SQLite};
use experiments::tables::{ForesightDatabase as Foresight, NaiveDatabase as Naive, UserDetails};

use rand::{seq::SliceRandom, Rng};

mod utils;
use utils::*;

// const TABLE_SIZES: [usize; 9] = [1, 8, 64, 128, 512, 4096, 16384, 65536, 262144];
const TABLE_SIZES: [usize; 3] = [1, 8, 16];

fn main() {
    divan::main();
}

/// Time taken for a number of inserts of random premium/non-premium
#[divan::bench(
    name = "random_inserts",
    types = [Foresight, Naive, DuckDB, SQLite],
    consts = TABLE_SIZES
)]
fn inserts<'a, T, const N: usize>(bencher: Bencher)
where
    T: UserDetails<'a>,
{
    bencher
        .with_inputs(|| {
            let db = T::new();
            let mut rng = rand::thread_rng();

            (
                (0..N)
                    .map(|i| (format!("User{}", i), rng.gen_bool(0.5)))
                    .collect::<Vec<_>>(),
                db,
            )
        })
        .bench_local_values(|(users, mut db)| {
            for (name, prem) in users {
                black_box(db.new_user(name, prem));
            }
        })
}

fn random_table<'a, const SIZE: usize, T: UserDetails<'a>>() -> (Vec<T::UsersID>, T) {
    let mut db = T::new();
    let mut rng = rand::thread_rng();

    let mut ids = (0..SIZE)
        .map(|i| {
            let prem = rng.gen_bool(0.5);
            let name = format!("User{}", i);
            let id = db.new_user(name, prem);
            id
        })
        .collect::<Vec<_>>();
    ids.shuffle(&mut rng);

    for id in ids.iter() {
        db.add_credits(id.clone(), rng.gen_range(2..100)).unwrap();
    }
    db.reward_premium(2f32).unwrap();
    black_box((ids, db))
}

/// Time taken to get ids in random order
#[divan::bench(
    name = "random_get_ids",
    types = [Foresight, Naive, SQLite],
    consts = TABLE_SIZES
)]
fn gets<'a, T, const N: usize>(bencher: Bencher)
where
    T: UserDetails<'a>,
{
    bencher
        .with_inputs(random_table::<N, T>)
        .bench_local_refs(|(ids, db)| {
            for id in ids {
                black_box(db.get_info(*id)).unwrap();
            }
        })
}

/// Time taken to get a snapshot
#[divan::bench(
    name = "snapshot",
    types = [Foresight, Naive, DuckDB, SQLite],
    consts = TABLE_SIZES
)]
fn snapshot<'a, T, const N: usize>(bencher: Bencher)
where
    T: UserDetails<'a>,
{
    bencher
        .with_inputs(random_table::<N, T>)
        .bench_local_refs(|(_, db)| black_box(db.get_snapshot()))
}

/// Time taken to get the total credits of premium users
#[divan::bench(
    name = "get_total_prem_credits",
    types = [Foresight, Naive, SQLite],
    consts = TABLE_SIZES,
    max_time = 1
)]
fn premium_credits<'a, T, const N: usize>(bencher: Bencher)
where
    T: UserDetails<'a>,
{
    bencher
        .with_inputs(random_table::<N, T>)
        .bench_local_refs(|(_, db)| black_box(db.total_premium_credits()))
}

/// Time taken to reward premium users
#[divan::bench(
    name = "reward_premium_users",
    types = [Foresight, Naive, SQLite],
    consts = TABLE_SIZES,
    max_time = 1
)]
fn reward_premium<'a, T, const N: usize>(bencher: Bencher)
where
    T: UserDetails<'a>,
{
    bencher
        .with_inputs(random_table::<N, T>)
        .bench_local_refs(|(_, db)| black_box(db.reward_premium(2f32)))
}

/// Random workload of N actions
#[divan::bench(
    name = "random_workloads",
    types = [Foresight, Naive, SQLite],
    consts = [1024, 2048, 4096]
)]
fn mixed_workload<'a, T, const N: usize>(bencher: Bencher)
where
    T: UserDetails<'a>,
{
    bencher.bench_local(|| {
        let mut db = T::new();
        let mut rng = rand::thread_rng();

        // avoid reallocations
        let mut ids = Vec::with_capacity(N);
        ids.push(db.new_user(String::from("bob"), true));

        for _ in 0..N {
            choose! { rng
                10 => { ids.push(db.new_user(String::from("bob"), true)); },
                20 => { black_box(db.get_info(ids[rng.gen_range(0..ids.len())])).unwrap(); },
                1 => { black_box(db.get_snapshot()); },
                2 => { black_box(db.total_premium_credits()); },
                1 => { let _ = black_box(db.reward_premium(1.2f32)); },
                20 => { let _ = black_box(db.add_credits(ids[rng.gen_range(0..ids.len())], rng.gen_range(2..100))); },
            }
        }
    })
}
