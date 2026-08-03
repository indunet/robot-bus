// TF buffer + listener smoke against an ephemeral in-process broker.
#include "harness.hpp"

#include <robot_bus/Tf.hpp>
#include <robot_bus/geometry_msgs/msg/v1/stamped.pb.h>
#include <robot_bus/tf2_msgs/msg/v1/tf_message.pb.h>

#include <iostream>
#include <string>

int main() {
  using namespace robot_bus::test;

  // Offline buffer (no broker).
  {
    robot_bus::TfBuffer buf;
    tf2_msgs::msg::v1::TFMessage msg;
    auto *t = msg.add_transforms();
    t->mutable_header()->set_frame_id("base_link");
    t->set_child_frame_id("camera");
    t->mutable_transform()->mutable_translation()->set_x(1.0);
    t->mutable_transform()->mutable_rotation()->set_w(1.0);
    std::string bytes;
    ROBOT_BUS_CHECK(msg.SerializeToString(&bytes));
    buf.set_transform_msg(bytes, true);
    ROBOT_BUS_CHECK(buf.can_transform("base_link", "camera"));
    auto stamped_bytes = buf.lookup_transform("base_link", "camera");
    geometry_msgs::msg::v1::TransformStamped stamped;
    ROBOT_BUS_CHECK(
        stamped.ParseFromArray(stamped_bytes.data(), static_cast<int>(stamped_bytes.size())));
    ROBOT_BUS_CHECK(stamped.child_frame_id() == "camera");
    ROBOT_BUS_CHECK(stamped.transform().translation().x() == 1.0);
  }

  // Listener over the bus.
  {
    auto bus = TestBus::start();
    auto node = bus.make_node("cpp-tf");
    robot_bus::TfListener listener(node);
    auto buf = listener.buffer();
    auto pub = node.create_publisher("/tf_static");
    robot_bus::TransformBroadcaster br(std::move(pub));

    node.start();
    sleep_ms(200);

    tf2_msgs::msg::v1::TFMessage msg;
    auto *t = msg.add_transforms();
    t->mutable_header()->set_frame_id("odom");
    t->set_child_frame_id("base_link");
    t->mutable_transform()->mutable_translation()->set_y(2.0);
    t->mutable_transform()->mutable_rotation()->set_w(1.0);
    std::string bytes;
    ROBOT_BUS_CHECK(msg.SerializeToString(&bytes));
    br.send(bytes);

    ROBOT_BUS_CHECK(wait_until([&] { return buf.can_transform("odom", "base_link"); }));
    auto stamped_bytes = buf.lookup_transform("odom", "base_link");
    geometry_msgs::msg::v1::TransformStamped stamped;
    ROBOT_BUS_CHECK(
        stamped.ParseFromArray(stamped_bytes.data(), static_cast<int>(stamped_bytes.size())));
    ROBOT_BUS_CHECK(stamped.transform().translation().y() == 2.0);

    node.shutdown();
    node.wait();
    bus.stop();
  }

  std::cout << "ok: tf_lookup\n";
  return 0;
}
