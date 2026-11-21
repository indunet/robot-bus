if(NOT DEFINED VERSION OR VERSION STREQUAL "")
  message(FATAL_ERROR "VERSION is required")
endif()
if(NOT DEFINED OUT OR OUT STREQUAL "")
  message(FATAL_ERROR "OUT is required")
endif()
include(CMakePackageConfigHelpers)
write_basic_package_version_file(
  "${OUT}"
  VERSION "${VERSION}"
  COMPATIBILITY SameMajorVersion
  ARCH_INDEPENDENT)
