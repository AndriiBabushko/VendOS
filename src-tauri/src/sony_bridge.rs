#[cxx::bridge]
mod ffi {
    // УВАГА: шлях відносно кореня crate (src-tauri/)
    unsafe extern "C++" {
        include!("cpp/sony_bridge.hpp");

        #[namespace = "sony"]
        fn crsdk_init() -> bool;

        #[namespace = "sony"]
        fn crsdk_release();

        #[namespace = "sony"]
        fn crsdk_connect_first_usb() -> bool;

        // cxx підтримує &str як read-only rust::Str на боці C++
        #[namespace = "sony"]
        fn crsdk_set_save_dir(path: &str) -> bool;

        /// Робить кадр і повертає абсолютний шлях до збереженого файлу.
        /// Порожній рядок = помилка/таймаут.
        #[namespace = "sony"]
        fn crsdk_capture_blocking(timeout_ms: u32) -> String;

        /// JPEG кадр live-view (порожній — якщо нема)
        #[namespace = "sony"]
        fn crsdk_liveview_frame() -> Vec<u8>;

        #[namespace = "sony"]
        fn crsdk_is_connected() -> bool;
    }
}

pub use self::ffi::*;
