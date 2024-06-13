//! Rustc segfault originating from pulpit's simple macro
//! ```text
//! failures:
//! 
//! ---- pulpit/src/lib.rs - (line 33) stdout ----
//! error: rustc interrupted by SIGSEGV, printing backtrace
//! 
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x3163606)[0x7f97f3763606]
//! /lib/x86_64-linux-gnu/libc.so.6(+0x42520)[0x7f97f02ed520]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x169361)[0x7f97d2d3e361]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x152951)[0x7f97d2d27951]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x154366)[0x7f97d2d29366]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x1548cc)[0x7f97d2d298cc]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x1549cb)[0x7f97d2d299cb]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x1547f9)[0x7f97d2d297f9]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x1522fd)[0x7f97d2d272fd]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x153239)[0x7f97d2d28239]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x150628)[0x7f97d2d25628]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x15063e)[0x7f97d2d2563e]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x15214e)[0x7f97d2d2714e]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x152106)[0x7f97d2d27106]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x152291)[0x7f97d2d27291]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x151f61)[0x7f97d2d26f61]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x154390)[0x7f97d2d29390]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x1548fc)[0x7f97d2d298fc]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x1549cb)[0x7f97d2d299cb]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x154882)[0x7f97d2d29882]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x151b4e)[0x7f97d2d26b4e]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x15212e)[0x7f97d2d2712e]
//! /home/oliverkillane/files/emDB/crates/target/debug/deps/libpulpit_macro-1c3f2ed391cceef1.so(+0x153fa4)[0x7f97d2d28fa4]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x4f19977)[0x7f97f5519977]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x4f1960e)[0x7f97f551960e]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(_RNvXs_NtCs5jjoAhuLzIs_12rustc_expand10proc_macroNtB4_13BangProcMacroNtNtB6_4base13BangProcMacro6expand+0x88)[0x7f97f5cecbca]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(_RNvMs1_NtCs5jjoAhuLzIs_12rustc_expand6expandNtB5_13MacroExpander21fully_expand_fragment+0x367d8)[0x7f97f1614f98]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(_RNvMs1_NtCs5jjoAhuLzIs_12rustc_expand6expandNtB5_13MacroExpander12expand_crate+0x298)[0x7f97f59b9918]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x4b3354a)[0x7f97f513354a]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x4b32b0d)[0x7f97f5132b0d]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x4b32ae7)[0x7f97f5132ae7]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x5413a85)[0x7f97f5a13a85]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x54137ad)[0x7f97f5a137ad]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x527eed0)[0x7f97f587eed0]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x52400c9)[0x7f97f58400c9]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/librustc_driver-4f8cb1b2fa29bc89.so(+0x523fe86)[0x7f97f583fe86]
//! /home/oliverkillane/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/libstd-45288dcc88911a1f.so(rust_metadata_std_69052015ce1b7124+0xc3bfb)[0x7f97f0597bfb]
//! /lib/x86_64-linux-gnu/libc.so.6(+0x94ac3)[0x7f97f033fac3]
//! /lib/x86_64-linux-gnu/libc.so.6(+0x126850)[0x7f97f03d1850]
//! 
//! note: we would appreciate a report at https://github.com/rust-lang/rust
//! help: you can increase rustc's stack size by setting RUST_MIN_STACK=16777216
//! note: backtrace dumped due to SIGSEGV! resuming signal
//! Couldn't compile the test.
//! ```


#[allow(dead_code)]
#[derive(Clone)]
enum RGB {
    Red,
    Green,
    Blue,
}

// emdb::dependencies::pulpit::macros::simple! {
//     fields {
//         name: String,
//         id: usize @ unique(unique_reference_number),
//         age: u8,
//         fav_rgb_colour: crate::RGB,
//     },
//     updates {
//         update_age: [age],
//     },
//     gets {
//         get_all: [name, id, age, fav_rgb_colour],
//     },
//     predicates {
//         adults_only: *age > 18,
//         age_cap: *age < 100,
//     },
//     limit {
//         cool_limit: 2000
//     },
//     transactions: on,
//     deletions: on,
//     name: bowling_club
// }
