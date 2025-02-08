use rustc_version::{version_meta, Channel};

// NOTE: This is used to activate the 'nightly' feature described in the Cargo.toml file, as we want
//       to support using the on_unimplemented messages, proc_macro_error2 similarly checks for its
//       own nightly feature
fn main() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}
