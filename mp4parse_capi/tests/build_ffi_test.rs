#[cfg(not(windows))]
#[test]
fn build_ffi_test() {
    use std::process::Command;

    let output = Command::new("make")
        .arg("-C")
        .arg("examples")
        .arg("check")
        .output()
        .expect("failed to execute process");

    println!("status: {}", output.status);
    println!("--- stdout ---");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("-- stderr ---");
    println!("{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}
