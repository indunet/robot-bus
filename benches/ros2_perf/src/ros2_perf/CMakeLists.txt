cmake_minimum_required(VERSION 3.8)
project(ros2_perf)

if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  add_compile_options(-Wall -Wextra -Wpedantic -O3)
endif()

find_package(ament_cmake REQUIRED)
find_package(rclcpp REQUIRED)
find_package(rclcpp_action REQUIRED)
find_package(std_msgs REQUIRED)
find_package(rosidl_default_generators REQUIRED)

rosidl_generate_interfaces(${PROJECT_NAME}
  "srv/Echo.srv"
  "action/Echo.action"
)

add_executable(ros2_perf_bench src/bench_main.cpp)
ament_target_dependencies(ros2_perf_bench
  rclcpp
  rclcpp_action
  std_msgs
)
rosidl_get_typesupport_target(cpp_typesupport_target ${PROJECT_NAME} "rosidl_typesupport_cpp")
target_link_libraries(ros2_perf_bench "${cpp_typesupport_target}")

install(TARGETS ros2_perf_bench DESTINATION lib/${PROJECT_NAME})

ament_package()