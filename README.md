# VendOS

A little C/SDL2/LVGL application that talks to Sony Camera Remote SDK and shows a 3-screen UI.

---

## 1. Directory layout

```text
.
├── third_party/
│   └── sony_sdk/
│       ├── CrSDK_v1-2_Mac
│       ├── CrSDK_v1-3_Linux 32 bit ARMv7
│       ├── CrSDK_v1-4_Linux 64 bit ARMv8
│       ├── CrSDK_v1-5_Linux 64bit x86
│       └── CrSDK_v1_Windows
├── src/
│   ├── main.c
│   └── screens/
│       ├── screen1.c/.h
│       ├── screen2.c/.h
│       └── screen3.c/.h
├── ui/                   ← (if you have XML or other assets)
│   └── screens/
├── CMakeLists.txt
├── Makefile
└── README.md
```

Before you build anything, copy the SDK folders into: `third_party/sony_sdk/` 
so that, for example, on macOS you end up with: `third_party/sony_sdk/CrSDK_v1-2_Mac/`

## 2. Install dependencies

### Linux (Debian/Ubuntu)
```bash
sudo apt update
sudo apt install \
  build-essential \
  cmake \
  pkg-config \
  libsdl2-dev \
  clang-format
```

### macOS
```bash
brew update
brew install \
  cmake \
  pkg-config \
  sdl2 \
  llvm
```

### Windows
1. Visual Studio 2022 (Desktop development with C++ workload).
2. CMake (install via the MSI from cmake.org).
3. SDL2 development libs:
   - Option A: [vcpkg]
   ```powershell
   git clone https://github.com/microsoft/vcpkg.git
    .\vcpkg\bootstrap-vcpkg.bat
    .\vcpkg\vcpkg integrate install
    .\vcpkg\vcpkg install sdl2
   ```
   - Option B: download the SDL2 Devel Bundle and unpack it; point CMake to its include/lib dirs.
4. (Optional) clang-format: install via Chocolatey or VS installer.

## 3. Build & run

We’ve provided a simple Makefile with these targets:

```bash
make format   # run clang-format on all .c/.h under src/
make clean    # remove the build artifacts
make run      # shortcut: build (if needed) + ./vending_app
```

If you prefer CMake+Ninja by hand:

```bash
mkdir build && cd build
cmake -G Ninja ..
ninja
./vending_app
```

## 4. What’s inside

- `src/main.c`

Initializes LVGL, SDL2, loads three screens, and handles the main loop.

- `src/screens/*.c/.h`

Each screen module creates its own LVGL objects and hooks up the “Next” button handler.

- `Makefile`

Wraps CMake for easy make all, plus a make format rule for code style.
