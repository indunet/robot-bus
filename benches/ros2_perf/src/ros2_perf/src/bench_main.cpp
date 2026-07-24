#include <algorithm>
#include <atomic>
#include <cctype>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <fstream>
#include <functional>
#include <iomanip>
#include <iostream>
#include <memory>
#include <mutex>
#include <numeric>
#include <sstream>
#include <string>
#include <thread>
#include <vector>

#include "rclcpp/rclcpp.hpp"
#include "rclcpp_action/rclcpp_action.hpp"
#include "std_msgs/msg/u_int8_multi_array.hpp"

#include "ros2_perf/action/echo.hpp"
#include "ros2_perf/srv/echo.hpp"

using namespace std::chrono_literals;

namespace {

constexpr size_t kPayloadLen = 64;
constexpr size_t kWarmup = 50;
constexpr size_t kDefaultMsgLatencySamples = 5000;
constexpr size_t kDefaultIters = 100000;
constexpr size_t kDefaultGoodputTrialMsgs = 0;  // 0 → duration × rate
constexpr double kDefaultGoodputTrialSecs = 1.0;
constexpr size_t kDefaultGoodputRateLo = 500;
constexpr size_t kDefaultGoodputRateHi = 500000;
constexpr size_t kDefaultGoodputSettleMs = 100;
constexpr double kDefaultMaxLossPct = 1.0;
constexpr size_t kMsgHistory = 2048;

struct LatencyStats {
  size_t count = 0;
  double p50_us = 0;
  double p95_us = 0;
  double p99_us = 0;
  double mean_us = 0;
};

struct ScenarioResult {
  std::string transport;
  std::string scenario;
  size_t sent = 0;
  size_t received = 0;
  double elapsed_s = 0;
  double publish_per_s = 0;
  double subscribe_per_s = 0;
  double delivery_pct = 0;
  LatencyStats latency;
  std::string note;
  bool is_message = false;
};

uint64_t now_ns()
{
  return static_cast<uint64_t>(
    std::chrono::duration_cast<std::chrono::nanoseconds>(
      std::chrono::system_clock::now().time_since_epoch())
      .count());
}

std::vector<uint8_t> make_payload(uint64_t ts_ns)
{
  std::vector<uint8_t> buf(kPayloadLen, 0);
  for (size_t i = 0; i < 8; ++i) {
    buf[i] = static_cast<uint8_t>((ts_ns >> (8 * i)) & 0xff);
  }
  return buf;
}

bool read_ts(const std::vector<uint8_t> & payload, uint64_t & out)
{
  if (payload.size() < 8) {
    return false;
  }
  uint64_t ts = 0;
  for (size_t i = 0; i < 8; ++i) {
    ts |= static_cast<uint64_t>(payload[i]) << (8 * i);
  }
  out = ts;
  return true;
}

double percentile_us(std::vector<uint64_t> & samples_ns, double p)
{
  if (samples_ns.empty()) {
    return 0.0;
  }
  std::sort(samples_ns.begin(), samples_ns.end());
  const size_t n = samples_ns.size();
  const size_t idx = static_cast<size_t>(std::round((n - 1) * p));
  return static_cast<double>(samples_ns[std::min(idx, n - 1)]) / 1000.0;
}

LatencyStats from_ns(std::vector<uint64_t> samples)
{
  LatencyStats s;
  if (samples.empty()) {
    return s;
  }
  s.count = samples.size();
  const double sum = std::accumulate(
    samples.begin(), samples.end(), 0.0,
    [](double a, uint64_t b) { return a + static_cast<double>(b); });
  s.mean_us = (sum / static_cast<double>(samples.size())) / 1000.0;
  auto copy = samples;
  s.p50_us = percentile_us(copy, 0.50);
  copy = samples;
  s.p95_us = percentile_us(copy, 0.95);
  copy = samples;
  s.p99_us = percentile_us(copy, 0.99);
  return s;
}

ScenarioResult make_ok_message(
  const std::string & transport, const std::string & scenario,
  size_t sent, size_t received, double elapsed_s, LatencyStats latency)
{
  ScenarioResult r;
  r.transport = transport;
  r.scenario = scenario;
  r.sent = sent;
  r.received = received;
  r.elapsed_s = elapsed_s;
  r.publish_per_s = elapsed_s > 0 ? sent / elapsed_s : 0;
  r.subscribe_per_s = elapsed_s > 0 ? received / elapsed_s : 0;
  r.delivery_pct = sent == 0 ? 0.0 : (100.0 * static_cast<double>(received) / static_cast<double>(sent));
  r.latency = std::move(latency);
  r.is_message = true;
  return r;
}

ScenarioResult make_ok_rpc(
  const std::string & transport, const std::string & scenario,
  size_t iterations, size_t received, double elapsed_s, LatencyStats latency)
{
  ScenarioResult r;
  r.transport = transport;
  r.scenario = scenario;
  r.sent = iterations;
  r.received = received;
  r.elapsed_s = elapsed_s;
  const double rate = elapsed_s > 0 ? received / elapsed_s : 0;
  r.publish_per_s = rate;
  r.subscribe_per_s = rate;
  r.delivery_pct = iterations == 0
    ? 0.0
    : (100.0 * static_cast<double>(received) / static_cast<double>(iterations));
  r.latency = std::move(latency);
  r.is_message = false;
  return r;
}

ScenarioResult make_skip(
  const std::string & transport, const std::string & scenario, std::string note)
{
  ScenarioResult r;
  r.transport = transport;
  r.scenario = scenario;
  r.note = std::move(note);
  return r;
}

rclcpp::QoS msg_qos()
{
  // Deep enough for paced goodput; best effort aligns with ZMQ PUB drop semantics.
  return rclcpp::QoS(rclcpp::KeepLast(kMsgHistory)).best_effort().durability_volatile();
}

bool wait_until(
  const std::function<bool()> & pred, std::chrono::milliseconds timeout)
{
  const auto deadline = std::chrono::steady_clock::now() + timeout;
  while (!pred()) {
    if (std::chrono::steady_clock::now() >= deadline) {
      return false;
    }
    std::this_thread::sleep_for(1ms);
  }
  return true;
}

size_t env_size(const char * name, size_t fallback)
{
  const char * v = std::getenv(name);
  if (!v || !*v) {
    return fallback;
  }
  return static_cast<size_t>(std::strtoull(v, nullptr, 10));
}

double env_double(const char * name, double fallback)
{
  const char * v = std::getenv(name);
  if (!v || !*v) {
    return fallback;
  }
  return std::strtod(v, nullptr);
}

double loss_pct(size_t sent, size_t received)
{
  if (sent == 0) {
    return 100.0;
  }
  if (received >= sent) {
    return 0.0;
  }
  return 100.0 * static_cast<double>(sent - received) / static_cast<double>(sent);
}

size_t trial_msg_count(size_t rate_hz)
{
  const size_t fixed = env_size("ROS2_PERF_GOODPUT_TRIAL_MSGS", kDefaultGoodputTrialMsgs);
  if (fixed > 0) {
    return fixed;
  }
  const double secs = env_double("ROS2_PERF_GOODPUT_TRIAL_SECS", kDefaultGoodputTrialSecs);
  const size_t n = static_cast<size_t>(std::llround(static_cast<double>(rate_hz) * secs));
  return std::clamp(n, static_cast<size_t>(1000), static_cast<size_t>(50000));
}

class PubSubBench
{
public:
  explicit PubSubBench(const std::string & transport)
  : transport_(transport)
  {
    sub_node_ = std::make_shared<rclcpp::Node>("ros2_perf_sub_" + transport_);
    pub_node_ = std::make_shared<rclcpp::Node>("ros2_perf_pub_" + transport_);
    const std::string topic = "/ros2_perf/" + transport_ + "/msg";

    sub_ = sub_node_->create_subscription<std_msgs::msg::UInt8MultiArray>(
      topic, msg_qos(),
      [this](const std_msgs::msg::UInt8MultiArray::SharedPtr msg) {
        if (record_latency_.load()) {
          uint64_t sent = 0;
          if (read_ts(msg->data, sent)) {
            const uint64_t now = now_ns();
            if (now >= sent) {
              std::lock_guard<std::mutex> lock(mu_);
              latencies_.push_back(now - sent);
            }
          }
        }
        count_.fetch_add(1, std::memory_order_relaxed);
      });

    pub_ = pub_node_->create_publisher<std_msgs::msg::UInt8MultiArray>(topic, msg_qos());
  }

  std::vector<rclcpp::Node::SharedPtr> nodes() const { return {sub_node_, pub_node_}; }

  ScenarioResult run()
  {
    const std::string scenario = "message pub/sub";
    if (!wait_until(
        [this] { return pub_->get_subscription_count() > 0; }, 10s))
    {
      return make_skip(transport_, scenario, "no subscriber matched");
    }
    std::this_thread::sleep_for(200ms);

    for (size_t i = 0; i < kWarmup; ++i) {
      publish_one(now_ns());
    }
    std::this_thread::sleep_for(100ms);
    count_.store(0);
    {
      std::lock_guard<std::mutex> lock(mu_);
      latencies_.clear();
    }

    const size_t latency_samples = env_size("ROS2_PERF_MSG_LATENCY_SAMPLES", kDefaultMsgLatencySamples);

    // Phase 1: paced one-way latency.
    record_latency_.store(true);
    std::cout << "  … latency samples: " << latency_samples << std::endl;
    for (size_t i = 0; i < latency_samples; ++i) {
      const size_t before = count_.load();
      publish_one(now_ns());
      if (!wait_until([this, before] { return count_.load() >= before + 1; }, 2s)) {
        return make_skip(transport_, scenario, "latency sample timed out");
      }
    }
    LatencyStats latency;
    {
      std::lock_guard<std::mutex> lock(mu_);
      latency = from_ns(latencies_);
    }

    // Phase 2: binary-search max goodput at loss ≤ threshold.
    record_latency_.store(false);
    const double max_loss = env_double("ROS2_PERF_MAX_LOSS_PCT", kDefaultMaxLossPct);
    const size_t settle_ms = env_size("ROS2_PERF_GOODPUT_SETTLE_MS", kDefaultGoodputSettleMs);
    size_t lo = env_size("ROS2_PERF_GOODPUT_RATE_LO", kDefaultGoodputRateLo);
    size_t hi = env_size("ROS2_PERF_GOODPUT_RATE_HI", kDefaultGoodputRateHi);
    if (hi < lo) {
      hi = lo;
    }

    std::cout << "  … max goodput: binary search " << lo << "..=" << hi
              << " Hz, loss≤" << std::fixed << std::setprecision(1) << max_loss
              << "%, trial≈" << env_double("ROS2_PERF_GOODPUT_TRIAL_SECS", kDefaultGoodputTrialSecs)
              << "s (KeepLast=" << kMsgHistory
              << ", best_effort)" << std::endl;

    bool have_best = false;
    size_t best_sent = 0;
    size_t best_recv = 0;
    double best_elapsed = 0;
    size_t best_target = 0;

    while (lo <= hi) {
      const size_t mid = lo + (hi - lo) / 2;
      count_.store(0);
      const size_t trial_msgs = trial_msg_count(mid);
      const double interval_s = 1.0 / static_cast<double>(std::max<size_t>(mid, 1));
      const auto t0 = std::chrono::steady_clock::now();
      auto next = t0;
      size_t sent = 0;
      for (size_t i = 0; i < trial_msgs; ++i) {
        publish_one(now_ns());
        ++sent;
        next += std::chrono::duration_cast<std::chrono::steady_clock::duration>(
          std::chrono::duration<double>(interval_s));
        // Coarse sleep + busy-wait (sleep alone is too coarse for multi-kHz).
        for (;;) {
          const auto now = std::chrono::steady_clock::now();
          if (now >= next) {
            break;
          }
          const auto remain = next - now;
          if (remain > 2ms) {
            std::this_thread::sleep_for(remain - 1ms);
          }
        }
      }
      const auto t_send_end = std::chrono::steady_clock::now();
      std::this_thread::sleep_for(std::chrono::milliseconds(settle_ms));
      const double elapsed =
        std::max(std::chrono::duration<double>(t_send_end - t0).count(), 1e-9);
      const size_t received = count_.load();
      const double loss = loss_pct(sent, received);
      const double actual_pub = sent / elapsed;
      const bool sustained = actual_pub >= 0.90 * static_cast<double>(mid);
      std::cout << "  …   try " << mid << " Hz (" << trial_msgs << " msgs) → sent=" << sent
                << " recv=" << received
                << " loss=" << std::setprecision(2) << loss << "% pub="
                << std::setprecision(0) << actual_pub
                << "/s sub=" << (received / elapsed) << "/s sustained="
                << (sustained ? "true" : "false") << std::endl;

      if (received > 0 && loss <= max_loss && sustained) {
        have_best = true;
        best_sent = sent;
        best_recv = received;
        best_elapsed = elapsed;
        best_target = mid;
        lo = mid + 1;
      } else if (mid == 0) {
        break;
      } else {
        hi = mid - 1;
      }
    }

    if (!have_best) {
      return make_skip(
        transport_, scenario,
        "no rate met loss≤" + std::to_string(static_cast<int>(max_loss)) + "%");
    }

    std::cout << "  … max goodput ≈ " << best_target << " Hz target (loss≤"
              << std::setprecision(1) << max_loss << "%)" << std::endl;
    return make_ok_message(transport_, scenario, best_sent, best_recv, best_elapsed, latency);
  }

private:
  void publish_one(uint64_t ts)
  {
    std_msgs::msg::UInt8MultiArray msg;
    msg.data = make_payload(ts);
    pub_->publish(msg);
  }

  std::string transport_;
  rclcpp::Node::SharedPtr sub_node_;
  rclcpp::Node::SharedPtr pub_node_;
  rclcpp::Subscription<std_msgs::msg::UInt8MultiArray>::SharedPtr sub_;
  rclcpp::Publisher<std_msgs::msg::UInt8MultiArray>::SharedPtr pub_;
  std::atomic<size_t> count_{0};
  std::atomic<bool> record_latency_{true};
  std::mutex mu_;
  std::vector<uint64_t> latencies_;
};

class ServiceBench
{
public:
  explicit ServiceBench(const std::string & transport)
  : transport_(transport)
  {
    server_node_ = std::make_shared<rclcpp::Node>("ros2_perf_svc_srv_" + transport_);
    client_node_ = std::make_shared<rclcpp::Node>("ros2_perf_svc_cli_" + transport_);
    const std::string name = "/ros2_perf/" + transport_ + "/echo";

    server_ = server_node_->create_service<ros2_perf::srv::Echo>(
      name,
      [](const std::shared_ptr<ros2_perf::srv::Echo::Request> req,
        std::shared_ptr<ros2_perf::srv::Echo::Response> res) {
        res->data = req->data;
      });

    client_ = client_node_->create_client<ros2_perf::srv::Echo>(name);
  }

  std::vector<rclcpp::Node::SharedPtr> nodes() const
  {
    return {server_node_, client_node_};
  }

  ScenarioResult run(size_t n)
  {
    const std::string scenario = "service call";
    if (!client_->wait_for_service(10s)) {
      return make_skip(transport_, scenario, "service not available");
    }

    auto payload = make_payload(0);
    auto probe = std::make_shared<ros2_perf::srv::Echo::Request>();
    probe->data = payload;
    auto probe_fut = client_->async_send_request(probe);
    if (probe_fut.wait_for(2s) != std::future_status::ready) {
      return make_skip(transport_, scenario, "probe call timed out");
    }
    (void)probe_fut.get();

    for (size_t i = 1; i < kWarmup; ++i) {
      auto req = std::make_shared<ros2_perf::srv::Echo::Request>();
      req->data = payload;
      auto fut = client_->async_send_request(req);
      if (fut.wait_for(2s) != std::future_status::ready) {
        return make_skip(transport_, scenario, "warmup call timed out");
      }
      (void)fut.get();
    }

    std::vector<uint64_t> samples;
    samples.reserve(n);
    size_t received = 0;
    const auto t0 = std::chrono::steady_clock::now();
    for (size_t i = 0; i < n; ++i) {
      auto req = std::make_shared<ros2_perf::srv::Echo::Request>();
      req->data = payload;
      const auto start = std::chrono::steady_clock::now();
      auto fut = client_->async_send_request(req);
      if (fut.wait_for(5s) != std::future_status::ready) {
        if (received == 0) {
          return make_skip(transport_, scenario, "call timed out");
        }
        break;
      }
      (void)fut.get();
      samples.push_back(
        static_cast<uint64_t>(
          std::chrono::duration_cast<std::chrono::nanoseconds>(
            std::chrono::steady_clock::now() - start)
            .count()));
      ++received;
    }
    const double elapsed =
      std::chrono::duration<double>(std::chrono::steady_clock::now() - t0).count();
    if (received == 0) {
      return make_skip(transport_, scenario, "0 successful calls");
    }
    return make_ok_rpc(transport_, scenario, n, received, elapsed, from_ns(std::move(samples)));
  }

private:
  std::string transport_;
  rclcpp::Node::SharedPtr server_node_;
  rclcpp::Node::SharedPtr client_node_;
  rclcpp::Service<ros2_perf::srv::Echo>::SharedPtr server_;
  rclcpp::Client<ros2_perf::srv::Echo>::SharedPtr client_;
};

class ActionBench
{
public:
  using Echo = ros2_perf::action::Echo;
  using GoalHandle = rclcpp_action::ServerGoalHandle<Echo>;

  explicit ActionBench(const std::string & transport)
  : transport_(transport)
  {
    server_node_ = std::make_shared<rclcpp::Node>("ros2_perf_act_srv_" + transport_);
    client_node_ = std::make_shared<rclcpp::Node>("ros2_perf_act_cli_" + transport_);
    const std::string name = "/ros2_perf/" + transport_ + "/act";

    server_ = rclcpp_action::create_server<Echo>(
      server_node_,
      name,
      [](const rclcpp_action::GoalUUID &, std::shared_ptr<const Echo::Goal>) {
        return rclcpp_action::GoalResponse::ACCEPT_AND_EXECUTE;
      },
      [](const std::shared_ptr<GoalHandle>) {
        return rclcpp_action::CancelResponse::REJECT;
      },
      [](const std::shared_ptr<GoalHandle> goal_handle) {
        // Detach so succeed() does not run on the same executor worker the
        // client is blocked waiting on (avoids single-thread deadlocks).
        std::thread([goal_handle]() {
          const auto goal = goal_handle->get_goal();
          auto result = std::make_shared<Echo::Result>();
          result->data = goal->data;
          goal_handle->succeed(result);
        }).detach();
      });

    client_ = rclcpp_action::create_client<Echo>(client_node_, name);
  }

  std::vector<rclcpp::Node::SharedPtr> nodes() const
  {
    return {server_node_, client_node_};
  }

  ScenarioResult run(size_t n)
  {
    const std::string scenario = "action send_goal";
    if (!client_->wait_for_action_server(10s)) {
      return make_skip(transport_, scenario, "action server not available");
    }

    auto payload = make_payload(0);
    if (!send_one(payload, 2s)) {
      return make_skip(transport_, scenario, "probe send_goal failed");
    }
    for (size_t i = 1; i < std::min(kWarmup, size_t{10}); ++i) {
      if (!send_one(payload, 2s)) {
        return make_skip(transport_, scenario, "warmup send_goal failed");
      }
    }

    std::vector<uint64_t> samples;
    samples.reserve(n);
    size_t received = 0;
    const auto t0 = std::chrono::steady_clock::now();
    for (size_t i = 0; i < n; ++i) {
      const auto start = std::chrono::steady_clock::now();
      if (!send_one(payload, 5s)) {
        if (received == 0) {
          return make_skip(transport_, scenario, "send_goal failed");
        }
        break;
      }
      samples.push_back(
        static_cast<uint64_t>(
          std::chrono::duration_cast<std::chrono::nanoseconds>(
            std::chrono::steady_clock::now() - start)
            .count()));
      ++received;
    }
    const double elapsed =
      std::chrono::duration<double>(std::chrono::steady_clock::now() - t0).count();
    if (received == 0) {
      return make_skip(transport_, scenario, "0 successful goals");
    }
    return make_ok_rpc(transport_, scenario, n, received, elapsed, from_ns(std::move(samples)));
  }

private:
  bool send_one(const std::vector<uint8_t> & payload, std::chrono::seconds timeout)
  {
    auto goal = Echo::Goal();
    goal.data = payload;
    auto future_gh = client_->async_send_goal(goal);
    if (future_gh.wait_for(timeout) != std::future_status::ready) {
      return false;
    }
    auto goal_handle = future_gh.get();
    if (!goal_handle) {
      return false;
    }
    auto future_result = client_->async_get_result(goal_handle);
    if (future_result.wait_for(timeout) != std::future_status::ready) {
      return false;
    }
    auto wrapped = future_result.get();
    return wrapped.code == rclcpp_action::ResultCode::SUCCEEDED;
  }

  std::string transport_;
  rclcpp::Node::SharedPtr server_node_;
  rclcpp::Node::SharedPtr client_node_;
  rclcpp_action::Server<Echo>::SharedPtr server_;
  rclcpp_action::Client<Echo>::SharedPtr client_;
};

class SpinExecutor
{
public:
  void add(const std::vector<rclcpp::Node::SharedPtr> & nodes)
  {
    for (const auto & n : nodes) {
      exec_.add_node(n);
    }
  }

  void start()
  {
    thread_ = std::thread([this] { exec_.spin(); });
  }

  void stop()
  {
    exec_.cancel();
    if (thread_.joinable()) {
      thread_.join();
    }
  }

private:
  rclcpp::executors::MultiThreadedExecutor exec_;
  std::thread thread_;
};

std::string env_summary()
{
  std::ostringstream os;
  os << "- ROS: Humble (rmw_fastrtps_cpp)\n";
  const char * mode = std::getenv("ROS2_PERF_MODE");
  os << "- Mode: " << (mode ? mode : "unknown") << "\n";
  const char * profiles = std::getenv("FASTRTPS_DEFAULT_PROFILES_FILE");
  if (profiles) {
    os << "- Fast DDS profile: `" << profiles << "`\n";
  }
  os << "- Payload: 64 bytes\n";
  os << "- Message max loss / trial / rate range: "
     << env_double("ROS2_PERF_MAX_LOSS_PCT", kDefaultMaxLossPct) << "% / ~"
     << env_double("ROS2_PERF_GOODPUT_TRIAL_SECS", kDefaultGoodputTrialSecs) << "s / "
     << env_size("ROS2_PERF_GOODPUT_RATE_LO", kDefaultGoodputRateLo) << ".."
     << env_size("ROS2_PERF_GOODPUT_RATE_HI", kDefaultGoodputRateHi)
     << " Hz (KeepLast(" << kMsgHistory << ") best_effort)\n";
  os << "- Message latency samples: "
     << env_size("ROS2_PERF_MSG_LATENCY_SAMPLES", kDefaultMsgLatencySamples)
     << " (paced)\n";
  os << "- Service/action iterations: "
     << env_size("ROS2_PERF_SVC_ITERS", kDefaultIters) << " / "
     << env_size("ROS2_PERF_ACT_ITERS", kDefaultIters) << "\n";
  return os.str();
}

void print_result(const ScenarioResult & r)
{
  if (!r.note.empty()) {
    std::cout << "[" << r.transport << "/" << r.scenario << "] SKIP: " << r.note << "\n";
    return;
  }
  if (r.is_message) {
    std::cout << "[" << r.transport << "/" << r.scenario << "] sent=" << r.sent
              << " recv=" << r.received << " pub=" << std::fixed << std::setprecision(0)
              << r.publish_per_s << "/s sub=" << r.subscribe_per_s
              << "/s delivery=" << std::setprecision(1) << r.delivery_pct
              << "% p50=" << std::setprecision(0) << r.latency.p50_us
              << "µs p99=" << r.latency.p99_us << "µs\n";
    return;
  }
  std::cout << "[" << r.transport << "/" << r.scenario << "] n=" << r.sent
            << " got=" << r.received << " " << std::fixed << std::setprecision(0)
            << r.subscribe_per_s << "/s p50=" << r.latency.p50_us
            << "µs p99=" << r.latency.p99_us << "µs\n";
}

std::string cell_sub(const std::vector<ScenarioResult> & results,
  const std::string & transport, const std::string & scenario)
{
  for (const auto & r : results) {
    if (r.transport == transport && r.scenario == scenario) {
      if (!r.note.empty()) {
        return "—";
      }
      std::ostringstream os;
      if (r.is_message) {
        os << std::fixed << std::setprecision(0) << r.subscribe_per_s << "/s ("
           << std::setprecision(1) << r.delivery_pct << "% delivered)";
      } else {
        os << std::fixed << std::setprecision(0) << r.subscribe_per_s << "/s";
      }
      return os.str();
    }
  }
  return "—";
}

std::string cell_pub(const std::vector<ScenarioResult> & results,
  const std::string & transport, const std::string & scenario)
{
  for (const auto & r : results) {
    if (r.transport == transport && r.scenario == scenario) {
      if (!r.note.empty()) {
        return "—";
      }
      std::ostringstream os;
      os << std::fixed << std::setprecision(0) << r.publish_per_s << "/s";
      return os.str();
    }
  }
  return "—";
}

void write_section(
  std::ostringstream & md, const std::string & title,
  const std::vector<ScenarioResult> & results, const std::string & transport)
{
  md << "## " << title << "\n\n";
  md << "| 场景 | 发送 | 接收 | 耗时 | 发布/s | 订阅/s | 投递% | p50 (µs) | p95 (µs) | p99 (µs) | mean (µs) |\n";
  md << "|------|------|------|------|--------|--------|-------|----------|----------|----------|-----------|\n";
  for (const auto & r : results) {
    if (r.transport != transport) {
      continue;
    }
    if (!r.note.empty()) {
      md << "| " << r.scenario << " | — | — | — | SKIP: " << r.note << " | — | — | — | — | — | — |\n";
      continue;
    }
    md << "| " << r.scenario
       << " | " << r.sent
       << " | " << r.received
       << " | " << std::fixed << std::setprecision(3) << r.elapsed_s << "s"
       << " | " << std::setprecision(0) << r.publish_per_s
       << " | " << r.subscribe_per_s
       << " | " << std::setprecision(1) << r.delivery_pct
       << " | " << std::setprecision(0) << r.latency.p50_us
       << " | " << r.latency.p95_us
       << " | " << r.latency.p99_us
       << " | " << r.latency.mean_us
       << " |\n";
  }
  md << "\n";
}

void write_report(const std::vector<ScenarioResult> & results, const std::string & out_path)
{
  std::ostringstream md;
  md << "# ROS 2 性能测试报告\n\n";
  md << "由 `benches/ros2_perf/run.sh`（容器内 `ros2_perf_bench`）生成，方法对齐 `docs/perf-report.md`。\n\n";
  md << "## 环境\n\n";
  md << env_summary() << "\n";
  md << "## 方法\n\n";
  md << "- RMW: `rmw_fastrtps_cpp`；传输由 Fast DDS XML 固定为 **SHM** 或 **UDPv4**。\n";
  md << "- 单进程多 Node + `MultiThreadedExecutor`（本机回环，非跨机）。\n";
  md << "- Payload：64 字节；QoS `KeepLast(" << kMsgHistory << ")` **best_effort**（对齐 ZMQ PUB 不可靠语义）。\n";
  md << "- Message **吞吐（主指标）**：在目标速率下限速发送，**二分搜索**丢包率 ≤ "
     << env_double("ROS2_PERF_MAX_LOSS_PCT", kDefaultMaxLossPct)
     << "% 的最大可持续速率（max goodput）；每档约 "
     << env_double("ROS2_PERF_GOODPUT_TRIAL_SECS", kDefaultGoodputTrialSecs) << "s。\n";
  md << "- Message **延迟**：另做限速抽样（默认 " << kDefaultMsgLatencySamples << "）。\n";
  md << "- Service / action 延迟：每次 call / send_goal 本地计时。\n";
  md << "- 指标机器相关，不作为 CI 门槛。\n\n";

  md << "## 横比\n\n";
  md << "message 为 **max goodput**（丢包阈值内的最大可持续订阅速率）；括号为该档实测投递率。另附发布速率行。\n\n";
  md << "| 场景 | shm | udp |\n";
  md << "|------|-----|-----|\n";
  md << "| message 发布 | " << cell_pub(results, "shm", "message pub/sub")
     << " | " << cell_pub(results, "udp", "message pub/sub") << " |\n";
  md << "| message max goodput | " << cell_sub(results, "shm", "message pub/sub")
     << " | " << cell_sub(results, "udp", "message pub/sub") << " |\n";
  md << "| service call | " << cell_sub(results, "shm", "service call")
     << " | " << cell_sub(results, "udp", "service call") << " |\n";
  md << "| action send_goal | " << cell_sub(results, "shm", "action send_goal")
     << " | " << cell_sub(results, "udp", "action send_goal") << " |\n\n";

  write_section(md, "shm（Fast DDS Shared Memory）", results, "shm");
  write_section(md, "udp（Fast DDS UDPv4，无 SHM）", results, "udp");

  md << "## 复现\n\n";
  md << "```bash\n";
  md << "./benches/ros2_perf/run.sh\n";
  md << "ROS2_PERF_ONLY=message ./benches/ros2_perf/run.sh   # 仅 message\n";
  md << "```\n";

  std::ofstream f(out_path);
  f << md.str();
  std::cout << "wrote " << out_path << "\n";
}

}  // namespace

int main(int argc, char ** argv)
{
  rclcpp::init(argc, argv);

  const std::string mode = [] {
    const char * v = std::getenv("ROS2_PERF_MODE");
    return v ? std::string(v) : std::string("shm");
  }();
  const bool only_message = [] {
    const char * v = std::getenv("ROS2_PERF_ONLY");
    if (!v) {
      return false;
    }
    std::string s(v);
    for (char & c : s) {
      c = static_cast<char>(std::tolower(static_cast<unsigned char>(c)));
    }
    return s == "message" || s == "msg" || s == "pubsub";
  }();
  const size_t svc_iters = env_size("ROS2_PERF_SVC_ITERS", kDefaultIters);
  const size_t act_iters = env_size("ROS2_PERF_ACT_ITERS", kDefaultIters);
  const char * report_path_env = std::getenv("ROS2_PERF_REPORT");
  const std::string report_path =
    report_path_env ? report_path_env : "docs/ros2-perf-report.md";

  std::cout << "=== ROS2 perf mode=" << mode
            << " goodput loss≤" << env_double("ROS2_PERF_MAX_LOSS_PCT", kDefaultMaxLossPct) << "%"
            << " svc=" << svc_iters << " act=" << act_iters
            << (only_message ? " (message only)" : "")
            << " ===\n";

  std::vector<ScenarioResult> results;

  {
    std::cout << "--- message pub/sub ---\n";
    PubSubBench bench(mode);
    SpinExecutor spin;
    spin.add(bench.nodes());
    spin.start();
    auto r = bench.run();
    spin.stop();
    print_result(r);
    results.push_back(std::move(r));
  }

  if (!only_message) {
    {
      std::cout << "--- service call ---\n";
      ServiceBench bench(mode);
      SpinExecutor spin;
      spin.add(bench.nodes());
      spin.start();
      auto r = bench.run(svc_iters);
      spin.stop();
      print_result(r);
      results.push_back(std::move(r));
    }

    {
      std::cout << "--- action send_goal ---\n";
      ActionBench bench(mode);
      SpinExecutor spin;
      spin.add(bench.nodes());
      spin.start();
      auto r = bench.run(act_iters);
      spin.stop();
      print_result(r);
      results.push_back(std::move(r));
    }
  }

  // Append mode results into an existing combined report if ROS2_PERF_MERGE=1.
  const char * merge = std::getenv("ROS2_PERF_MERGE");
  if (merge && std::string(merge) == "1") {
    // Caller (run.sh) merges; just write a side JSON-ish lines file.
    const std::string partial = report_path + "." + mode + ".partial.md";
    write_report(results, partial);
  } else {
    write_report(results, report_path);
  }

  rclcpp::shutdown();
  return 0;
}
