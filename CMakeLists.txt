cmake_minimum_required(VERSION 3.14)
include(FetchContent)
project(VendOS_UI LANGUAGES C CXX)

set(LV_CONF_PATH
        ${CMAKE_CURRENT_SOURCE_DIR}/lv_conf.h
        CACHE FILEPATH "Path to lv_conf.h" FORCE
)

find_package(SDL2 REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(SDL2 REQUIRED sdl2)
include_directories(${SDL2_INCLUDE_DIRS})
link_directories(${SDL2_LIBRARY_DIRS})

FetchContent_Declare(
        lvgl
        GIT_REPOSITORY https://github.com/lvgl/lvgl.git
        GIT_TAG        v9.3.0
)
FetchContent_MakeAvailable(lvgl)


add_executable(vending_app src/main.c)

#add_library(sony_sdk STATIC
#        third_party/sony_sdk/src/foo.c
#)
#target_include_directories(sony_sdk PUBLIC
#        ${CMAKE_SOURCE_DIR}/third_party/sony_sdk/include
#)

target_link_libraries(vending_app
        PRIVATE
        lvgl::lvgl
        SDL2::SDL2
)
target_include_directories(vending_app PRIVATE
        ${CMAKE_SOURCE_DIR}
        ${lvgl_SOURCE_DIR}/src/drivers/sdl
)
