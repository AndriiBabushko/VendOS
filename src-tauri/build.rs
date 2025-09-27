fn main() {
    cxx_build::bridge("src/sony_bridge.rs")
        .file("cpp/sony_bridge.cc")
        .flag_if_supported("-std=c++17")
        .include("libs/include")
        .include("cpp")
        .compile("sony_bridge");

    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "x86_64".into());
    let dir = match arch.as_str() {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "arm" => "armv7",
        _ => "x86_64",
    };

    println!("cargo:rustc-link-search=native=libs/linux/{dir}");
    println!("cargo:rustc-link-search=native=libs/linux/{dir}/CrAdapter");

    println!("cargo:rustc-link-lib=dylib=Cr_Core");
    println!("cargo:rustc-link-lib=dylib=usb-1.0");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // Щоб бінарник знаходив вбудовані .so відносно себе:
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../libs/linux/{dir}");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../../libs/linux/{dir}/CrAdapter");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");


    println!("cargo:rerun-if-changed=src/sony_bridge.rs");
    println!("cargo:rerun-if-changed=cpp/sony_bridge.cc");
    println!("cargo:rerun-if-changed=cpp/sony_bridge.hpp");
    println!("cargo:rerun-if-changed=libs/include/CRSDK");
}
