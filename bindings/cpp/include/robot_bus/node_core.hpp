#pragma once

#include <robot_bus.h>

#include <cstdint>
#include <cstring>
#include <functional>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <variant>
#include <vector>

namespace robot_bus {

/// Non-owning view of bytes (C++17; avoids requiring C++20 std::span).
struct BytesView {
  const uint8_t *data = nullptr;
  size_t size = 0;

  BytesView() = default;
  BytesView(const uint8_t *d, size_t n) : data(d), size(n) {}
  BytesView(const std::vector<uint8_t> &v) : data(v.data()), size(v.size()) {}
  BytesView(const std::string &s)
      : data(reinterpret_cast<const uint8_t *>(s.data())), size(s.size()) {}
  BytesView(std::string_view s)
      : data(reinterpret_cast<const uint8_t *>(s.data())), size(s.size()) {}
};

inline std::string last_error() {
  const char *e = robot_bus_last_error();
  return e ? std::string(e) : std::string();
}

class Error : public std::runtime_error {
 public:
  explicit Error(std::string msg) : std::runtime_error(std::move(msg)) {}
};

inline void check(int rc, const char *what) {
  if (rc < 0) {
    auto err = last_error();
    throw Error(std::string(what) + ": " + (err.empty() ? "unknown error" : err));
  }
}

inline void *check_ptr(void *p, const char *what) {
  if (!p) {
    auto err = last_error();
    throw Error(std::string(what) + ": " + (err.empty() ? "null" : err));
  }
  return p;
}

/// Local node parameter value (bool / int64 / double / string).
using ParameterValue = std::variant<bool, int64_t, double, std::string>;

struct Parameter {
  std::string name;
  ParameterValue value;
};

inline ParameterValue parameter_value_from_c(RobotBusParameterValue &v) {
  ParameterValue out;
  switch (v.type) {
    case ROBOT_BUS_PARAM_BOOL:
      out = v.bool_value != 0;
      break;
    case ROBOT_BUS_PARAM_INTEGER:
      out = v.integer_value;
      break;
    case ROBOT_BUS_PARAM_DOUBLE:
      out = v.double_value;
      break;
    case ROBOT_BUS_PARAM_STRING:
      out = v.string_value ? std::string(v.string_value) : std::string();
      robot_bus_free_string(v.string_value);
      v.string_value = nullptr;
      break;
    default:
      throw Error("unknown parameter type");
  }
  return out;
}

/// Shared ZeroMQ runtime context (required for same-process inproc).
class Context {
 public:
  Context() {
    c_ = static_cast<RobotBusContext *>(check_ptr(robot_bus_context_new(), "Context"));
  }

  explicit Context(RobotBusContext *raw) : c_(raw) {}

  ~Context() { robot_bus_context_free(c_); }

  Context(const Context &o) {
    c_ = static_cast<RobotBusContext *>(
        check_ptr(robot_bus_context_clone(o.c_), "Context::clone"));
  }

  Context &operator=(const Context &o) {
    if (this != &o) {
      RobotBusContext *next = static_cast<RobotBusContext *>(
          check_ptr(robot_bus_context_clone(o.c_), "Context::clone"));
      robot_bus_context_free(c_);
      c_ = next;
    }
    return *this;
  }

  Context(Context &&o) noexcept : c_(o.c_) { o.c_ = nullptr; }

  Context &operator=(Context &&o) noexcept {
    if (this != &o) {
      robot_bus_context_free(c_);
      c_ = o.c_;
      o.c_ = nullptr;
    }
    return *this;
  }

  RobotBusContext *raw() { return c_; }
  const RobotBusContext *raw() const { return c_; }

 private:
  RobotBusContext *c_ = nullptr;
};

inline uint8_t *alloc_reply_bytes(BytesView payload) {
  if (payload.size == 0) {
    return nullptr;
  }
  uint8_t *buf = robot_bus_alloc_bytes(payload.size);
  if (!buf) {
    throw Error("robot_bus_alloc_bytes failed");
  }
  std::memcpy(buf, payload.data, payload.size);
  return buf;
}

class OwnedString {
 public:
  explicit OwnedString(char *p) : p_(p) {}
  ~OwnedString() { robot_bus_free_string(p_); }
  OwnedString(const OwnedString &) = delete;
  OwnedString &operator=(const OwnedString &) = delete;
  OwnedString(OwnedString &&o) noexcept : p_(o.p_) { o.p_ = nullptr; }
  std::string str() const { return p_ ? std::string(p_) : std::string(); }

 private:
  char *p_;
};

}  // namespace robot_bus
