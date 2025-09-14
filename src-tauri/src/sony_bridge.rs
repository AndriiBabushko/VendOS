#![allow(non_snake_case)]

#[cxx::bridge]
mod ffi {
    // include без "cpp/" — build.rs додає include("cpp")
    unsafe extern "C++" {
        include!("sony_bridge.hpp");

        // базове
        #[namespace = "sony"] fn crsdk_init() -> bool;
        #[namespace = "sony"] fn crsdk_release();
        #[namespace = "sony"] fn crsdk_is_connected() -> bool;

        // під’єднання / енумерація
        #[namespace = "sony"] fn crsdk_connect_first_usb() -> bool;
        #[namespace = "sony"] fn crsdk_enum_refresh() -> bool;
        #[namespace = "sony"] fn crsdk_enum_count() -> u32;
        #[namespace = "sony"] fn crsdk_enum_model(i: u32) -> String;
        #[namespace = "sony"] fn crsdk_connect_index(i: u32) -> i32;

        // збереження / кадри
        #[namespace = "sony"] fn crsdk_set_save_dir(path: &str) -> bool;
        #[namespace = "sony"] fn crsdk_capture_blocking(timeout_ms: u32) -> String;
        #[namespace = "sony"] fn crsdk_liveview_frame() -> Vec<u8>;

        // діагностика
        #[namespace = "sony"] fn crsdk_last_error() -> i32;
        #[namespace = "sony"] fn crsdk_last_error_text() -> String;

        #[namespace = "sony"] fn crsdk_version_raw() -> u32;
    }
}

pub use self::ffi::*;
