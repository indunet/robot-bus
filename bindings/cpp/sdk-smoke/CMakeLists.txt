cmake_minimum_required(VERSION 3.22)
project(robot_bus_sdk_smoke LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if(NOT ROBOT_BUS_VERSION)
  message(FATAL_ERROR "Set -DROBOT_BUS_VERSION=<x.y.z> to the SDK release to fetch")
endif()

include("${CMAKE_CURRENT_LIST_DIR}/../cmake/FetchRobotBus.cmake")

set(sdk_args VERSION "${ROBOT_BUS_VERSION}")
if(ROBOT_BUS_ROS2)
  list(APPEND sdk_args ROS2 "${ROBOT_BUS_ROS2}")
endif()
if(ROBOT_BUS_SHA256)
  list(APPEND sdk_args SHA256 "${ROBOT_BUS_SHA256}")
endif()
robot_bus_fetch_sdk(${sdk_args})

add_executable(sdk_smoke main.cpp)
target_link_libraries(sdk_smoke PRIVATE robot_bus::robot_bus robot_bus::msgs)
if(ROBOT_BUS_ROS2)
  target_link_libraries(sdk_smoke PRIVATE robot_bus::ros2_bridge)
endif()
robot_bus_copy_runtime_libs(sdk_smoke)

enable_testing()
add_test(NAME sdk_smoke COMMAND sdk_smoke)
