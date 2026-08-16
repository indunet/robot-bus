# Optional topic-shaped ROS message (same dual-definition idea as the .srv).
#
# Pair with a bus protobuf such as:
#   package my_pkg.msg.v1;
#   message Sum { int64 value = 1; }
#
# Not mounted by the runnable AddTwoInts demo — included so .msg / .srv /
# .action ↔ .proto pairing is obvious.

int64 value
