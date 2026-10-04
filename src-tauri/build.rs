use std::process::Command;

fn main() {
    // Compile block_hidraw.so — blocks /dev/hidraw* access so winebus
    // uses SDL (which supports rumble) instead of hidraw (which doesn't).
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let status = Command::new("gcc")
        .args(["-shared", "-fPIC", "-O2",
               "-o", &format!("{out_dir}/block_hidraw.so"),
               "lib/block_hidraw.c", "-ldl"])
        .status()
        .expect("failed to compile block_hidraw.so");
    assert!(status.success(), "gcc failed to compile block_hidraw.so");
    println!("cargo:rerun-if-changed=lib/block_hidraw.c");

    tauri_build::build()
}
