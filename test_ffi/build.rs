extern crate cbindgen;
extern crate cc;

use cbindgen::{Config, RenameRule};
use std::path::Path;

const CAPI_CRATE: &str = "../mp4parse_capi";

fn main() {
    println!("cargo:rerun-if-changed={}/src/lib.rs", CAPI_CRATE);

    let crate_dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(CAPI_CRATE);
    let generated_include_dir = Path::new(&std::env::var("OUT_DIR").unwrap()).join("include");
    let header_path = generated_include_dir.join("mp4parse_ffi_generated.h");

    cbindgen::generate(&crate_dir)
        .expect("Could not generate header")
        .write_to_file(header_path);

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/test.cc");

    cc::Build::new()
        .file("src/test.cc")
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .include("include")
        .include(generated_include_dir)
        .compile("libtest.a");

    #[cfg(unix)]
    let suffix = "";
    #[cfg(windows)]
    let suffix = ".dll";
    println!("cargo:rustc-link-lib=dylib=mp4parse_capi{}", suffix);

    let profile = std::env::var("PROFILE").unwrap();
    println!("cargo:rustc-link-search=native=target/{}", profile);
}
