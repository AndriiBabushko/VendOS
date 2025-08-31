#include "sony_bridge.hpp"

#include <CRSDK/CameraRemote_SDK.h>
#include <CRSDK/IDeviceCallback.h>
#include <CRSDK/ICrCameraObjectInfo.h>
#include <CRSDK/CrCommandData.h>
#include <CRSDK/CrImageDataBlock.h>
#include <CRSDK/CrDefines.h>
#include <CRSDK/CrTypes.h>

#include <atomic>
#include <mutex>
#include <condition_variable>
#include <chrono>
#include <string>
#include <vector>
#include <memory>   // <- додали

using namespace SCRSDK;

namespace {
  class DeviceCallback final : public IDeviceCallback {
  public:
    // !!! У ВИЗНАЧЕННІ ХЕДЕРІВ ТИП НАЗИВАЄТЬСЯ DeviceConnectionVersioin (з помилкою)
    void OnConnected(DeviceConnectionVersioin) override { is_connected.store(true); }
    void OnDisconnected(CrInt32u) override { is_connected.store(false); }
    void OnCompleteDownload(CrChar* filename, CrInt32u = 0xFFFFFFFF) override {
      if (!filename) return;
      std::lock_guard<std::mutex> lock(mutex_download);
      last_downloaded_path = filename;
      has_new_file = true;
      cv_download.notify_all();
    }

    // інші події нам не потрібні
    void OnPropertyChanged() override {}
    void OnLvPropertyChanged() override {}
    void OnWarning(CrInt32u) override {}
    void OnError(CrInt32u) override {}
    void OnPropertyChangedCodes(CrInt32u, CrInt32u*) override {}
    void OnLvPropertyChangedCodes(CrInt32u, CrInt32u*) override {}
    void OnNotifyContentsTransfer(CrInt32u, CrContentHandle, CrChar*) override {}

    bool wait_for_photo(unsigned timeout_ms, std::string& out_path) {
      std::unique_lock<std::mutex> lock(mutex_download);
      if (!cv_download.wait_for(lock, std::chrono::milliseconds(timeout_ms), [&]{ return has_new_file; })) {
        return false;
      }
      out_path = last_downloaded_path;
      has_new_file = false;
      return true;
    }

    std::atomic<bool> is_connected{false};

  private:
    std::mutex mutex_download;
    std::condition_variable cv_download;
    std::string last_downloaded_path;
    bool has_new_file{false};
  };

  DeviceCallback g_callback;
  CrDeviceHandle g_device = 0;

  // live-view тимчасові буфери
  std::unique_ptr<CrImageDataBlock> g_image_block;
  std::vector<uint8_t> g_tmp_buffer;
}

namespace sony {

bool crsdk_init() {
  return SCRSDK::Init(0);
}

void crsdk_release() {
  if (g_device) {
    SCRSDK::Disconnect(g_device);
    g_device = 0;
  }
  SCRSDK::Release();
}

bool crsdk_is_connected() {
  return g_callback.is_connected.load();
}

bool crsdk_connect_first_usb() {
  ICrEnumCameraObjectInfo* enumerator = nullptr;
  if (SCRSDK::EnumCameraObjects(&enumerator) != CrError_None || !enumerator) return false;

  // повертається const*
  const ICrCameraObjectInfo* camera_info_const = enumerator->GetCameraObjectInfo(0);
  if (!camera_info_const) { enumerator->Release(); return false; }

  // Connect у більшості версій приймає не-const → робимо safe const_cast
  bool ok = (SCRSDK::Connect(const_cast<ICrCameraObjectInfo*>(camera_info_const),
                             &g_callback, &g_device, CrSdkControlMode_RemoteTransfer) == CrError_None);
  enumerator->Release();
  return ok;
}

bool crsdk_set_save_dir(rust::Str path) {
  if (!g_device) return false;
  return SCRSDK::SetSaveInfo(g_device, (CrChar*)path.data(), (CrChar*)"", -1) == CrError_None;
}

rust::String crsdk_capture_blocking(uint32_t timeout_ms) {
  if (!g_device) return rust::String();

  SCRSDK::SendCommand(g_device, CrCommandId_Release, CrCommandParam_Down);
  SCRSDK::SendCommand(g_device, CrCommandId_Release, CrCommandParam_Up);

  std::string saved_path;
  if (!g_callback.wait_for_photo(timeout_ms, saved_path)) return rust::String();
  return rust::String(saved_path);
}

rust::Vec<uint8_t> crsdk_liveview_frame() {
  rust::Vec<uint8_t> result;
  if (!g_device) return result;

  CrImageInfo info;
  if (SCRSDK::GetLiveViewImageInfo(g_device, &info) != CrError_None) return result;

  if (!g_image_block) g_image_block.reset(new CrImageDataBlock());
  g_image_block->SetSize(info.GetBufferSize());

  if (g_tmp_buffer.size() < info.GetBufferSize())
    g_tmp_buffer.resize(info.GetBufferSize());

  g_image_block->SetData(g_tmp_buffer.data());

  if (SCRSDK::GetLiveViewImage(g_device, g_image_block.get()) != CrError_None) return result;

  uint32_t image_size = g_image_block->GetImageSize();
  result.reserve(image_size);
  for (uint32_t i = 0; i < image_size; ++i) {
    result.push_back(g_image_block->GetImageData()[i]);
  }
  return result; // JPEG
}

} // namespace sony
