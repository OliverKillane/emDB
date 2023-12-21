use divan::{black_box, Bencher};
use query_opt_comparison::tables::{
    foresight::ForesightDatabase as ForeDB, naive::NaiveDatabase as NaiveDB, UserDetails,
};
use rand::{seq::SliceRandom, Rng};

mod utils;
use utils::*;

const TABLE_SIZES: [usize; 4] = [4096, 16384, 65536, 262144];

fn main() {
    divan::main();
}

#[divan::bench(
    name = "Time taken for a number of inserts of random premium/non-premium",
    types = [NaiveDB, ForeDB],
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
        db.add_credits(id.clone(), rng.gen_range(2..100));
    }
    db.reward_premium(2f32);
    black_box((ids, db))
}

#[divan::bench(
    name = "Time taken to get ids in random order",
    types = [NaiveDB, ForeDB],
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
                black_box(db.get_info(*id));
            }
        })
}

#[divan::bench(
    name = "Time taken to get a snapshot",
    types = [NaiveDB, ForeDB],
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

#[divan::bench(
    name = "Time taken to get the total credits of premium users",
    types = [NaiveDB, ForeDB],
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

#[divan::bench(
    name = "Time taken to reward premium users",
    types = [NaiveDB, ForeDB],
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

#[divan::bench(
    name = "Random workload of N actions",
    types = [NaiveDB, ForeDB],
    consts = [100000]
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
                20 => { black_box(db.get_info(ids[rng.gen_range(0..ids.len())])); },
                1 => { black_box(db.get_snapshot()); },
                2 => { black_box(db.total_premium_credits()); },
                1 => { black_box(db.reward_premium(2f32)); },
                20 => { black_box(db.add_credits(ids[rng.gen_range(0..ids.len())], rng.gen_range(2..100))); },
            }
        }
    })
}
