cmake_minimum_required(VERSION 3.16)
project(robot_bus_cpp VERSION 0.1.9 LANGUAGES CXX)

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
find_package(Protobuf CONFIG REQUIRED)

set(ROBOT_BUS_GENERATED_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/generated")
set(ROBOT_BUS_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/include")

# --- msgs library from committed generated sources ---
if(ROBOT_BUS_BUILD_MSGS)
  file(GLOB_RECURSE ROBOT_BUS_PB_SOURCES CONFIGURE_DEPENDS
    "${ROBOT_BUS_GENERATED_ROOT}/robot_bus/*.pb.cc")
  add_library(robot_bus_msgs SHARED ${ROBOT_BUS_PB_SOURCES})
  add_library(robot_bus::msgs ALIAS robot_bus_msgs)
  target_include_directories(robot_bus_msgs
    PUBLIC
      $<BUILD_INTERFACE:${ROBOT_BUS_GENERATED_ROOT}>
      $<INSTALL_INTERFACE:include>)
  target_link_libraries(robot_bus_msgs PUBLIC protobuf::libprotobuf)
  # Belt-and-suspenders: shared Abseil on MSVC exports LogMessage templates
  # from abseil_dll; without ABSL_CONSUME_DLL the linker looks for static
  # symbols and fails with LNK2001.
  if(WIN32)
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
  set(ROBOT_BUS_FFI_SONAME "robot_bus.dll")
else()
  set(ROBOT_BUS_FFI_BUILT "${ROBOT_BUS_FFI_TARGET_DIR}/librobot_bus_c.so")
  set(ROBOT_BUS_FFI_SONAME "librobot_bus.so")
endif()

add_custom_target(robot_bus_ffi_build
  COMMAND cargo build --release --manifest-path "${ROBOT_BUS_FFI_DIR}/Cargo.toml"
  WORKING_DIRECTORY "${ROBOT_BUS_FFI_DIR}"
  COMMENT "Building robot-bus C FFI (cargo)"
  BYPRODUCTS "${ROBOT_BUS_FFI_BUILT}")

add_library(robot_bus SHARED IMPORTED GLOBAL)
add_library(robot_bus::robot_bus ALIAS robot_bus)
add_dependencies(robot_bus robot_bus_ffi_build)
set_target_properties(robot_bus PROPERTIES
  IMPORTED_LOCATION "${ROBOT_BUS_FFI_BUILT}"
  INTERFACE_INCLUDE_DIRECTORIES "${ROBOT_BUS_INCLUDE_DIR}")
if(WIN32)
  set_target_properties(robot_bus PROPERTIES
    IMPORTED_IMPLIB "${ROBOT_BUS_FFI_TARGET_DIR}/robot_bus_c.dll.lib")
endif()

# --- Native rclcpp Ros2Bridge (optional) ---
if(ROBOT_BUS_ROS2)
  find_package(rclcpp REQUIRED)
  find_package(rclcpp_action REQUIRED)
  find_package(std_msgs REQUIRED)
  find_package(sensor_msgs REQUIRED)
  find_package(std_srvs REQUIRED)
  find_package(example_interfaces REQUIRED)
  message(STATUS "robot_bus: native ROS 2 bridge enabled (source ROS 2 before build)")

  add_library(robot_bus_ros2_bridge SHARED src/ros2_bridge.cpp)
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
      ${std_srvs_TARGETS}
      ${example_interfaces_TARGETS})
  # ament packages export modern targets via *_TARGETS; fall back to older names.
  if(TARGET std_msgs::std_msgs)
    target_link_libraries(robot_bus_ros2_bridge PUBLIC std_msgs::std_msgs)
  endif()
  if(TARGET sensor_msgs::sensor_msgs)
    target_link_libraries(robot_bus_ros2_bridge PUBLIC sensor_msgs::sensor_msgs)
  endif()
  if(TARGET std_srvs::std_srvs)
    target_link_libraries(robot_bus_ros2_bridge PUBLIC std_srvs::std_srvs)
  endif()
  if(TARGET example_interfaces::example_interfaces)
    target_link_libraries(robot_bus_ros2_bridge PUBLIC example_interfaces::example_interfaces)
  endif()
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
    ros2_bridge_stub)

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
  add_custom_target(cpp_tests DEPENDS ${ROBOT_BUS_TEST_TARGETS})
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

# Install renamed FFI shared library as librobot_bus.
install(CODE "
  file(INSTALL \"${ROBOT_BUS_FFI_BUILT}\"
       DESTINATION \"\${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_LIBDIR}\"
       RENAME \"${ROBOT_BUS_FFI_SONAME}\")
")

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
