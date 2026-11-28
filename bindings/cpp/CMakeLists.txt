cmake_minimum_required(VERSION 3.16)
project(robot_bus_cpp VERSION 3.3.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
# Match ROS 2 Humble (and later) minimum: C++17 / C99.
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

option(ROBOT_BUS_BUILD_TESTS "Build C++ smoke tests" ON)
option(ROBOT_BUS_BUILD_MSGS "Build librobot_bus_msgs from generated protobufs" ON)
# Build native rclcpp Ros2Bridge library (does NOT enable cargo --features ros2).
option(ROBOT_BUS_ROS2 "Enable native C++ ROS 2 bridge (rclcpp)" OFF)

# Prefer protobuf's CMake package config so Abseil INTERFACE deps propagate
# (ABSL_CONSUME_DLL / PROTOBUF_USE_DLLS on Windows shared builds). CMake's
# FindProtobuf MODULE mode only locates libprotobuf and skips Abseil, which
# causes MSVC LNK2001 for LogMessage::operator<< in generated .pb.cc.
# Older distro packages (Ubuntu 22.04's 3.12) ship no config file; fall back
# to the module and synthesize the same imported target.
find_package(Protobuf CONFIG QUIET)
if(NOT Protobuf_FOUND)
  find_package(Protobuf REQUIRED)
  if(NOT TARGET protobuf::libprotobuf)
    add_library(protobuf::libprotobuf UNKNOWN IMPORTED)
    set_target_properties(protobuf::libprotobuf PROPERTIES
      IMPORTED_LOCATION "${Protobuf_LIBRARIES}"
      INTERFACE_INCLUDE_DIRECTORIES "${Protobuf_INCLUDE_DIRS}")
  endif()
endif()

set(ROBOT_BUS_GENERATE_MSGS "AUTO" CACHE STRING
  "Generate C++ stubs with the protobuf found here. AUTO regenerates when pregenerated stubs do not match. ON always generates. OFF keeps bindings/cpp/generated.")
set_property(CACHE ROBOT_BUS_GENERATE_MSGS PROPERTY STRINGS AUTO ON OFF)
set(ROBOT_BUS_PROTO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../proto")
set(ROBOT_BUS_PREGENERATED_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/generated")
set(ROBOT_BUS_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/include")

# Code baked into a pregenerated header (#if PROTOBUF_VERSION != N) or, for
# protobuf 3.x stubs, absent. The installed runtime's matching macro is
# PROTOBUF_VERSION (22+) or GOOGLE_PROTOBUF_VERSION (3.x).
function(_robot_bus_header_version header macro out_var)
  set(${out_var} "" PARENT_SCOPE)
  if(NOT EXISTS "${header}")
    return()
  endif()
  file(READ "${header}" _text LIMIT 8000)
  string(REGEX MATCH "#define ${macro} ([0-9]+)" _match "${_text}")
  if(CMAKE_MATCH_1)
    set(${out_var} "${CMAKE_MATCH_1}" PARENT_SCOPE)
  endif()
endfunction()

function(_robot_bus_runtime_version out_var)
  set(_header "")
  if(TARGET protobuf::libprotobuf)
    get_target_property(_incs protobuf::libprotobuf INTERFACE_INCLUDE_DIRECTORIES)
    if(NOT _incs)
      set(_incs "")
    endif()
    foreach(_dir IN LISTS _incs Protobuf_INCLUDE_DIRS)
      if(EXISTS "${_dir}/google/protobuf/runtime_version.h")
        set(_header "${_dir}/google/protobuf/runtime_version.h")
        break()
      elseif(EXISTS "${_dir}/google/protobuf/stubs/common.h")
        set(_header "${_dir}/google/protobuf/stubs/common.h")
      endif()
    endforeach()
  endif()
  set(_code "")
  if(_header MATCHES "runtime_version.h$")
    _robot_bus_header_version("${_header}" PROTOBUF_VERSION _code)
  elseif(NOT _header STREQUAL "")
    _robot_bus_header_version("${_header}" GOOGLE_PROTOBUF_VERSION _code)
  endif()
  set(${out_var} "${_code}" PARENT_SCOPE)
endfunction()

function(_robot_bus_pregenerated_version out_var)
  set(${out_var} "" PARENT_SCOPE)
  file(GLOB _sample "${ROBOT_BUS_PREGENERATED_ROOT}/robot_bus/std_msgs/msg/v1/header.pb.h")
  if(NOT _sample)
    return()
  endif()
  file(READ "${_sample}" _text LIMIT 2500)
  string(REGEX MATCH "PROTOBUF_VERSION != ([0-9]+)" _match "${_text}")
  if(CMAKE_MATCH_1)
    set(${out_var} "${CMAKE_MATCH_1}" PARENT_SCOPE)
  endif()
endfunction()

set(_robot_bus_generate OFF)
if(ROBOT_BUS_GENERATE_MSGS STREQUAL "ON")
  set(_robot_bus_generate ON)
elseif(ROBOT_BUS_GENERATE_MSGS STREQUAL "OFF")
  set(_robot_bus_generate OFF)
else()
  _robot_bus_runtime_version(_robot_bus_runtime)
  _robot_bus_pregenerated_version(_robot_bus_pregenerated)
  if(_robot_bus_pregenerated STREQUAL "")
    set(_robot_bus_generate ON)
  elseif(NOT _robot_bus_runtime STREQUAL "" AND NOT _robot_bus_runtime STREQUAL _robot_bus_pregenerated)
    set(_robot_bus_generate ON)
  endif()
endif()

if(_robot_bus_generate)
  if(NOT EXISTS "${ROBOT_BUS_PROTO_ROOT}/std_msgs/msg/v1/header.proto")
    message(FATAL_ERROR
      "protobuf ${Protobuf_VERSION} does not match the pregenerated C++ stubs, "
      "and ${ROBOT_BUS_PROTO_ROOT} is not available to regenerate them. "
      "Install the protobuf that matches bindings/cpp/generated, or build from a git checkout that contains proto/.")
  endif()
  if(NOT Protobuf_PROTOC_EXECUTABLE)
    message(FATAL_ERROR "protoc was not found next to the protobuf runtime")
  endif()
  execute_process(
    COMMAND "${Protobuf_PROTOC_EXECUTABLE}" --version
    OUTPUT_VARIABLE _robot_bus_protoc_version
    OUTPUT_STRIP_TRAILING_WHITESPACE
    ERROR_QUIET)
  # proto3 `optional` needs protoc 3.15+. Ubuntu 22.04's 3.12 cannot compile these files.
  if(_robot_bus_protoc_version MATCHES "([0-9]+)\\.([0-9]+)")
    if(CMAKE_MATCH_1 EQUAL 3 AND CMAKE_MATCH_2 LESS 15)
      message(FATAL_ERROR
        "${_robot_bus_protoc_version} cannot compile these protos: proto3 optional requires protoc 3.15 or newer. "
        "Ubuntu 22.04 ships 3.12. Use Ubuntu 24.04's protobuf, or the prebuilt package's protobuf (CPP_PROTOC in scripts/protobuf-versions.env).")
    endif()
  endif()
  set(ROBOT_BUS_GENERATED_ROOT "${CMAKE_CURRENT_BINARY_DIR}/generated")
  message(STATUS "robot_bus: generating C++ stubs with ${Protobuf_PROTOC_EXECUTABLE} into ${ROBOT_BUS_GENERATED_ROOT}")
else()
  set(ROBOT_BUS_GENERATED_ROOT "${ROBOT_BUS_PREGENERATED_ROOT}")
  message(STATUS "robot_bus: using pregenerated C++ stubs in ${ROBOT_BUS_GENERATED_ROOT}")
endif()

# --- msgs library ---
if(ROBOT_BUS_BUILD_MSGS)
  if(_robot_bus_generate)
    file(GLOB_RECURSE ROBOT_BUS_PROTO_FILES CONFIGURE_DEPENDS
      "${ROBOT_BUS_PROTO_ROOT}/*.proto")
    list(FILTER ROBOT_BUS_PROTO_FILES EXCLUDE REGEX "/robot_bus_interfaces/grpc/")
    set(ROBOT_BUS_PB_SOURCES "")
    foreach(_proto IN LISTS ROBOT_BUS_PROTO_FILES)
      file(RELATIVE_PATH _rel "${ROBOT_BUS_PROTO_ROOT}" "${_proto}")
      string(REGEX REPLACE "\\.proto$" ".pb.cc" _rel_cc "${_rel}")
      list(APPEND ROBOT_BUS_PB_SOURCES "${ROBOT_BUS_GENERATED_ROOT}/robot_bus/${_rel_cc}")
    endforeach()
    # protobuf_generate() cannot add the robot_bus/ include prefix or the
    # dllexport_decl this shared library needs. Use the protoc find_package
    # located, which is what that function would invoke, then rewrite includes.
    set(_gen_root "${ROBOT_BUS_GENERATED_ROOT}/robot_bus")
    add_custom_command(
      OUTPUT ${ROBOT_BUS_PB_SOURCES}
      COMMAND ${CMAKE_COMMAND} -E rm -rf "${ROBOT_BUS_GENERATED_ROOT}"
      COMMAND ${CMAKE_COMMAND} -E make_directory "${_gen_root}"
      COMMAND ${Protobuf_PROTOC_EXECUTABLE}
              --proto_path=${ROBOT_BUS_PROTO_ROOT}
              --cpp_out=dllexport_decl=ROBOT_BUS_MSGS_API:${_gen_root}
              ${ROBOT_BUS_PROTO_FILES}
      COMMAND ${CMAKE_COMMAND}
              -DROBOT_BUS_GEN_ROOT=${_gen_root}
              -P ${CMAKE_CURRENT_SOURCE_DIR}/cmake/rewrite_pb_includes.cmake
      DEPENDS ${ROBOT_BUS_PROTO_FILES}
              ${CMAKE_CURRENT_SOURCE_DIR}/cmake/rewrite_pb_includes.cmake
      COMMENT "Generating C++ protobuf stubs"
      VERBATIM)
  else()
    file(GLOB_RECURSE ROBOT_BUS_PB_SOURCES CONFIGURE_DEPENDS
      "${ROBOT_BUS_GENERATED_ROOT}/robot_bus/*.pb.cc")
    if(NOT ROBOT_BUS_PB_SOURCES)
      message(FATAL_ERROR
        "No pregenerated C++ stubs in ${ROBOT_BUS_GENERATED_ROOT}. "
        "Run scripts/generate_cpp_msgs.py or configure with -DROBOT_BUS_GENERATE_MSGS=ON.")
    endif()
  endif()
  add_library(robot_bus_msgs SHARED ${ROBOT_BUS_PB_SOURCES})
  add_library(robot_bus::msgs ALIAS robot_bus_msgs)
  target_include_directories(robot_bus_msgs
    PUBLIC
      $<BUILD_INTERFACE:${ROBOT_BUS_GENERATED_ROOT}>
      $<BUILD_INTERFACE:${ROBOT_BUS_INCLUDE_DIR}>
      $<INSTALL_INTERFACE:include>)
  target_link_libraries(robot_bus_msgs PUBLIC protobuf::libprotobuf)
  add_executable(msgs_proto_only tests/msgs_proto_only.cpp)
  target_link_libraries(msgs_proto_only PRIVATE robot_bus_msgs)
  # Belt-and-suspenders: shared Abseil on MSVC exports LogMessage templates
  # from abseil_dll; without ABSL_CONSUME_DLL the linker looks for static
  # symbols and fails with LNK2001.
  if(WIN32)
    target_compile_definitions(robot_bus_msgs PRIVATE ROBOT_BUS_MSGS_BUILDING_DLL)
    get_target_property(_rb_pb_defs protobuf::libprotobuf INTERFACE_COMPILE_DEFINITIONS)
    if(_rb_pb_defs MATCHES "PROTOBUF_USE_DLLS")
      target_compile_definitions(robot_bus_msgs PUBLIC ABSL_CONSUME_DLL)
    endif()
  endif()
  set_target_properties(robot_bus_msgs PROPERTIES
    OUTPUT_NAME robot_bus_msgs
    VERSION ${PROJECT_VERSION}
    SOVERSION 0
    BUILD_RPATH_USE_ORIGIN TRUE
    INSTALL_RPATH "\$ORIGIN")
endif()

# --- Rust FFI shared library (prebuilt or built via cargo) ---
set(ROBOT_BUS_FFI_DIR "${CMAKE_CURRENT_SOURCE_DIR}/native")
set(ROBOT_BUS_FFI_TARGET_DIR "${ROBOT_BUS_FFI_DIR}/target/release")
if(APPLE)
  set(ROBOT_BUS_FFI_BUILT "${ROBOT_BUS_FFI_TARGET_DIR}/librobot_bus_c.dylib")
  set(ROBOT_BUS_FFI_SONAME "librobot_bus.dylib")
elseif(WIN32)
  set(ROBOT_BUS_FFI_BUILT "${ROBOT_BUS_FFI_TARGET_DIR}/robot_bus_c.dll")
else()
  set(ROBOT_BUS_FFI_BUILT "${ROBOT_BUS_FFI_TARGET_DIR}/librobot_bus_c.so")
  set(ROBOT_BUS_FFI_SONAME "librobot_bus.so")
endif()

set(ROBOT_BUS_FFI_LINK "${ROBOT_BUS_FFI_BUILT}")
set(ROBOT_BUS_FFI_BYPRODUCTS "${ROBOT_BUS_FFI_BUILT}")
if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  # Rust's cdylib has no ELF SONAME. Link with -lrobot_bus from the build
  # directory so the bridge records the installed name instead of a build path.
  set(ROBOT_BUS_FFI_LINK "${CMAKE_CURRENT_BINARY_DIR}/${ROBOT_BUS_FFI_SONAME}")
  list(APPEND ROBOT_BUS_FFI_BYPRODUCTS "${ROBOT_BUS_FFI_LINK}")
  set(ROBOT_BUS_FFI_STAGE_COMMAND COMMAND ${CMAKE_COMMAND} -E copy_if_different
    "${ROBOT_BUS_FFI_BUILT}" "${ROBOT_BUS_FFI_LINK}")
endif()
add_custom_target(robot_bus_ffi_build
  COMMAND cargo build --release --manifest-path "${ROBOT_BUS_FFI_DIR}/Cargo.toml"
  ${ROBOT_BUS_FFI_STAGE_COMMAND}
  WORKING_DIRECTORY "${ROBOT_BUS_FFI_DIR}"
  COMMENT "Building robot-bus C FFI (cargo)"
  BYPRODUCTS ${ROBOT_BUS_FFI_BYPRODUCTS}
  VERBATIM)

add_library(robot_bus SHARED IMPORTED GLOBAL)
add_library(robot_bus::robot_bus ALIAS robot_bus)
add_dependencies(robot_bus robot_bus_ffi_build)
set_target_properties(robot_bus PROPERTIES
  IMPORTED_LOCATION "${ROBOT_BUS_FFI_LINK}"
  IMPORTED_LOCATION_RELEASE "${ROBOT_BUS_FFI_LINK}"
  IMPORTED_CONFIGURATIONS RELEASE
  MAP_IMPORTED_CONFIG_DEBUG Release
  MAP_IMPORTED_CONFIG_RELWITHDEBINFO Release
  MAP_IMPORTED_CONFIG_MINSIZEREL Release
  INTERFACE_INCLUDE_DIRECTORIES "${ROBOT_BUS_INCLUDE_DIR}")
if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  set_property(TARGET robot_bus PROPERTY IMPORTED_NO_SONAME TRUE)
endif()
if(WIN32)
  # Cargo builds this cdylib once, as Release. A multi-config generator still
  # requires IMPORTED_IMPLIB_RELEASE or Debug/RelWithDebInfo/MinSizeRel fail
  # at generate time with "IMPORTED_IMPLIB not set".
  set_target_properties(robot_bus PROPERTIES
    IMPORTED_IMPLIB "${ROBOT_BUS_FFI_TARGET_DIR}/robot_bus_c.dll.lib"
    IMPORTED_IMPLIB_RELEASE "${ROBOT_BUS_FFI_TARGET_DIR}/robot_bus_c.dll.lib")
endif()

# --- Native rclcpp Ros2Bridge (optional) ---
if(ROBOT_BUS_ROS2)
  find_package(rclcpp REQUIRED)
  find_package(rclcpp_action REQUIRED)
  find_package(std_msgs REQUIRED)
  find_package(sensor_msgs REQUIRED)
  find_package(geometry_msgs REQUIRED)
  find_package(nav_msgs REQUIRED)
  find_package(std_srvs REQUIRED)
  find_package(example_interfaces REQUIRED)
  find_package(builtin_interfaces REQUIRED)
  find_package(action_msgs REQUIRED)
  find_package(diagnostic_msgs REQUIRED)
  find_package(shape_msgs REQUIRED)
  find_package(stereo_msgs REQUIRED)
  find_package(tf2_msgs REQUIRED)
  find_package(trajectory_msgs REQUIRED)
  find_package(unique_identifier_msgs REQUIRED)
  find_package(visualization_msgs REQUIRED)
  message(STATUS "robot_bus: native ROS 2 bridge enabled (core distro pkgs; source ROS 2 before build)")

  add_library(robot_bus_ros2_bridge SHARED
    src/ros2_bridge.cpp
    src/ros2_bridge_wire.cpp)
  add_library(robot_bus::ros2_bridge ALIAS robot_bus_ros2_bridge)
  target_compile_definitions(robot_bus_ros2_bridge PUBLIC ROBOT_BUS_HAS_ROS2)
  target_include_directories(robot_bus_ros2_bridge
    PUBLIC
      $<BUILD_INTERFACE:${ROBOT_BUS_INCLUDE_DIR}>
      $<BUILD_INTERFACE:${ROBOT_BUS_GENERATED_ROOT}>
      $<INSTALL_INTERFACE:include>)
  target_link_libraries(robot_bus_ros2_bridge
    PUBLIC
      robot_bus
      robot_bus_msgs
      rclcpp::rclcpp
      rclcpp_action::rclcpp_action
      ${std_msgs_TARGETS}
      ${sensor_msgs_TARGETS}
      ${geometry_msgs_TARGETS}
      ${nav_msgs_TARGETS}
      ${std_srvs_TARGETS}
      ${example_interfaces_TARGETS}
      ${builtin_interfaces_TARGETS}
      ${action_msgs_TARGETS}
      ${diagnostic_msgs_TARGETS}
      ${shape_msgs_TARGETS}
      ${stereo_msgs_TARGETS}
      ${tf2_msgs_TARGETS}
      ${trajectory_msgs_TARGETS}
      ${unique_identifier_msgs_TARGETS}
      ${visualization_msgs_TARGETS})
  # ament packages export modern targets via *_TARGETS; fall back to older names.
  foreach(_pkg std_msgs sensor_msgs geometry_msgs nav_msgs std_srvs example_interfaces
               builtin_interfaces action_msgs diagnostic_msgs shape_msgs stereo_msgs
               tf2_msgs trajectory_msgs unique_identifier_msgs visualization_msgs)
    if(TARGET ${_pkg}::${_pkg})
      target_link_libraries(robot_bus_ros2_bridge PUBLIC ${_pkg}::${_pkg})
    endif()
  endforeach()
  add_dependencies(robot_bus_ros2_bridge robot_bus_ffi_build)
  set_target_properties(robot_bus_ros2_bridge PROPERTIES
    OUTPUT_NAME robot_bus_ros2_bridge
    VERSION ${PROJECT_VERSION}
    SOVERSION 0
    BUILD_RPATH_USE_ORIGIN TRUE
    INSTALL_RPATH "\$ORIGIN")
endif()

if(ROBOT_BUS_BUILD_TESTS)
  set(ROBOT_BUS_TEST_SOURCES
    pub_sub_imu
    service_set_bool
    action_fibonacci
    timer_spin
    msgs_roundtrip
    ws_node
    inproc_context
    federation_opts
    node_parameters
    typed_api
    interop_peer
    ros2_bridge_stub
    ros2_bridge_diagnostics)

  set(ROBOT_BUS_TEST_TARGETS)
  foreach(t ${ROBOT_BUS_TEST_SOURCES})
    add_executable(${t} tests/${t}.cpp)
    target_include_directories(${t} PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/tests")
    target_link_libraries(${t} PRIVATE robot_bus robot_bus_msgs)
    add_dependencies(${t} robot_bus_ffi_build)
    if(ROBOT_BUS_ROS2 AND t STREQUAL "ros2_bridge_stub")
      target_link_libraries(${t} PRIVATE robot_bus_ros2_bridge)
    endif()
    list(APPEND ROBOT_BUS_TEST_TARGETS ${t})
  endforeach()
  if(ROBOT_BUS_ROS2)
    # Real rclcpp integration; needs a sourced ROS 2 (not part of the stub-only CI build).
    add_executable(ros2_bridge_service_failure tests/ros2_bridge_service_failure.cpp)
    target_include_directories(ros2_bridge_service_failure PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/tests")
    target_link_libraries(ros2_bridge_service_failure PRIVATE robot_bus robot_bus_msgs robot_bus_ros2_bridge)
    add_dependencies(ros2_bridge_service_failure robot_bus_ffi_build)
    list(APPEND ROBOT_BUS_TEST_TARGETS ros2_bridge_service_failure)
  endif()
  add_custom_target(cpp_tests DEPENDS ${ROBOT_BUS_TEST_TARGETS})
endif()

option(ROBOT_BUS_BUILD_EXAMPLES "Build repo examples/ C++ demos" OFF)
if(ROBOT_BUS_BUILD_EXAMPLES)
  set(ROBOT_BUS_EXAMPLES_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../examples")
  set(ROBOT_BUS_EXAMPLE_SOURCES
    topic_imu_listener:topic_imu/cpp/listener.cpp
    topic_imu_talker:topic_imu/cpp/talker.cpp
    service_set_bool_server:service_set_bool/cpp/server.cpp
    service_set_bool_client:service_set_bool/cpp/client.cpp
    action_fibonacci_server:action_fibonacci/cpp/server.cpp
    action_fibonacci_client:action_fibonacci/cpp/client.cpp)
  set(ROBOT_BUS_EXAMPLE_TARGETS)
  foreach(entry ${ROBOT_BUS_EXAMPLE_SOURCES})
    string(REPLACE ":" ";" parts "${entry}")
    list(GET parts 0 target_name)
    list(GET parts 1 rel_src)
    add_executable(${target_name} "${ROBOT_BUS_EXAMPLES_ROOT}/${rel_src}")
    target_link_libraries(${target_name} PRIVATE robot_bus robot_bus_msgs)
    add_dependencies(${target_name} robot_bus_ffi_build)
    set_target_properties(${target_name} PROPERTIES
      RUNTIME_OUTPUT_DIRECTORY "${ROBOT_BUS_EXAMPLES_ROOT}/build")
    list(APPEND ROBOT_BUS_EXAMPLE_TARGETS ${target_name})
  endforeach()
  if(ROBOT_BUS_ROS2)
    add_executable(ros2_bridge_builtin
      "${ROBOT_BUS_EXAMPLES_ROOT}/ros2_bridge/cpp/builtin.cpp")
    target_link_libraries(ros2_bridge_builtin PRIVATE
      robot_bus robot_bus_msgs robot_bus_ros2_bridge)
    add_dependencies(ros2_bridge_builtin robot_bus_ffi_build)
    set_target_properties(ros2_bridge_builtin PROPERTIES
      RUNTIME_OUTPUT_DIRECTORY "${ROBOT_BUS_EXAMPLES_ROOT}/build")
    list(APPEND ROBOT_BUS_EXAMPLE_TARGETS ros2_bridge_builtin)

    # Custom mapper: compile example-local my_pkg protobuf (not SDK msgs).
    add_executable(ros2_bridge_custom_add_two_ints
      "${ROBOT_BUS_EXAMPLES_ROOT}/ros2_bridge/cpp/custom_add_two_ints.cpp"
      "${ROBOT_BUS_EXAMPLES_ROOT}/ros2_bridge/generated/my_pkg/srv/v1/add_two_ints.pb.cc")
    target_include_directories(ros2_bridge_custom_add_two_ints PRIVATE
      "${ROBOT_BUS_EXAMPLES_ROOT}/ros2_bridge/generated")
    target_link_libraries(ros2_bridge_custom_add_two_ints PRIVATE
      robot_bus robot_bus_msgs robot_bus_ros2_bridge protobuf::libprotobuf)
    add_dependencies(ros2_bridge_custom_add_two_ints robot_bus_ffi_build)
    set_target_properties(ros2_bridge_custom_add_two_ints PROPERTIES
      RUNTIME_OUTPUT_DIRECTORY "${ROBOT_BUS_EXAMPLES_ROOT}/build")
    list(APPEND ROBOT_BUS_EXAMPLE_TARGETS ros2_bridge_custom_add_two_ints)
  endif()
  add_custom_target(cpp_examples DEPENDS ${ROBOT_BUS_EXAMPLE_TARGETS})
endif()

include(GNUInstallDirs)

install(DIRECTORY "${ROBOT_BUS_INCLUDE_DIR}/"
  DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
install(DIRECTORY "${ROBOT_BUS_GENERATED_ROOT}/robot_bus"
  DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

if(ROBOT_BUS_BUILD_MSGS)
  install(TARGETS robot_bus_msgs
    EXPORT robot_busTargets
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})
endif()

if(ROBOT_BUS_ROS2)
  install(TARGETS robot_bus_ros2_bridge
    EXPORT robot_busTargets
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})
endif()

if(WIN32)
  install(FILES "${ROBOT_BUS_FFI_BUILT}" DESTINATION ${CMAKE_INSTALL_BINDIR})
  install(FILES "${ROBOT_BUS_FFI_BUILT}.lib"
    DESTINATION ${CMAKE_INSTALL_LIBDIR} RENAME robot_bus.lib)
else()
  install(FILES "${ROBOT_BUS_FFI_BUILT}"
    DESTINATION ${CMAKE_INSTALL_LIBDIR} RENAME "${ROBOT_BUS_FFI_SONAME}")
endif()

# Also install broker binary if present at repo root target.
set(ROBOT_BUS_BROKER "${CMAKE_CURRENT_SOURCE_DIR}/../../target/release/robot_bus_broker")
if(EXISTS "${ROBOT_BUS_BROKER}" OR TRUE)
  install(PROGRAMS "${ROBOT_BUS_BROKER}"
    DESTINATION ${CMAKE_INSTALL_BINDIR}
    OPTIONAL)
endif()

configure_file(
  "${CMAKE_CURRENT_SOURCE_DIR}/cmake/robot_bus.pc.in"
  "${CMAKE_CURRENT_BINARY_DIR}/robot_bus.pc"
  @ONLY)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/robot_bus.pc"
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig)

include(CMakePackageConfigHelpers)
configure_package_config_file(
  "${CMAKE_CURRENT_SOURCE_DIR}/cmake/robot_busConfig.cmake.in"
  "${CMAKE_CURRENT_BINARY_DIR}/robot_busConfig.cmake"
  INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/robot_bus)
write_basic_package_version_file(
  "${CMAKE_CURRENT_BINARY_DIR}/robot_busConfigVersion.cmake"
  VERSION ${PROJECT_VERSION}
  COMPATIBILITY SameMajorVersion)
install(FILES
  "${CMAKE_CURRENT_BINARY_DIR}/robot_busConfig.cmake"
  "${CMAKE_CURRENT_BINARY_DIR}/robot_busConfigVersion.cmake"
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/robot_bus)
if(ROBOT_BUS_BUILD_MSGS OR ROBOT_BUS_ROS2)
  install(EXPORT robot_busTargets
    NAMESPACE robot_bus::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/robot_bus)
endif()
