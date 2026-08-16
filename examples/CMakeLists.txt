# C++ demos under examples/*/cpp are built via bindings/cpp:
#
#   cmake -S bindings/cpp -B bindings/cpp/build -DROBOT_BUS_BUILD_EXAMPLES=ON
#   cmake --build bindings/cpp/build --target cpp_examples
#
# Or: just examples-cpp
#
# Binaries land in examples/build/.
cmake_minimum_required(VERSION 3.16)
message(FATAL_ERROR
  "Do not configure this directory directly. Use `just examples-cpp` "
  "(bindings/cpp with -DROBOT_BUS_BUILD_EXAMPLES=ON).")
