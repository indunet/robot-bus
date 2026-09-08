#pragma once

#include <atomic>
#include <chrono>
#include <cstdint>
#include <cstdio>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <string>
#include <vector>
#include <utility>

namespace robot_bus {

struct RpcStats {
  uint64_t calls = 0, failures = 0, timeouts = 0, cancelled = 0, rejected = 0;
  std::string last_error, last_status;
};

struct RouteHealth {
  std::atomic<uint64_t> rx{0};
  std::atomic<uint64_t> tx{0};
  std::atomic<uint64_t> convert_fail{0};
  std::atomic<uint64_t> decode_fail{0};
  std::atomic<uint64_t> publish_fail{0};
  std::atomic<uint64_t> last_rx_ms{0};
  std::atomic<uint64_t> last_warn_ms{0};
  std::atomic<bool> idle_latched{false};
  bool latched = false;
  mutable std::mutex rpc_mutex;
  RpcStats rpc;

  static uint64_t unix_ms() {
    return static_cast<uint64_t>(std::chrono::duration_cast<std::chrono::milliseconds>(
                                     std::chrono::system_clock::now().time_since_epoch())
                                     .count());
  }

  void record_rx() {
    rx.fetch_add(1, std::memory_order_relaxed);
    last_rx_ms.store(unix_ms(), std::memory_order_relaxed);
    idle_latched.store(false, std::memory_order_relaxed);
  }
  void record_tx() { tx.fetch_add(1, std::memory_order_relaxed); }
  void record_convert_fail() { convert_fail.fetch_add(1, std::memory_order_relaxed); }
  void record_decode_fail() { decode_fail.fetch_add(1, std::memory_order_relaxed); }
  void record_publish_fail() { publish_fail.fetch_add(1, std::memory_order_relaxed); }

  bool should_log_warn() {
    const uint64_t now = unix_ms();
    const uint64_t prev = last_warn_ms.load(std::memory_order_relaxed);
    if (prev != 0 && now - prev < 1000) {
      return false;
    }
    last_warn_ms.store(now, std::memory_order_relaxed);
    return true;
  }

  void rpc_start() {
    std::lock_guard<std::mutex> lock(rpc_mutex);
    ++rpc.calls;
    record_rx();
  }
  void rpc_finish(const std::string &status, const std::string &message = "") {
    std::lock_guard<std::mutex> lock(rpc_mutex);
    rpc.last_status = status;
    if (status == "succeeded") record_tx();
    else if (status == "cancelled") ++rpc.cancelled;
    else {
      ++rpc.failures;
      if (status == "timeout") ++rpc.timeouts;
      if (status == "rejected") ++rpc.rejected;
    }
    if (!message.empty()) rpc.last_error = message.substr(0, 512);
    const bool log_warn = !message.empty() && should_log_warn();
    if (log_warn) {
      std::fprintf(stderr, "ROS bridge RPC %s: %s\n", status.c_str(), message.c_str());
    }
  }
  RpcStats rpc_snapshot() const {
    std::lock_guard<std::mutex> lock(rpc_mutex);
    return rpc;
  }

  bool is_idle(bool enabled, bool grace_elapsed) const {
    const auto last = last_rx_ms.load(std::memory_order_relaxed);
    const auto now = unix_ms();
    return enabled && grace_elapsed && (last == 0 || (!latched && now >= last && now - last >= 15000));
  }

  bool take_idle_event(bool enabled, bool grace_elapsed) {
    if (!is_idle(enabled, grace_elapsed)) {
      idle_latched.store(false, std::memory_order_relaxed);
      return false;
    }
    return !idle_latched.exchange(true, std::memory_order_relaxed);
  }
};

class BridgeRpcError : public std::runtime_error {
 public:
  BridgeRpcError(std::string status, const std::string &message)
      : std::runtime_error(message), status(std::move(status)) {}
  std::string status;
};

inline std::string rpc_failure_status(const std::exception &error) {
  if (auto *rpc = dynamic_cast<const BridgeRpcError *>(&error)) return rpc->status;
  const std::string message = error.what();
  if (message.find("timed out") != std::string::npos || message.find("timeout") != std::string::npos) return "timeout";
  if (message.find("cancelled") != std::string::npos) return "cancelled";
  if (message.find("action rejected") != std::string::npos) return "rejected";
  if (message.find("action aborted") != std::string::npos) return "aborted";
  return "failed";
}

inline std::vector<uint8_t> bridge_rpc_error_body(const std::string &status, const std::string &message) {
  const std::string prefix = status == "timeout" ? "RPC_TIMEOUT" :
      status == "cancelled" ? "CANCELLED" : status == "rejected" ? "ACTION_REJECTED" :
      status == "aborted" ? "ACTION_ABORTED" : "RPC_FAILED";
  auto body = std::vector<uint8_t>(prefix.begin(), prefix.end());
  body.push_back(0);
  body.insert(body.end(), message.begin(), message.end());
  return body;
}

class RpcObservation {
 public:
  explicit RpcObservation(std::shared_ptr<RouteHealth> health) : health_(std::move(health)) {
    if (health_) health_->rpc_start();
  }
  ~RpcObservation() { if (!finished_) finish("failed", "RPC exited without a result"); }
  void finish(const std::string &status = "succeeded", const std::string &message = "") {
    if (finished_) return;
    finished_ = true;
    if (health_) health_->rpc_finish(status, message);
  }
 private:
  std::shared_ptr<RouteHealth> health_;
  bool finished_ = false;
};

}  // namespace robot_bus
