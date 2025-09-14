#include "sony_bridge.hpp"

// Sony CRSDK (umbrella + потрібні типи)
#include <CRSDK/CameraRemote_SDK.h>
#include <CRSDK/IDeviceCallback.h>
#include <CRSDK/ICrCameraObjectInfo.h>
#include <CRSDK/CrCommandData.h>
#include <CRSDK/CrImageDataBlock.h>
#include <CRSDK/CrDefines.h>
#include <CRSDK/CrTypes.h>

// std
#include <atomic>
#include <condition_variable>
#include <chrono>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

using namespace SCRSDK;

namespace {

// --- Callback ---
class DeviceCallback final : public IDeviceCallback {
public:
    // у деяких версіях хедерів назва з опечаткою: DeviceConnectionVersioin
    void OnConnected(DeviceConnectionVersioin) override { is_connected.store(true); }
    void OnDisconnected(CrInt32u) override { is_connected.store(false); }

    void OnCompleteDownload(CrChar* filename, CrInt32u = 0xFFFFFFFF) override {
        if (!filename) return;
        std::lock_guard<std::mutex> lk(mx);
        last_downloaded_path = filename;
        has_new = true;
        cv.notify_all();
    }

    void OnPropertyChanged() override {}
    void OnLvPropertyChanged() override {}
    void OnWarning(CrInt32u) override {}
    void OnError(CrInt32u) override {}
    void OnPropertyChangedCodes(CrInt32u, CrInt32u*) override {}
    void OnLvPropertyChangedCodes(CrInt32u, CrInt32u*) override {}
    void OnNotifyContentsTransfer(CrInt32u, CrContentHandle, CrChar*) override {}

    bool wait_for_photo(unsigned timeout_ms, std::string& out) {
        std::unique_lock<std::mutex> lk(mx);
        if (!cv.wait_for(lk, std::chrono::milliseconds(timeout_ms), [&]{ return has_new; })) {
            return false;
        }
        out = last_downloaded_path;
        has_new = false;
        return true;
    }

    std::atomic<bool> is_connected{false};

private:
    std::mutex mx;
    std::condition_variable cv;
    std::string last_downloaded_path;
    bool has_new{false};
};

// --- глобальний стан ---
DeviceCallback g_cb;
CrDeviceHandle g_dev = 0;

// Тримаємо енумератор живим, щоб ICrCameraObjectInfo* лишались валідними
ICrEnumCameraObjectInfo* g_enum = nullptr;
std::vector<const ICrCameraObjectInfo*> g_list;

// Live-view буфери
std::unique_ptr<CrImageDataBlock> g_img_blk;
std::vector<std::uint8_t> g_tmp;

// наш останній код помилки (0 == CrError_None)
std::atomic<int> g_last_err{0};

} // namespace

namespace sony {

// ====== БАЗА ======
bool crsdk_init() {
    g_last_err = 0;
    bool ok = SCRSDK::Init(0);
    if (!ok) g_last_err = -1001; // Init failed (bool)
    return ok;
}

void crsdk_release() {
    if (g_dev) {
        SCRSDK::Disconnect(g_dev);
        g_dev = 0;
    }
    if (g_enum) { g_enum->Release(); g_enum = nullptr; }
    SCRSDK::Release();
    g_last_err = 0;
}

bool crsdk_is_connected() {
    return g_cb.is_connected.load();
}

// ====== ПІД’ЄДНАННЯ ======
bool crsdk_connect_first_usb() {
    g_last_err = 0;
    ICrEnumCameraObjectInfo* e = nullptr;
    CrError err = SCRSDK::EnumCameraObjects(&e);
    if (err != CrError_None || !e) { g_last_err = (int)err; return false; }

    const ICrCameraObjectInfo* info = e->GetCameraObjectInfo(0);
    bool ok = false;
    if (info) {
        err = SCRSDK::Connect(const_cast<ICrCameraObjectInfo*>(info),
                              &g_cb, &g_dev, CrSdkControlMode_RemoteTransfer);
        g_last_err = (int)err;
        ok = (err == CrError_None);
    }
    e->Release();
    return ok;
}

// --- перелік камер (простий цикл, без GetCount()) ---
bool crsdk_enum_refresh() {
    g_last_err = 0;
    if (g_enum) { g_enum->Release(); g_enum = nullptr; }
    g_list.clear();

    CrError err = SCRSDK::EnumCameraObjects(&g_enum);
    if (err != CrError_None || !g_enum) { g_last_err = (int)err; return false; }

    for (CrInt32u i = 0;; ++i) {
        const ICrCameraObjectInfo* info = g_enum->GetCameraObjectInfo(i);
        if (!info) break;
        g_list.push_back(info);
    }
    return true;
}

std::uint32_t crsdk_enum_count() {
    return static_cast<std::uint32_t>(g_list.size());
}

rust::String crsdk_enum_model(std::uint32_t i) {
    if (i >= g_list.size() || !g_list[i]) return rust::String();

    const CrChar* name = g_list[i]->GetModel();
#ifdef _WIN32
    const wchar_t* w = reinterpret_cast<const wchar_t*>(name);
    if (!w) return rust::String();
    int len = WideCharToMultiByte(CP_UTF8, 0, w, -1, nullptr, 0, nullptr, nullptr);
    if (len <= 1) return rust::String();
    std::string utf8(len - 1, '\0');
    WideCharToMultiByte(CP_UTF8, 0, w, -1, &utf8[0], len, nullptr, nullptr);
    return rust::String(utf8);
#else
    return rust::String(name ? name : "");
#endif
}

std::int32_t crsdk_connect_index(std::uint32_t i) {
    if (i >= g_list.size() || !g_list[i]) { g_last_err = -2; return -1; } // bad index
    if (g_dev) { SCRSDK::Disconnect(g_dev); g_dev = 0; }

    CrError err = SCRSDK::Connect(const_cast<ICrCameraObjectInfo*>(g_list[i]),
                                  &g_cb, &g_dev, CrSdkControlMode_RemoteTransfer);
    g_last_err = (int)err;
    return (int)err; // 0 == CrError_None
}

// ====== НАЛАШТУВАННЯ/КАДРИ ======
bool crsdk_set_save_dir(rust::Str path_utf8) {
    if (!g_dev) { g_last_err = -3; return false; } // not connected
    CrError err = SCRSDK::SetSaveInfo(g_dev,
                                      (CrChar*)path_utf8.data(),
                                      (CrChar*)"", -1);
    g_last_err = (int)err;
    return (err == CrError_None);
}

rust::String crsdk_capture_blocking(std::uint32_t timeout_ms) {
    if (!g_dev) { g_last_err = -3; return rust::String(); } // not connected

    SCRSDK::SendCommand(g_dev, CrCommandId_Release, CrCommandParam_Down);
    SCRSDK::SendCommand(g_dev, CrCommandId_Release, CrCommandParam_Up);

    std::string saved;
    if (!g_cb.wait_for_photo(timeout_ms, saved)) { g_last_err = -110; return rust::String(); }
    g_last_err = 0;
    return rust::String(saved);
}

rust::Vec<std::uint8_t> crsdk_liveview_frame() {
    rust::Vec<std::uint8_t> out;
    if (!g_dev) { g_last_err = -3; return out; } // not connected

    CrImageInfo info;
    CrError err = SCRSDK::GetLiveViewImageInfo(g_dev, &info);
    if (err != CrError_None) { g_last_err = (int)err; return out; }

    if (!g_img_blk) g_img_blk.reset(new CrImageDataBlock());
    auto size = info.GetBufferSize();
    g_img_blk->SetSize(size);

    if (g_tmp.size() < size) g_tmp.resize(size);
    g_img_blk->SetData(g_tmp.data());

    err = SCRSDK::GetLiveViewImage(g_dev, g_img_blk.get());
    if (err != CrError_None) { g_last_err = (int)err; return out; }

    std::uint32_t img_sz = g_img_blk->GetImageSize();
    out.reserve(img_sz);
    const CrInt8u* p = g_img_blk->GetImageData();
    for (std::uint32_t i = 0; i < img_sz; ++i) out.push_back(p[i]);
    g_last_err = 0;
    return out;
}

// ====== ДІАГНОСТИКА ======
std::int32_t crsdk_last_error() {
    return g_last_err.load();
}

static std::string last_error_text_impl(int code) {
    if (code == 0) return "CrError_None";
    switch (code) {
        case -1001: return "Init failed";
        case -2:    return "Invalid device index";
        case -3:    return "Not connected";
        case -110:  return "Capture timeout";
        default:    break;
    }
    return std::string("CrError code ") + std::to_string(code);
}

rust::String crsdk_last_error_text() {
    return rust::String(last_error_text_impl(g_last_err.load()));
}

std::uint32_t crsdk_version_raw() {
    // CRSDK повертає номер версії як ціле (наприклад 11400 == 1.14.00)
    return SCRSDK::GetSDKVersion();
}


} // namespace sony
