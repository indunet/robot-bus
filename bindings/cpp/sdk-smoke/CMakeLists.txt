cmake_minimum_required(VERSION 3.24)
project(robot_bus_sdk_smoke LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT ROBOT_BUS_VERSION)
  message(FATAL_ERROR "Set -DROBOT_BUS_VERSION=<x.y.z> to the SDK release to fetch")
endif()

include("${CMAKE_CURRENT_LIST_DIR}/../cmake/FetchRobotBus.cmake")

if(ROBOT_BUS_SHA256)
  robot_bus_fetch_sdk(VERSION "${ROBOT_BUS_VERSION}" SHA256 "${ROBOT_BUS_SHA256}")
else()
  robot_bus_fetch_sdk(VERSION "${ROBOT_BUS_VERSION}")
endif()

add_executable(sdk_smoke main.cpp)
target_link_libraries(sdk_smoke PRIVATE robot_bus::robot_bus robot_bus::msgs)
robot_bus_copy_runtime_libs(sdk_smoke)
