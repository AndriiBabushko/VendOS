cmake_minimum_required(VERSION 3.14)
project(VendOS_UI LANGUAGES C CXX)

find_package(PkgConfig REQUIRED)
pkg_check_modules(SDL2_PKG REQUIRED sdl2)
find_package(SDL2 REQUIRED)
include_directories(${SDL2_PKG_INCLUDE_DIRS} ${lvgl_SOURCE_DIR}/src/others/xml)

include(FetchContent)
set(LV_CONF_PATH "${CMAKE_SOURCE_DIR}/lv_conf.h" CACHE FILEPATH "" FORCE)
FetchContent_Declare(lvgl
        GIT_REPOSITORY https://github.com/lvgl/lvgl.git
        GIT_TAG        v9.3.0
)
FetchContent_MakeAvailable(lvgl)

if(APPLE)
    set(SONY_SDK_SRC "${CMAKE_SOURCE_DIR}/third_party/sony_sdk/CrSDK_v1-2_Mac")
elseif(WIN32)
    set(SONY_SDK_SRC "${CMAKE_SOURCE_DIR}/third_party/sony_sdk/CrSDK_v1_Windows")
elseif(UNIX)
    if(CMAKE_SIZEOF_VOID_P EQUAL 8 AND CMAKE_SYSTEM_PROCESSOR MATCHES "x86_64")
        set(SONY_SDK_SRC "${CMAKE_SOURCE_DIR}/third_party/sony_sdk/CrSDK_v1-5_Linux 64bit x86")
    elseif(CMAKE_SIZEOF_VOID_P EQUAL 8 AND CMAKE_SYSTEM_PROCESSOR MATCHES "(aarch64|arm64)")
        set(SONY_SDK_SRC "${CMAKE_SOURCE_DIR}/third_party/sony_sdk/CrSDK_v1-4_Linux 64 bit ARMv8")
    elseif(CMAKE_SIZEOF_VOID_P EQUAL 4 AND CMAKE_SYSTEM_PROCESSOR MATCHES "armv7l")
        set(SONY_SDK_SRC "${CMAKE_SOURCE_DIR}/third_party/sony_sdk/CrSDK_v1-3_Linux 32 bit ARMv7")
    else()
        message(FATAL_ERROR "Unsupported Linux architecture ${CMAKE_SYSTEM_PROCESSOR}/${CMAKE_SIZEOF_VOID_P}")
    endif()
else()
    message(FATAL_ERROR "Unsupported platform: ${CMAKE_SYSTEM_NAME}")
endif()

set(SONY_SDK_STAGE "${CMAKE_BINARY_DIR}/sony_sdk_stage")
set(SONY_SDK_STAGE_INC  "${SONY_SDK_STAGE}/app/CRSDK")
set(SONY_SDK_STAGE_LIB  "${SONY_SDK_STAGE}/external/crsdk")
set(SONY_SDK_STAGE_ADPT "${SONY_SDK_STAGE}/external/crsdk/CrAdapter")

file(MAKE_DIRECTORY ${SONY_SDK_STAGE_INC} ${SONY_SDK_STAGE_LIB} ${SONY_SDK_STAGE_ADPT})

file(COPY ${SONY_SDK_SRC}/app/CRSDK DESTINATION ${SONY_SDK_STAGE}/app)
file(COPY ${SONY_SDK_SRC}/external/crsdk DESTINATION ${SONY_SDK_STAGE}/external)

if(APPLE)
    file(GLOB_RECURSE _all_dylib
            "${SONY_SDK_STAGE_LIB}/lib*.dylib"
            "${SONY_SDK_STAGE_ADPT}/lib*.dylib"
    )

    cmake_policy(SET CMP0175 NEW)

    add_custom_target(patch_crsdk
            COMMENT "Patching install_name inside CrSDK .dylib"
    )

    foreach(_lib IN LISTS _all_dylib)
        get_filename_component(_fname ${_lib} NAME)
        add_custom_command(
                TARGET patch_crsdk
                POST_BUILD
                COMMAND ${CMAKE_COMMAND} -E echo " - patching ${_fname}"
                COMMAND install_name_tool -id @rpath/${_fname} ${_lib}
        )
    endforeach()
endif()

file(GLOB APP_SRCS
        src/main.c
        src/screens/*.c
)

add_executable(vending_app ${APP_SRCS})

if(APPLE)
    add_dependencies(vending_app patch_crsdk)
endif()

target_include_directories(vending_app PRIVATE
        ${lvgl_SOURCE_DIR}/src
        ${lvgl_SOURCE_DIR}/src/drivers/sdl
        ${lvgl_SOURCE_DIR}/src/others/xml
        ${CMAKE_SOURCE_DIR}/src/screens
)

target_link_libraries(vending_app PRIVATE
        lvgl::lvgl
        SDL2::SDL2
)

if(WIN32)
    target_link_libraries(vending_app PRIVATE SDL2::SDL2main)
endif()

if(APPLE OR UNIX)
    set_target_properties(vending_app PROPERTIES
            BUILD_WITH_INSTALL_RPATH TRUE
            INSTALL_RPATH "${SONY_SDK_STAGE}/external/crsdk"
    )
endif()
