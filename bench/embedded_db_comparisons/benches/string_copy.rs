use divan;
use embedded_db_comparisons::string_copy::{
    copy_string::{self as interface, Database},
    emdb_copy_impl::EmDBCopy,
    emdb_copy_ignore_impl::EmDBCopyIgnore,
    emdb_ref_ignore_impl::EmDBRefIgnore,
    emdb_ref_impl::EmDBRef,
    populate_database,
};

const STR_LEN_SIZES: [usize; 7] = [512, 1024, 2048, 4096, 8192, 16384, 32768];
const DB_SIZE: usize = 4096;

#[divan::bench(
    name = "count values",
    types = [EmDBRef, EmDBCopy, EmDBCopyIgnore, EmDBRefIgnore],
    consts = STR_LEN_SIZES,
)]
fn count_values<DS: interface::Datastore, const STR_LEN: usize>(bencher: divan::Bencher) {
    bencher
        .with_inputs(|| divan::black_box(populate_database(DB_SIZE, STR_LEN)))
        .bench_local_values(|mut ds: DS| {
            {
                let db = ds.db();
                db.count_values();
            }
            ds
        })
}
fn main() {
    divan::main()
}
