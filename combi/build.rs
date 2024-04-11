use rustc_version::{version_meta, Channel};

// NOTE: This is used to activate the 'nightly' feature described in the Cargo.toml file, as we want 
//       to support using the diagnostics API on nightly
fn main() {
    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}
