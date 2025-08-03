BUILD_DIR := cmake-build-debug
CMAKE        := cmake
BUILD_TOOL   := ninja
CLANG_FORMAT := clang-format

SRC_FILES := $(shell find src -type f \( -name "*.c" -o -name "*.h" \))

.PHONY: build format run clean

build:
	@echo "==> Creating build directory and run CMake+Ninja"
	@mkdir -p $(BUILD_DIR)
	@cd $(BUILD_DIR) && $(CMAKE) -G Ninja .. && $(CMAKE) --build .

format:
	@echo "==> Formatting C/C++ files"
	@$(CLANG_FORMAT) -i --style=file $(SRC_FILES)

run: build
	@echo "==> Running vending_app"
	@$(BUILD_DIR)/vending_app

clean:
	@echo "==> Deleting build directory"
	@rm -rf $(BUILD_DIR)
