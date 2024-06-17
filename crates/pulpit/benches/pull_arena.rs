/// Compare the performance of generational arenas
/// Here we compare insert, followed by sequential access.
use divan;
use pulpit::column::{
    Column, Data, Entry, PrimaryGenerationalArena, PrimaryRetain, PrimaryThunderDome,
    PrimaryWindow, PrimaryWindowPull,
};

const STRING_LEN: usize = 128;
const SCALE_FACTORS: [usize; 5] = [1024, 2048, 4096, 8192, 16384];

/// Sequential insert & access. assumes the user needs to get a value lasting longer than a borrow.   
fn workload<ImmData, MutData, Col>(to_insert: Vec<Data<ImmData, MutData>>)
where
    Col: Column,
    MutData: Clone,
    for<'a> Col::WindowKind<'a>: PrimaryWindowPull<'a, ImmData, MutData>,
{
    let mut col = Col::new(to_insert.len());
    let mut vals = Vec::with_capacity(to_insert.len());
    let mut win = col.window();

    for val in to_insert {
        let (key, _) = win.insert(val);
        let Entry { index: _, data } = win.get(key).unwrap();
        vals.push(data.extract());
    }

    divan::black_box_drop(win);
    divan::black_box_drop(vals);
    divan::black_box_drop(col);
}

/// Sequential insert & access, measures only the borrow access.
fn borrow_only_workload<ImmData, MutData, Col>(to_insert: Vec<Data<ImmData, MutData>>)
where
    Col: Column,
    for<'a> Col::WindowKind<'a>: PrimaryWindowPull<'a, ImmData, MutData>,
{
    let mut col = Col::new(to_insert.len());
    let mut win = col.window();

    for val in to_insert {
        let (key, _) = win.insert(val);
        divan::black_box_drop(win.brw(key));
    }

    divan::black_box_drop(win);
    divan::black_box_drop(col);
}

#[divan::bench(
    name="Get (String, mut usize)",
    types=[
        PrimaryGenerationalArena<String, usize>,
        PrimaryThunderDome<String, usize>,
        PrimaryRetain<String, usize, 1024>
    ],
    consts=SCALE_FACTORS,
)]
fn bench_workload<Col, const ELEMENTS: usize>(bencher: divan::Bencher)
where
    Col: Column,
    for<'a> Col::WindowKind<'a>: PrimaryWindowPull<'a, String, usize>,
{
    bencher
        .counter(divan::counter::ItemsCount::new(ELEMENTS))
        .with_inputs(|| {
            let x: Vec<Data<String, usize>> = (0..ELEMENTS)
                .map(|i| Data {
                    imm_data: "a".repeat(STRING_LEN),
                    mut_data: i,
                })
                .collect();
            x
        })
        .bench_values(|v: Vec<Data<String, usize>>| workload::<String, usize, Col>(v))
}

#[divan::bench(
    name="Brw (String, mut usize)",
    types=[
        PrimaryGenerationalArena<String, usize>,
        PrimaryThunderDome<String, usize>,
        PrimaryRetain<String, usize, 1024>
        ],
        consts=SCALE_FACTORS,
    )]
fn bench_workload_brw<Col, const ELEMENTS: usize>(bencher: divan::Bencher)
where
    Col: Column,
    for<'a> Col::WindowKind<'a>: PrimaryWindowPull<'a, String, usize>,
{
    bencher
        .counter(divan::counter::ItemsCount::new(ELEMENTS))
        .with_inputs(|| {
            (0..ELEMENTS)
                .map(|i| Data {
                    imm_data: "a".repeat(STRING_LEN),
                    mut_data: i,
                })
                .collect()
        })
        .bench_values(|v| borrow_only_workload::<String, usize, Col>(v))
}

#[divan::bench(
    name="Workload (usize, mut usize)",
    types=[
        PrimaryGenerationalArena<usize, usize>,
        PrimaryThunderDome<usize, usize>,
        PrimaryRetain<usize, usize, 1024>
    ],
    consts=SCALE_FACTORS,
)]
fn bench_workload_no_imm<Col, const ELEMENTS: usize >(bencher: divan::Bencher)
where
    Col: Column,
    for<'a> Col::WindowKind<'a>: PrimaryWindowPull<'a, usize, usize>,
{
    bencher
        .counter(divan::counter::ItemsCount::new(ELEMENTS))
        .with_inputs(|| {
            (0..ELEMENTS)
                .map(|i| Data {
                    imm_data: i,
                    mut_data: i,
                })
                .collect()
        })
        .bench_values(|v| workload::<usize, usize, Col>(v))
}

#[divan::bench(
    name="Workload ((), mut ())",
    types=[
        PrimaryGenerationalArena<(), ()>,
        PrimaryThunderDome<(), ()>,
        PrimaryRetain<(), (), 1024>
    ],
    consts=SCALE_FACTORS,
)]
fn bench_workload_zero_size<Col, const ELEMENTS: usize>(bencher: divan::Bencher)
where
    Col: Column,
    for<'a> Col::WindowKind<'a>: PrimaryWindowPull<'a, (), ()>,
{
    bencher
        .counter(divan::counter::ItemsCount::new(ELEMENTS))
        .with_inputs(|| {
            (0..ELEMENTS)
                .map(|_| Data {
                    imm_data: (),
                    mut_data: (),
                })
                .collect()
        })
        .bench_values(|v| workload::<(), (), Col>(v))
}

fn main() {
    divan::main()
}
