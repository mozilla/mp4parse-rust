extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/test.cc");

    cc::Build::new()
        .file("src/test.cc")
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .include("../mp4parse_capi/include")
        .compile("libtest.a");

    #[cfg(unix)]
    let suffix = "";
    #[cfg(windows)]
    let suffix = ".dll";
    println!("cargo:rustc-link-lib=dylib=mp4parse_capi{}", suffix);

    let profile = std::env::var("PROFILE").unwrap();
    println!("cargo:rustc-link-search=native=target/{}", profile);
}
