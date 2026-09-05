//! Per-type topic mappers, organized as `mappers/<pkg>/<msg>.rs`.
//!
//! Typed field converters live in [`convert`]; service/action helpers used by
//! the bridge builder live in [`service`] / [`action`].
//!
//! Builtin topic packages match Humble/Jazzy distro-common interfaces only.
//! Extension stacks (nav2 / control / foxglove / apriltag) are not shipped;
//! write a TypedTopicMapper for those.

pub mod action;
pub mod action_bridges;
pub mod convert;
pub mod service;
pub mod service_bridges;

pub mod action_msgs;
pub mod builtin_interfaces;
pub mod diagnostic_msgs;
pub mod geometry_msgs;
pub mod nav_msgs;
pub mod sensor_msgs;
pub mod shape_msgs;
pub mod std_msgs;
pub mod stereo_msgs;
pub mod tf2_msgs;
pub mod trajectory_msgs;
pub mod unique_identifier_msgs;
pub mod visualization_msgs;

use crate::ros2_bridge::mapper::TopicMapper;

/// All built-in topic mappers (one entry per ROS message type).
pub(crate) static BUILTIN_MAPPER_LIST: &[(&'static str, &'static dyn TopicMapper)] = &[
    ("action_msgs/msg/GoalInfo", &action_msgs::goal_info::ActionMsgsGoalInfoMapper),
    ("action_msgs/msg/GoalStatus", &action_msgs::goal_status::ActionMsgsGoalStatusMapper),
    ("action_msgs/msg/GoalStatusArray", &action_msgs::goal_status_array::ActionMsgsGoalStatusArrayMapper),
    ("builtin_interfaces/msg/Duration", &builtin_interfaces::duration::BuiltinInterfacesDurationMapper),
    ("builtin_interfaces/msg/Time", &builtin_interfaces::time::BuiltinInterfacesTimeMapper),
    ("diagnostic_msgs/msg/DiagnosticArray", &diagnostic_msgs::diagnostic_array::DiagnosticMsgsDiagnosticArrayMapper),
    ("diagnostic_msgs/msg/DiagnosticStatus", &diagnostic_msgs::diagnostic_status::DiagnosticMsgsDiagnosticStatusMapper),
    ("diagnostic_msgs/msg/KeyValue", &diagnostic_msgs::key_value::DiagnosticMsgsKeyValueMapper),
    ("geometry_msgs/msg/Accel", &geometry_msgs::accel::GeometryMsgsAccelMapper),
    ("geometry_msgs/msg/AccelStamped", &geometry_msgs::accel_stamped::GeometryMsgsAccelStampedMapper),
    ("geometry_msgs/msg/AccelWithCovariance", &geometry_msgs::accel_with_covariance::GeometryMsgsAccelWithCovarianceMapper),
    ("geometry_msgs/msg/AccelWithCovarianceStamped", &geometry_msgs::accel_with_covariance_stamped::GeometryMsgsAccelWithCovarianceStampedMapper),
    ("geometry_msgs/msg/Inertia", &geometry_msgs::inertia::GeometryMsgsInertiaMapper),
    ("geometry_msgs/msg/InertiaStamped", &geometry_msgs::inertia_stamped::GeometryMsgsInertiaStampedMapper),
    ("geometry_msgs/msg/Point", &geometry_msgs::point::GeometryMsgsPointMapper),
    ("geometry_msgs/msg/Point32", &geometry_msgs::point32::GeometryMsgsPoint32Mapper),
    ("geometry_msgs/msg/PointStamped", &geometry_msgs::point_stamped::GeometryMsgsPointStampedMapper),
    ("geometry_msgs/msg/Polygon", &geometry_msgs::polygon::GeometryMsgsPolygonMapper),
    ("geometry_msgs/msg/PolygonInstance", &geometry_msgs::polygon_instance::GeometryMsgsPolygonInstanceMapper),
    ("geometry_msgs/msg/PolygonInstanceStamped", &geometry_msgs::polygon_instance_stamped::GeometryMsgsPolygonInstanceStampedMapper),
    ("geometry_msgs/msg/PolygonStamped", &geometry_msgs::polygon_stamped::GeometryMsgsPolygonStampedMapper),
    ("geometry_msgs/msg/Pose", &geometry_msgs::pose::GeometryMsgsPoseMapper),
    ("geometry_msgs/msg/Pose2D", &geometry_msgs::pose2_d::GeometryMsgsPose2DMapper),
    ("geometry_msgs/msg/PoseArray", &geometry_msgs::pose_array::GeometryMsgsPoseArrayMapper),
    ("geometry_msgs/msg/PoseStamped", &geometry_msgs::pose_stamped::GeometryMsgsPoseStampedMapper),
    ("geometry_msgs/msg/PoseWithCovariance", &geometry_msgs::pose_with_covariance::GeometryMsgsPoseWithCovarianceMapper),
    ("geometry_msgs/msg/PoseWithCovarianceStamped", &geometry_msgs::pose_with_covariance_stamped::GeometryMsgsPoseWithCovarianceStampedMapper),
    ("geometry_msgs/msg/Quaternion", &geometry_msgs::quaternion::GeometryMsgsQuaternionMapper),
    ("geometry_msgs/msg/QuaternionStamped", &geometry_msgs::quaternion_stamped::GeometryMsgsQuaternionStampedMapper),
    ("geometry_msgs/msg/Transform", &geometry_msgs::transform::GeometryMsgsTransformMapper),
    ("geometry_msgs/msg/TransformStamped", &geometry_msgs::transform_stamped::GeometryMsgsTransformStampedMapper),
    ("geometry_msgs/msg/Twist", &geometry_msgs::twist::GeometryMsgsTwistMapper),
    ("geometry_msgs/msg/TwistStamped", &geometry_msgs::twist_stamped::GeometryMsgsTwistStampedMapper),
    ("geometry_msgs/msg/TwistWithCovariance", &geometry_msgs::twist_with_covariance::GeometryMsgsTwistWithCovarianceMapper),
    ("geometry_msgs/msg/TwistWithCovarianceStamped", &geometry_msgs::twist_with_covariance_stamped::GeometryMsgsTwistWithCovarianceStampedMapper),
    ("geometry_msgs/msg/Vector3", &geometry_msgs::vector3::GeometryMsgsVector3Mapper),
    ("geometry_msgs/msg/Vector3Stamped", &geometry_msgs::vector3_stamped::GeometryMsgsVector3StampedMapper),
    ("geometry_msgs/msg/VelocityStamped", &geometry_msgs::velocity_stamped::GeometryMsgsVelocityStampedMapper),
    ("geometry_msgs/msg/Wrench", &geometry_msgs::wrench::GeometryMsgsWrenchMapper),
    ("geometry_msgs/msg/WrenchStamped", &geometry_msgs::wrench_stamped::GeometryMsgsWrenchStampedMapper),
    ("nav_msgs/msg/Goals", &nav_msgs::goals::NavMsgsGoalsMapper),
    ("nav_msgs/msg/GridCells", &nav_msgs::grid_cells::NavMsgsGridCellsMapper),
    ("nav_msgs/msg/MapMetaData", &nav_msgs::map_meta_data::NavMsgsMapMetaDataMapper),
    ("nav_msgs/msg/OccupancyGrid", &nav_msgs::occupancy_grid::NavMsgsOccupancyGridMapper),
    ("nav_msgs/msg/Odometry", &nav_msgs::odometry::NavMsgsOdometryMapper),
    ("nav_msgs/msg/Path", &nav_msgs::path::NavMsgsPathMapper),
    ("sensor_msgs/msg/BatteryState", &sensor_msgs::battery_state::SensorMsgsBatteryStateMapper),
    ("sensor_msgs/msg/CameraInfo", &sensor_msgs::camera_info::SensorMsgsCameraInfoMapper),
    ("sensor_msgs/msg/ChannelFloat32", &sensor_msgs::channel_float32::SensorMsgsChannelFloat32Mapper),
    ("sensor_msgs/msg/CompressedImage", &sensor_msgs::compressed_image::SensorMsgsCompressedImageMapper),
    ("sensor_msgs/msg/FluidPressure", &sensor_msgs::fluid_pressure::SensorMsgsFluidPressureMapper),
    ("sensor_msgs/msg/Illuminance", &sensor_msgs::illuminance::SensorMsgsIlluminanceMapper),
    ("sensor_msgs/msg/Image", &sensor_msgs::image::SensorMsgsImageMapper),
    ("sensor_msgs/msg/Imu", &sensor_msgs::imu::SensorMsgsImuMapper),
    ("sensor_msgs/msg/JointState", &sensor_msgs::joint_state::SensorMsgsJointStateMapper),
    ("sensor_msgs/msg/Joy", &sensor_msgs::joy::SensorMsgsJoyMapper),
    ("sensor_msgs/msg/JoyFeedback", &sensor_msgs::joy_feedback::SensorMsgsJoyFeedbackMapper),
    ("sensor_msgs/msg/JoyFeedbackArray", &sensor_msgs::joy_feedback_array::SensorMsgsJoyFeedbackArrayMapper),
    ("sensor_msgs/msg/LaserEcho", &sensor_msgs::laser_echo::SensorMsgsLaserEchoMapper),
    ("sensor_msgs/msg/LaserScan", &sensor_msgs::laser_scan::SensorMsgsLaserScanMapper),
    ("sensor_msgs/msg/MagneticField", &sensor_msgs::magnetic_field::SensorMsgsMagneticFieldMapper),
    ("sensor_msgs/msg/MultiDOFJointState", &sensor_msgs::multi_dof_joint_state::SensorMsgsMultiDofJointStateMapper),
    ("sensor_msgs/msg/MultiEchoLaserScan", &sensor_msgs::multi_echo_laser_scan::SensorMsgsMultiEchoLaserScanMapper),
    ("sensor_msgs/msg/NavSatFix", &sensor_msgs::nav_sat_fix::SensorMsgsNavSatFixMapper),
    ("sensor_msgs/msg/NavSatStatus", &sensor_msgs::nav_sat_status::SensorMsgsNavSatStatusMapper),
    ("sensor_msgs/msg/PointCloud", &sensor_msgs::point_cloud::SensorMsgsPointCloudMapper),
    ("sensor_msgs/msg/PointCloud2", &sensor_msgs::point_cloud2::SensorMsgsPointCloud2Mapper),
    ("sensor_msgs/msg/PointField", &sensor_msgs::point_field::SensorMsgsPointFieldMapper),
    ("sensor_msgs/msg/Range", &sensor_msgs::range::SensorMsgsRangeMapper),
    ("sensor_msgs/msg/RegionOfInterest", &sensor_msgs::region_of_interest::SensorMsgsRegionOfInterestMapper),
    ("sensor_msgs/msg/RelativeHumidity", &sensor_msgs::relative_humidity::SensorMsgsRelativeHumidityMapper),
    ("sensor_msgs/msg/Temperature", &sensor_msgs::temperature::SensorMsgsTemperatureMapper),
    ("sensor_msgs/msg/TimeReference", &sensor_msgs::time_reference::SensorMsgsTimeReferenceMapper),
    ("shape_msgs/msg/Mesh", &shape_msgs::mesh::ShapeMsgsMeshMapper),
    ("shape_msgs/msg/MeshTriangle", &shape_msgs::mesh_triangle::ShapeMsgsMeshTriangleMapper),
    ("shape_msgs/msg/Plane", &shape_msgs::plane::ShapeMsgsPlaneMapper),
    ("shape_msgs/msg/SolidPrimitive", &shape_msgs::solid_primitive::ShapeMsgsSolidPrimitiveMapper),
    ("std_msgs/msg/Bool", &std_msgs::bool::StdMsgsBoolMapper),
    ("std_msgs/msg/Byte", &std_msgs::byte::StdMsgsByteMapper),
    ("std_msgs/msg/ByteMultiArray", &std_msgs::byte_multi_array::StdMsgsByteMultiArrayMapper),
    ("std_msgs/msg/ColorRGBA", &std_msgs::color_rgba::StdMsgsColorRgbaMapper),
    ("std_msgs/msg/Empty", &std_msgs::empty::StdMsgsEmptyMapper),
    ("std_msgs/msg/Float32", &std_msgs::float32::StdMsgsFloat32Mapper),
    ("std_msgs/msg/Float32MultiArray", &std_msgs::float32_multi_array::StdMsgsFloat32MultiArrayMapper),
    ("std_msgs/msg/Float64", &std_msgs::float64::StdMsgsFloat64Mapper),
    ("std_msgs/msg/Float64MultiArray", &std_msgs::float64_multi_array::StdMsgsFloat64MultiArrayMapper),
    ("std_msgs/msg/Header", &std_msgs::header::StdMsgsHeaderMapper),
    ("std_msgs/msg/Int16", &std_msgs::int16::StdMsgsInt16Mapper),
    ("std_msgs/msg/Int16MultiArray", &std_msgs::int16_multi_array::StdMsgsInt16MultiArrayMapper),
    ("std_msgs/msg/Int32", &std_msgs::int32::StdMsgsInt32Mapper),
    ("std_msgs/msg/Int32MultiArray", &std_msgs::int32_multi_array::StdMsgsInt32MultiArrayMapper),
    ("std_msgs/msg/Int64", &std_msgs::int64::StdMsgsInt64Mapper),
    ("std_msgs/msg/Int64MultiArray", &std_msgs::int64_multi_array::StdMsgsInt64MultiArrayMapper),
    ("std_msgs/msg/Int8", &std_msgs::int8::StdMsgsInt8Mapper),
    ("std_msgs/msg/Int8MultiArray", &std_msgs::int8_multi_array::StdMsgsInt8MultiArrayMapper),
    ("std_msgs/msg/MultiArrayDimension", &std_msgs::multi_array_dimension::StdMsgsMultiArrayDimensionMapper),
    ("std_msgs/msg/MultiArrayLayout", &std_msgs::multi_array_layout::StdMsgsMultiArrayLayoutMapper),
    ("std_msgs/msg/String", &std_msgs::string::StdMsgsStringMapper),
    ("std_msgs/msg/UInt16", &std_msgs::uint16::StdMsgsUInt16Mapper),
    ("std_msgs/msg/UInt16MultiArray", &std_msgs::uint16_multi_array::StdMsgsUInt16MultiArrayMapper),
    ("std_msgs/msg/UInt32", &std_msgs::uint32::StdMsgsUInt32Mapper),
    ("std_msgs/msg/UInt32MultiArray", &std_msgs::uint32_multi_array::StdMsgsUInt32MultiArrayMapper),
    ("std_msgs/msg/UInt64", &std_msgs::uint64::StdMsgsUInt64Mapper),
    ("std_msgs/msg/UInt64MultiArray", &std_msgs::uint64_multi_array::StdMsgsUInt64MultiArrayMapper),
    ("std_msgs/msg/UInt8", &std_msgs::uint8::StdMsgsUInt8Mapper),
    ("std_msgs/msg/UInt8MultiArray", &std_msgs::uint8_multi_array::StdMsgsUInt8MultiArrayMapper),
    ("stereo_msgs/msg/DisparityImage", &stereo_msgs::disparity_image::StereoMsgsDisparityImageMapper),
    ("tf2_msgs/msg/TFMessage", &tf2_msgs::tf_message::Tf2MsgsTfMessageMapper),
    ("trajectory_msgs/msg/JointTrajectory", &trajectory_msgs::joint_trajectory::TrajectoryMsgsJointTrajectoryMapper),
    ("trajectory_msgs/msg/JointTrajectoryPoint", &trajectory_msgs::joint_trajectory_point::TrajectoryMsgsJointTrajectoryPointMapper),
    ("trajectory_msgs/msg/MultiDOFJointTrajectory", &trajectory_msgs::multi_dof_joint_trajectory::TrajectoryMsgsMultiDofJointTrajectoryMapper),
    ("trajectory_msgs/msg/MultiDOFJointTrajectoryPoint", &trajectory_msgs::multi_dof_joint_trajectory_point::TrajectoryMsgsMultiDofJointTrajectoryPointMapper),
    ("unique_identifier_msgs/msg/UUID", &unique_identifier_msgs::uuid::UniqueIdentifierMsgsUuidMapper),
    ("visualization_msgs/msg/ImageMarker", &visualization_msgs::image_marker::VisualizationMsgsImageMarkerMapper),
    ("visualization_msgs/msg/InteractiveMarker", &visualization_msgs::interactive_marker::VisualizationMsgsInteractiveMarkerMapper),
    ("visualization_msgs/msg/InteractiveMarkerControl", &visualization_msgs::interactive_marker_control::VisualizationMsgsInteractiveMarkerControlMapper),
    ("visualization_msgs/msg/InteractiveMarkerFeedback", &visualization_msgs::interactive_marker_feedback::VisualizationMsgsInteractiveMarkerFeedbackMapper),
    ("visualization_msgs/msg/InteractiveMarkerInit", &visualization_msgs::interactive_marker_init::VisualizationMsgsInteractiveMarkerInitMapper),
    ("visualization_msgs/msg/InteractiveMarkerPose", &visualization_msgs::interactive_marker_pose::VisualizationMsgsInteractiveMarkerPoseMapper),
    ("visualization_msgs/msg/InteractiveMarkerUpdate", &visualization_msgs::interactive_marker_update::VisualizationMsgsInteractiveMarkerUpdateMapper),
    ("visualization_msgs/msg/Marker", &visualization_msgs::marker::VisualizationMsgsMarkerMapper),
    ("visualization_msgs/msg/MarkerArray", &visualization_msgs::marker_array::VisualizationMsgsMarkerArrayMapper),
    ("visualization_msgs/msg/MenuEntry", &visualization_msgs::menu_entry::VisualizationMsgsMenuEntryMapper),
    ("visualization_msgs/msg/MeshFile", &visualization_msgs::mesh_file::VisualizationMsgsMeshFileMapper),
    ("visualization_msgs/msg/UVCoordinate", &visualization_msgs::uv_coordinate::VisualizationMsgsUvCoordinateMapper),
];
