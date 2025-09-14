fn main() {
    // rebuild triggers
    println!("cargo:rerun-if-changed=src/sony_bridge.rs");
    println!("cargo:rerun-if-changed=cpp/sony_bridge.cc");
    println!("cargo:rerun-if-changed=cpp/sony_bridge.hpp");
    println!("cargo:rerun-if-changed=libs/include");
    println!("cargo:rerun-if-changed=libs/macos");

    // cxx bridge
    let mut b = cxx_build::bridge("src/sony_bridge.rs");
    b.file("cpp/sony_bridge.cc")
        .include("cpp")
        .include("libs/include")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unknown-pragmas");

    #[cfg(target_os = "macos")]
    {
        // мінімалка системи (звично ок)
        b.flag_if_supported("-mmacosx-version-min=11.0");
    }

    b.compile("vend-os-cxxbridge");

    // ===== dev-шляхи до CRSDK (пробросимо в код як константи) =====
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dev_libs = format!("{}/libs/macos", manifest_dir);
    let dev_adapter = format!("{}/libs/macos/CrAdapter", manifest_dir);

    println!("cargo:rustc-env=CRSDK_DEV_LIBS={dev_libs}");
    println!("cargo:rustc-env=CRSDK_DEV_ADAPTER={dev_adapter}");

    // !!! для C++ макросів треба &str і одразу в лапках:
    let dev_libs_macro = format!(r#""{}""#, dev_libs);
    let dev_adapter_macro = format!(r#""{}""#, dev_adapter);

    b.define("CRSDK_DEV_LIBS",     Some(dev_libs_macro.as_str()));
    b.define("CRSDK_DEV_ADAPTER",  Some(dev_adapter_macro.as_str()));

    // ===== link CRSDK (macOS) =====
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=native=libs/macos");
        println!("cargo:rustc-link-search=native=libs/macos/CrAdapter");

        println!("cargo:rustc-link-lib=dylib=Cr_Core");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../..");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
    }
}
