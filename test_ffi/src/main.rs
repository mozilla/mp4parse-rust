use std::convert::TryInto as _;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

fn main() {
    #[link(name = "test", kind = "static")]
    extern "C" {
        fn run_main(npaths: c_int, paths: *const *const c_char) -> c_int;
    }

    let args: Vec<_> = std::env::args()
        .map(|arg| CString::new(arg).unwrap())
        .collect();
    let argv = {
        let mut a: Vec<_> = args.iter().map(|arg| arg.as_ptr()).collect();
        a.push(std::ptr::null());
        a
    };

    let argc = argv.len() - 1;
    unsafe {
        assert_eq!(run_main(argc.try_into().unwrap(), argv.as_ptr()), 0);
    }
}

#[test]
fn ffi_test() {
    use std::path::PathBuf;

    extern "C" {
        fn test_main(test_path: *const c_char) -> c_int;
    }

    const TEST_PATH: &str = "../mp4parse/tests/minimal.mp4";

    let path = {
        let base = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(TEST_PATH);
        let path = base.canonicalize().unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt as _;
            CString::new(path.as_os_str().as_bytes()).unwrap()
        }
        #[cfg(windows)]
        {
            CString::new(path.to_str().unwrap()).unwrap()
        }
    };

    unsafe {
        assert_eq!(test_main(path.as_ptr()), 0);
    }
}
