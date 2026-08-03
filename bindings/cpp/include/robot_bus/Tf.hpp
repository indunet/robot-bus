#pragma once

#include <robot_bus.h>

#include <robot_bus/Node.hpp>

#include <sstream>
#include <string>
#include <utility>
#include <vector>

namespace robot_bus {

/// In-memory TF tree (`robot_bus::tf::Buffer`). Protobuf at the boundary.
class TfBuffer {
 public:
  TfBuffer() {
    buf_ = static_cast<RobotBusTfBuffer *>(check_ptr(robot_bus_tf_buffer_new(), "TfBuffer"));
  }

  /// Take ownership of an existing C handle (e.g. from `TfListener::buffer()`).
  explicit TfBuffer(RobotBusTfBuffer *raw) : buf_(raw) {}

  ~TfBuffer() { robot_bus_tf_buffer_free(buf_); }

  TfBuffer(const TfBuffer &) = delete;
  TfBuffer &operator=(const TfBuffer &) = delete;

  TfBuffer(TfBuffer &&o) noexcept : buf_(o.buf_) { o.buf_ = nullptr; }

  TfBuffer &operator=(TfBuffer &&o) noexcept {
    if (this != &o) {
      robot_bus_tf_buffer_free(buf_);
      buf_ = o.buf_;
      o.buf_ = nullptr;
    }
    return *this;
  }

  void clear() { check(robot_bus_tf_buffer_clear(buf_), "TfBuffer::clear"); }

  /// Ingest a `tf2_msgs/TFMessage` protobuf. `is_static` marks `/tf_static` traffic.
  void set_transform_msg(BytesView tf_message, bool is_static) {
    check(robot_bus_tf_buffer_set_transform_msg(buf_, tf_message.data, tf_message.size,
                                                is_static ? 1 : 0),
          "TfBuffer::set_transform_msg");
  }

  /// Lookup transform of `source` relative to `target` as `TransformStamped` bytes.
  std::vector<uint8_t> lookup_transform(const char *target, const char *source) {
    uint8_t *out = nullptr;
    size_t len = 0;
    check(robot_bus_tf_buffer_lookup_transform(buf_, target, source, &out, &len),
          "TfBuffer::lookup_transform");
    std::vector<uint8_t> result(out, out + len);
    robot_bus_free_bytes(out, len);
    return result;
  }

  bool can_transform(const char *target, const char *source) const {
    return robot_bus_tf_buffer_can_transform(buf_, target, source) != 0;
  }

  std::vector<std::string> frames() const {
    OwnedString s(robot_bus_tf_buffer_frames(buf_));
    std::string joined = s.str();
    std::vector<std::string> out;
    if (joined.empty()) {
      return out;
    }
    std::istringstream iss(joined);
    std::string line;
    while (std::getline(iss, line)) {
      if (!line.empty()) {
        out.push_back(std::move(line));
      }
    }
    return out;
  }

  RobotBusTfBuffer *raw() { return buf_; }
  const RobotBusTfBuffer *raw() const { return buf_; }

 private:
  RobotBusTfBuffer *buf_ = nullptr;
};

/// Subscribes `/tf` + `/tf_static` (or custom topics) into a shared buffer.
class TfListener {
 public:
  explicit TfListener(Node &node) {
    listener_ = static_cast<RobotBusTfListener *>(
        check_ptr(robot_bus_tf_listener_with_defaults(node.raw()), "TfListener"));
  }

  TfListener(Node &node, const char *tf_topic, const char *tf_static_topic) {
    listener_ = static_cast<RobotBusTfListener *>(check_ptr(
        robot_bus_tf_listener_new(node.raw(), tf_topic, tf_static_topic), "TfListener"));
  }

  ~TfListener() { robot_bus_tf_listener_free(listener_); }

  TfListener(const TfListener &) = delete;
  TfListener &operator=(const TfListener &) = delete;

  TfListener(TfListener &&o) noexcept : listener_(o.listener_) { o.listener_ = nullptr; }

  TfListener &operator=(TfListener &&o) noexcept {
    if (this != &o) {
      robot_bus_tf_listener_free(listener_);
      listener_ = o.listener_;
      o.listener_ = nullptr;
    }
    return *this;
  }

  /// Shared buffer handle (Arc clone). Safe to keep after listener moves.
  TfBuffer buffer() {
    return TfBuffer(static_cast<RobotBusTfBuffer *>(
        check_ptr(robot_bus_tf_listener_buffer(listener_), "TfListener::buffer")));
  }

 private:
  RobotBusTfListener *listener_ = nullptr;
};

/// Thin helper over a typed `TopicPublisher` of `tf2_msgs/TFMessage` bytes.
class TransformBroadcaster {
 public:
  explicit TransformBroadcaster(TopicPublisher pub) : pub_(std::move(pub)) {}

  void send(BytesView tf_message) { pub_.publish(tf_message); }

 private:
  TopicPublisher pub_;
};

}  // namespace robot_bus
