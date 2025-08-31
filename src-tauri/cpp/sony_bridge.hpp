#pragma once
#include "rust/cxx.h"
#include <cstdint>

namespace sony {
bool crsdk_init();
void crsdk_release();

bool crsdk_connect_first_usb();

bool crsdk_is_connected();

// UTF-8 шлях/директорія куди SDK складатиме знімки
bool crsdk_set_save_dir(rust::Str path);

// зробити кадр і дочекатися збереження на ПК; "" якщо помилка або таймаут
rust::String crsdk_capture_blocking(uint32_t timeout_ms);

// отримати JPEG фрейм live-view; порожній вектор — якщо кадру немає
rust::Vec<uint8_t> crsdk_liveview_frame();
}
