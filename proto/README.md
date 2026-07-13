# ROS 2 Protobuf Messages

Protobuf 重定义 ROS 2 常用标准消息，字段语义与类型尽量贴近
[ros2/common_interfaces](https://github.com/ros2/common_interfaces) 与
[ros2/rcl_interfaces](https://github.com/ros2/rcl_interfaces) 中的 `.msg` 定义。

## 目录

```
proto/
├── builtin_interfaces/v1/   Time, Duration
├── std_msgs/v1/             Header, primitives, MultiArray
├── geometry_msgs/v1/        Point, Pose, Twist, Transform, ...
├── sensor_msgs/v1/          Imu, Image, NavSatFix, LaserScan, ...
└── nav_msgs/v1/             Odometry
```

## 与 ROS 2 的对应关系

| Protobuf package | ROS 2 package |
|------------------|---------------|
| `builtin_interfaces.v1` | `builtin_interfaces` |
| `std_msgs.v1` | `std_msgs` |
| `geometry_msgs.v1` | `geometry_msgs` |
| `sensor_msgs.v1` | `sensor_msgs` |
| `nav_msgs.v1` | `nav_msgs` |

## 类型映射

| ROS 2 | Protobuf |
|-------|----------|
| `bool` | `bool` |
| `int8` / `int16` / `int32` | `int32` |
| `int64` | `int64` |
| `uint8` / `uint16` / `uint32` | `uint32` |
| `uint64` | `uint64` |
| `float32` | `float` |
| `float64` | `double` |
| `string` | `string` |
| `byte` | `uint32`（单字节包装消息） |
| `uint8[]` / `byte[]` | `bytes` |
| `T[]` | `repeated T` |
| `float64[N]` 固定数组 | `repeated double`（约定长度 N，见各消息注释） |

## 已定义消息

### builtin_interfaces

| Protobuf | ROS 2 |
|----------|-------|
| `Time` | `builtin_interfaces/msg/Time` |
| `Duration` | `builtin_interfaces/msg/Duration` |

### std_msgs

| Protobuf | ROS 2 |
|----------|-------|
| `Header` | `std_msgs/msg/Header` |
| `ColorRGBA` | `std_msgs/msg/ColorRGBA` |
| `MultiArrayDimension` | `std_msgs/msg/MultiArrayDimension` |
| `MultiArrayLayout` | `std_msgs/msg/MultiArrayLayout` |
| `Bool`, `Int32`, `Float64`, `String`, ... | 同名 primitive wrappers |
| `Float64MultiArray` | `std_msgs/msg/Float64MultiArray` |

### geometry_msgs

| Protobuf | ROS 2 |
|----------|-------|
| `Point` | `geometry_msgs/msg/Point` |
| `Vector3` | `geometry_msgs/msg/Vector3` |
| `Quaternion` | `geometry_msgs/msg/Quaternion` |
| `Pose` | `geometry_msgs/msg/Pose` |
| `Pose2D` | `geometry_msgs/msg/Pose2D` |
| `Twist` | `geometry_msgs/msg/Twist` |
| `Transform` | `geometry_msgs/msg/Transform` |
| `Accel`, `Wrench` | 同名 |
| `PoseStamped`, `TwistStamped`, `TransformStamped` | 同名 |
| `PoseWithCovariance`, `TwistWithCovariance` | 同名 |

### sensor_msgs

| Protobuf | ROS 2 |
|----------|-------|
| `Imu` | `sensor_msgs/msg/Imu` |
| `Image` | `sensor_msgs/msg/Image` |
| `CompressedImage` | `sensor_msgs/msg/CompressedImage` |
| `LaserScan` | `sensor_msgs/msg/LaserScan` |
| `JointState` | `sensor_msgs/msg/JointState` |
| `NavSatStatus` | `sensor_msgs/msg/NavSatStatus` |
| `NavSatFix` | `sensor_msgs/msg/NavSatFix` |

### nav_msgs

| Protobuf | ROS 2 |
|----------|-------|
| `Odometry` | `nav_msgs/msg/Odometry` |

## 使用

robot-bus 传输层 body 为 opaque bytes；调用方将上述 protobuf 序列化后作为 payload。

```bash
# 校验语法（需安装 protoc）
protoc -I proto --descriptor_set_out=/dev/null $(find proto -name '*.proto')
```

## 约定

- `Header.stamp` 与 ROS 2 一致：`sec` + `nanosec`。
- 协方差矩阵使用 `repeated double`，长度与 ROS 2 固定数组相同（如 Imu 为 9，Odometry 为 36）。
- `NavSatStatus.status` 保留 ROS 2 的负值语义（-2 表示 unknown）。
