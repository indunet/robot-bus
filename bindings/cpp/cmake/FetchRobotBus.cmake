# Fetch a prebuilt robot-bus SDK archive and call find_package(robot_bus).
#
# Copy this file into your project (it is also attached to each GitHub Release
# and stored inside the SDK at lib/cmake/robot_bus/FetchRobotBus.cmake).
#
#   cmake_minimum_required(VERSION 3.22)
#   include(cmake/FetchRobotBus.cmake)
#   robot_bus_fetch_sdk(VERSION 3.3.0 SHA256 <hash>)
#   target_link_libraries(my_app PRIVATE robot_bus::robot_bus robot_bus::msgs)
#   robot_bus_copy_runtime_libs(my_app)
#
# Platform and architecture are chosen from CMAKE_SYSTEM_NAME /
# CMAKE_SYSTEM_PROCESSOR. Override the download host with
# -DROBOT_BUS_SDK_BASE_URL=https://mirror.example/robot-bus
# (or the same environment variable). The URL is
# <base>/robot-bus-sdk_<version>_<platform>_<arch>.<ext>
# and for ROS 2 <base>/robot-bus-sdk-ros2-<humble|jazzy>_<version>_linux_<arch>.tar.gz.
#
# Offline / already unpacked tree:
#   -DFETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK=/path/to/unpacked/sdk
# or the same name in the environment.

macro(robot_bus_fetch_sdk)
  if(CMAKE_VERSION VERSION_LESS 3.22)
    message(FATAL_ERROR "robot_bus_fetch_sdk requires CMake 3.22 or newer (found ${CMAKE_VERSION})")
  endif()
  cmake_parse_arguments(_RBSDK "" "VERSION;ROS2;SHA256" "" ${ARGN})
  if(NOT _RBSDK_VERSION)
    message(FATAL_ERROR "robot_bus_fetch_sdk: VERSION is required")
  endif()
  if(_RBSDK_UNPARSED_ARGUMENTS)
    message(FATAL_ERROR "robot_bus_fetch_sdk: unexpected arguments: ${_RBSDK_UNPARSED_ARGUMENTS}")
  endif()

  if(NOT TARGET robot_bus::robot_bus)
    if(NOT ROBOT_BUS_SDK_BASE_URL AND DEFINED ENV{ROBOT_BUS_SDK_BASE_URL})
      set(ROBOT_BUS_SDK_BASE_URL "$ENV{ROBOT_BUS_SDK_BASE_URL}")
    endif()
    if(NOT FETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK
       AND DEFINED ENV{FETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK})
      set(FETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK "$ENV{FETCHCONTENT_SOURCE_DIR_ROBOT_BUS_SDK}")
    endif()

    if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
      set(_RBSDK_PLATFORM linux)
      set(_RBSDK_EXT tar.gz)
      if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(aarch64|arm64|ARM64)$")
        set(_RBSDK_ARCH arm64)
      elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "^(x86_64|amd64|AMD64)$")
        set(_RBSDK_ARCH amd64)
      else()
        message(FATAL_ERROR
          "robot_bus_fetch_sdk: unsupported Linux architecture '${CMAKE_SYSTEM_PROCESSOR}'")
      endif()
    elseif(CMAKE_SYSTEM_NAME STREQUAL "Darwin")
      set(_RBSDK_PLATFORM macos)
      set(_RBSDK_EXT tar.gz)
      if(NOT CMAKE_SYSTEM_PROCESSOR MATCHES "^(aarch64|arm64|ARM64)$")
        message(FATAL_ERROR
          "robot_bus_fetch_sdk: the macOS SDK is Apple Silicon only (got '${CMAKE_SYSTEM_PROCESSOR}')")
      endif()
      set(_RBSDK_ARCH arm64)
    elseif(CMAKE_SYSTEM_NAME STREQUAL "Windows")
      set(_RBSDK_PLATFORM windows)
      set(_RBSDK_EXT zip)
      if(NOT CMAKE_SYSTEM_PROCESSOR MATCHES "^(AMD64|x86_64|amd64)$")
        message(FATAL_ERROR
          "robot_bus_fetch_sdk: the Windows SDK is x64 only (got '${CMAKE_SYSTEM_PROCESSOR}')")
      endif()
      set(_RBSDK_ARCH x64)
    else()
      message(FATAL_ERROR "robot_bus_fetch_sdk: unsupported system '${CMAKE_SYSTEM_NAME}'")
    endif()

    if(_RBSDK_ROS2)
      if(NOT _RBSDK_PLATFORM STREQUAL "linux")
        message(FATAL_ERROR "robot_bus_fetch_sdk: ROS 2 SDK archives are Linux only")
      endif()
      if(NOT _RBSDK_ROS2 MATCHES "^(humble|jazzy)$")
        message(FATAL_ERROR "robot_bus_fetch_sdk: ROS2 must be humble or jazzy (got '${_RBSDK_ROS2}')")
      endif()
      set(_RBSDK_ARCHIVE
        "robot-bus-sdk-ros2-${_RBSDK_ROS2}_${_RBSDK_VERSION}_${_RBSDK_PLATFORM}_${_RBSDK_ARCH}.${_RBSDK_EXT}")
    else()
      set(_RBSDK_ARCHIVE
        "robot-bus-sdk_${_RBSDK_VERSION}_${_RBSDK_PLATFORM}_${_RBSDK_ARCH}.${_RBSDK_EXT}")
    endif()

    if(ROBOT_BUS_SDK_BASE_URL)
      set(_RBSDK_BASE "${ROBOT_BUS_SDK_BASE_URL}")
    else()
      set(_RBSDK_BASE
        "https://github.com/indunet/robot-bus/releases/download/v${_RBSDK_VERSION}")
    endif()
    set(_RBSDK_URL "${_RBSDK_BASE}/${_RBSDK_ARCHIVE}")
    message(STATUS "robot-bus SDK: ${_RBSDK_URL}")

    include(FetchContent)
    set(_RBSDK_DOWNLOAD_ARGS)
    if(_RBSDK_SHA256)
      list(APPEND _RBSDK_DOWNLOAD_ARGS URL_HASH "SHA256=${_RBSDK_SHA256}")
    endif()
    if(POLICY CMP0135)
      list(APPEND _RBSDK_DOWNLOAD_ARGS DOWNLOAD_EXTRACT_TIMESTAMP FALSE)
    endif()
    FetchContent_Declare(robot_bus_sdk URL "${_RBSDK_URL}" ${_RBSDK_DOWNLOAD_ARGS})
    FetchContent_MakeAvailable(robot_bus_sdk)
    set(ROBOT_BUS_SDK_DIR "${robot_bus_sdk_SOURCE_DIR}"
      CACHE PATH "Unpacked robot-bus SDK prefix" FORCE)

    find_package(robot_bus CONFIG REQUIRED
      PATHS "${robot_bus_sdk_SOURCE_DIR}"
      NO_DEFAULT_PATH)
  endif()

  unset(_RBSDK_VERSION)
  unset(_RBSDK_ROS2)
  unset(_RBSDK_SHA256)
  unset(_RBSDK_UNPARSED_ARGUMENTS)
  unset(_RBSDK_KEYWORDS_MISSING_VALUES)
  unset(_RBSDK_PLATFORM)
  unset(_RBSDK_ARCH)
  unset(_RBSDK_EXT)
  unset(_RBSDK_ARCHIVE)
  unset(_RBSDK_BASE)
  unset(_RBSDK_URL)
  unset(_RBSDK_DOWNLOAD_ARGS)
endmacro()

# Windows: copy the SDK's bin/ (DLLs and the broker) next to the target.
# Linux / macOS: record the SDK lib dir as a build RPATH and $ORIGIN /
# @loader_path as the install RPATH. The package config also injects a build
# RPATH, so a binary run from the build tree works before this helper runs.
function(robot_bus_copy_runtime_libs target)
  if(NOT TARGET ${target})
    message(FATAL_ERROR "robot_bus_copy_runtime_libs: no such target '${target}'")
  endif()
  if(NOT ROBOT_BUS_SDK_DIR)
    message(FATAL_ERROR "robot_bus_copy_runtime_libs: call robot_bus_fetch_sdk() first")
  endif()
  if(WIN32)
    add_custom_command(TARGET ${target} POST_BUILD
      COMMAND ${CMAKE_COMMAND} -E copy_directory
              "${ROBOT_BUS_SDK_DIR}/bin"
              "$<TARGET_FILE_DIR:${target}>"
      VERBATIM)
  elseif(APPLE)
    string(CONCAT _robot_bus_loader_path "@" "loader_path")
    set_property(TARGET ${target} APPEND PROPERTY BUILD_RPATH "${ROBOT_BUS_SDK_DIR}/lib")
    set_property(TARGET ${target} APPEND PROPERTY INSTALL_RPATH "${_robot_bus_loader_path}")
  else()
    set_property(TARGET ${target} APPEND PROPERTY BUILD_RPATH "${ROBOT_BUS_SDK_DIR}/lib")
    set_property(TARGET ${target} APPEND PROPERTY INSTALL_RPATH "$ORIGIN")
  endif()
endfunction()
