cmake_minimum_required(VERSION 3.16)
project(robot_bus_cpp VERSION 0.0.7 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
# Match ROS 2 Humble (and later) minimum: C++17 / C99.
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

option(ROBOT_BUS_BUILD_TESTS "Build C++ smoke tests" ON)
option(ROBOT_BUS_BUILD_MSGS "Build librobot_bus_msgs from generated protobufs" ON)

find_package(Protobuf REQUIRED)
find_package(PkgConfig REQUIRED)
pkg_check_modules(PROTOBUF_PC REQUIRED protobuf)

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
      $<INSTALL_INTERFACE:include>
    PRIVATE
      ${PROTOBUF_PC_INCLUDE_DIRS})
  target_compile_options(robot_bus_msgs PRIVATE ${PROTOBUF_PC_CFLAGS_OTHER})
  target_link_libraries(robot_bus_msgs PUBLIC protobuf::libprotobuf ${PROTOBUF_PC_LIBRARIES})
  target_link_directories(robot_bus_msgs PUBLIC ${PROTOBUF_PC_LIBRARY_DIRS})
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

if(ROBOT_BUS_BUILD_TESTS)
  set(ROBOT_BUS_TEST_SOURCES
    pub_sub_imu
    service_set_bool
    action_fibonacci
    timer_spin
    msgs_roundtrip
    grpc_node
    inproc_context
    node_parameters)
  set(ROBOT_BUS_TEST_TARGETS)
  foreach(t ${ROBOT_BUS_TEST_SOURCES})
    add_executable(${t} tests/${t}.cpp)
    target_include_directories(${t} PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/tests")
    target_link_libraries(${t} PRIVATE robot_bus robot_bus_msgs)
    add_dependencies(${t} robot_bus_ffi_build)
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
if(ROBOT_BUS_BUILD_MSGS)
  install(EXPORT robot_busTargets
    NAMESPACE robot_bus::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/robot_bus)
endif()
