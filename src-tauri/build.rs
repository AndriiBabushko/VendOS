use std::{env, fs, path::PathBuf};

fn main() {
    tauri_build::build();

    let target_os   = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    // CXX міст
    let mut bridge = cxx_build::bridge("src/sony_bridge.rs");
    if target_os == "windows" {
        bridge.flag_if_supported("/std:c++17")
            .flag_if_supported("/Zc:__cplusplus")
            .flag_if_supported("/W3");
    } else {
        bridge.flag_if_supported("-std=c++17")
            .flag_if_supported("-Wno-unused-parameter")
            .flag_if_supported("-Wno-unknown-pragmas");
    }
    bridge
        .include(".")
        .include("libs/include")
        .file("cpp/sony_bridge.cc");
    bridge.compile("sony_bridge");

    // Куди копіювати динаміки для dev
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bin_dir = out_dir.join("../../../"); // target/{debug|release}

    match (target_os.as_str(), target_arch.as_str()) {
        // --- macOS ---
        ("macos", _) => {
            println!("cargo:rustc-link-search=native=libs/macos");
            println!("cargo:rustc-link-lib=dylib=Cr_Core");
            println!("cargo:rustc-link-lib=dylib=monitor_protocol");
            println!("cargo:rustc-link-lib=dylib=monitor_protocol_pf");
            println!("cargo:rustc-link-lib=dylib=c++");

            for name in ["libCr_Core.dylib","libmonitor_protocol.dylib","libmonitor_protocol_pf.dylib"] {
                let _ = fs::copy(PathBuf::from("libs/macos").join(name), bin_dir.join(name));
            }
        }

        // --- Windows x64 ---
        ("windows", "x86_64") => {
            println!("cargo:rustc-link-search=native=libs/windows/x64");
            // Потрібен лише цей .lib:
            println!("cargo:rustc-link-lib=dylib=Cr_Core");

            // Рантайм DLL, які мусять лежати біля .exe:
            for name in [
                "Cr_Core.dll",
                "monitor_protocol.dll",
                "monitor_protocol_pf.dll",
                "Cr_PTP_USB.dll",
                "Cr_PTP_IP.dll",
                "libusb-1.0.dll",
                "libssh2.dll",
            ] {
                let _ = fs::copy(PathBuf::from("libs/windows/x64").join(name), bin_dir.join(name));
            }
        }

        // --- Linux x86_64 ---
        ("linux", "x86_64") => {
            println!("cargo:rustc-link-search=native=libs/linux/x86_64");
            println!("cargo:rustc-link-lib=dylib=Cr_Core");
            println!("cargo:rustc-link-lib=dylib=monitor_protocol");
            println!("cargo:rustc-link-lib=dylib=monitor_protocol_pf");
            println!("cargo:rustc-link-lib=dylib=stdc++");

            for name in ["libCr_Core.so","libmonitor_protocol.so","libmonitor_protocol_pf.so"] {
                let _ = fs::copy(PathBuf::from("libs/linux/x86_64").join(name), bin_dir.join(name));
            }
        }

        // --- Linux aarch64 (ARMv8) ---
        ("linux", "aarch64") => {
            println!("cargo:rustc-link-search=native=libs/linux/aarch64");
            println!("cargo:rustc-link-lib=dylib=Cr_Core");
            println!("cargo:rustc-link-lib=dylib=monitor_protocol");
            println!("cargo:rustc-link-lib=dylib=monitor_protocol_pf");
            println!("cargo:rustc-link-lib=dylib=stdc++");

            for name in ["libCr_Core.so","libmonitor_protocol.so","libmonitor_protocol_pf.so"] {
                let _ = fs::copy(PathBuf::from("libs/linux/aarch64").join(name), bin_dir.join(name));
            }
        }

        // --- Linux ARMv7 (arm) ---
        ("linux", "arm") => {
            println!("cargo:rustc-link-search=native=libs/linux/armv7");
            println!("cargo:rustc-link-lib=dylib=Cr_Core");
            println!("cargo:rustc-link-lib=dylib=monitor_protocol");
            println!("cargo:rustc-link-lib=dylib=monitor_protocol_pf");
            println!("cargo:rustc-link-lib=dylib=stdc++");

            for name in ["libCr_Core.so","libmonitor_protocol.so","libmonitor_protocol_pf.so"] {
                let _ = fs::copy(PathBuf::from("libs/linux/armv7").join(name), bin_dir.join(name));
            }
        }

        _ => {}
    }
}
