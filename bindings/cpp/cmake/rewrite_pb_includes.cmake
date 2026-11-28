# Rewrite protoc C++ output so public includes use the robot_bus/ prefix.
# Invoked as: cmake -D ROBOT_BUS_GEN_ROOT=... -P rewrite_pb_includes.cmake
if(NOT DEFINED ROBOT_BUS_GEN_ROOT)
  message(FATAL_ERROR "ROBOT_BUS_GEN_ROOT is required")
endif()

set(_pkgs
  "ackermann_msgs|action_msgs|apriltag_msgs|builtin_interfaces|control_msgs"
  "diagnostic_msgs|example_interfaces|foxglove_msgs|geometry_msgs|lifecycle_msgs"
  "map_msgs|nav2_msgs|nav_msgs|robot_bus_interfaces|sensor_msgs|shape_msgs"
  "std_msgs|std_srvs|stereo_msgs|tf2_msgs|trajectory_msgs"
  "unique_identifier_msgs|vision_msgs|visualization_msgs")
string(REPLACE ";" "|" _pkgs "${_pkgs}")
set(_include_re "#include [\"<]((${_pkgs})/[^\">]+)[\">]")

file(GLOB_RECURSE _files
  "${ROBOT_BUS_GEN_ROOT}/*.pb.h"
  "${ROBOT_BUS_GEN_ROOT}/*.pb.cc")
foreach(_file IN LISTS _files)
  file(READ "${_file}" _text)
  string(REGEX REPLACE "${_include_re}" "#include <robot_bus/\\1>" _text "${_text}")
  if(_file MATCHES "\\.pb\\.h$")
    string(REPLACE
      "// @@protoc_insertion_point(includes)"
      "#include <robot_bus/detail/msgs_export.hpp>\n// @@protoc_insertion_point(includes)"
      _text "${_text}")
  endif()
  file(WRITE "${_file}" "${_text}")
endforeach()
