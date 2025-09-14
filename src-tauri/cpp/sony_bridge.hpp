#pragma once
#include "rust/cxx.h"
#include <cstdint>

namespace sony {

// Ініціалізація / завершення
bool   crsdk_init();
void   crsdk_release();
bool   crsdk_is_connected();

// Під’єднання
bool   crsdk_connect_first_usb();

// Перелік камер і під’єднання за індексом
bool            crsdk_enum_refresh();    // пере-сканувати пристрої
std::uint32_t   crsdk_enum_count();
rust::String    crsdk_enum_model(std::uint32_t i);
std::int32_t    crsdk_connect_index(std::uint32_t i); // 0 == CrError_None

// Збереження/кадри
bool                    crsdk_set_save_dir(rust::Str path_utf8);
rust::String            crsdk_capture_blocking(std::uint32_t timeout_ms);
rust::Vec<std::uint8_t> crsdk_liveview_frame(); // JPEG

// Діагностика
std::int32_t   crsdk_last_error();
rust::String   crsdk_last_error_text();

std::uint32_t crsdk_version_raw();

} // namespace sony
