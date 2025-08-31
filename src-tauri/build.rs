use std::{env, fs, path::PathBuf};

fn main() {
    // обов'язковий крок Tauri
    tauri_build::build();

    // генеруємо та збираємо CXX-міст
    let mut bridge = cxx_build::bridge("src/sony_bridge.rs");
    bridge
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unknown-pragmas")
        .include(".")
        .include("libs/include")
        .file("cpp/sony_bridge.cc");
    bridge.compile("sony_bridge");


    // лінкування CRSDK
    println!("cargo:rustc-link-search=native=libs/macos");
    println!("cargo:rustc-link-lib=dylib=Cr_Core");
    println!("cargo:rustc-link-lib=dylib=monitor_protocol");
    println!("cargo:rustc-link-lib=dylib=monitor_protocol_pf");
    println!("cargo:rustc-link-lib=dylib=c++"); // libc++ на macOS

    // скопіювати .dylib поруч із бінарником для dev
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bin_dir = out_dir.join("../../../");
    for name in ["libCr_Core.dylib","libmonitor_protocol.dylib","libmonitor_protocol_pf.dylib"] {
        let _ = fs::copy(PathBuf::from("libs/macos").join(name), bin_dir.join(name));
    }
}
