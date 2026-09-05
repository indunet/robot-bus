// Generated typed ROS message stubs for `use_ros_shim` (topic mappers).

pub mod diagnostic_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct DiagnosticArray {
            pub header: crate::std_msgs::msg::Header,
            pub status: Vec<crate::diagnostic_msgs::msg::DiagnosticStatus>,
        }
        impl_rmw!(DiagnosticArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct DiagnosticStatus {
            pub level: u32,
            pub name: rosidl_runtime_rs::String,
            pub message: rosidl_runtime_rs::String,
            pub hardware_id: rosidl_runtime_rs::String,
            pub values: Vec<crate::diagnostic_msgs::msg::KeyValue>,
        }
        impl_rmw!(DiagnosticStatus);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct KeyValue {
            pub key: rosidl_runtime_rs::String,
            pub value: rosidl_runtime_rs::String,
        }
        impl_rmw!(KeyValue);

    }
}

pub mod geometry_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Accel {
            pub linear: crate::geometry_msgs::msg::Vector3,
            pub angular: crate::geometry_msgs::msg::Vector3,
        }
        impl_rmw!(Accel);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct AccelStamped {
            pub header: crate::std_msgs::msg::Header,
            pub accel: crate::geometry_msgs::msg::Accel,
        }
        impl_rmw!(AccelStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct AccelWithCovariance {
            pub accel: crate::geometry_msgs::msg::Accel,
            pub covariance: Vec<f64>,
        }
        impl_rmw!(AccelWithCovariance);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct AccelWithCovarianceStamped {
            pub header: crate::std_msgs::msg::Header,
            pub accel: crate::geometry_msgs::msg::AccelWithCovariance,
        }
        impl_rmw!(AccelWithCovarianceStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Inertia {
            pub m: f64,
            pub com: crate::geometry_msgs::msg::Vector3,
            pub ixx: f64,
            pub ixy: f64,
            pub ixz: f64,
            pub iyy: f64,
            pub iyz: f64,
            pub izz: f64,
        }
        impl_rmw!(Inertia);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct InertiaStamped {
            pub header: crate::std_msgs::msg::Header,
            pub inertia: crate::geometry_msgs::msg::Inertia,
        }
        impl_rmw!(InertiaStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Point {
            pub x: f64,
            pub y: f64,
            pub z: f64,
        }
        impl_rmw!(Point);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Point32 {
            pub x: f32,
            pub y: f32,
            pub z: f32,
        }
        impl_rmw!(Point32);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PointStamped {
            pub header: crate::std_msgs::msg::Header,
            pub point: crate::geometry_msgs::msg::Point,
        }
        impl_rmw!(PointStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Polygon {
            pub points: Vec<crate::geometry_msgs::msg::Point32>,
        }
        impl_rmw!(Polygon);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PolygonInstance {
            pub polygon: crate::geometry_msgs::msg::Polygon,
            pub id: i64,
        }
        impl_rmw!(PolygonInstance);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PolygonInstanceStamped {
            pub header: crate::std_msgs::msg::Header,
            pub polygon: crate::geometry_msgs::msg::PolygonInstance,
        }
        impl_rmw!(PolygonInstanceStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PolygonStamped {
            pub header: crate::std_msgs::msg::Header,
            pub polygon: crate::geometry_msgs::msg::Polygon,
        }
        impl_rmw!(PolygonStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Pose {
            pub position: crate::geometry_msgs::msg::Point,
            pub orientation: crate::geometry_msgs::msg::Quaternion,
        }
        impl_rmw!(Pose);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Pose2D {
            pub x: f64,
            pub y: f64,
            pub theta: f64,
        }
        impl_rmw!(Pose2D);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PoseArray {
            pub header: crate::std_msgs::msg::Header,
            pub poses: Vec<crate::geometry_msgs::msg::Pose>,
        }
        impl_rmw!(PoseArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PoseStamped {
            pub header: crate::std_msgs::msg::Header,
            pub pose: crate::geometry_msgs::msg::Pose,
        }
        impl_rmw!(PoseStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PoseWithCovariance {
            pub pose: crate::geometry_msgs::msg::Pose,
            pub covariance: Vec<f64>,
        }
        impl_rmw!(PoseWithCovariance);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PoseWithCovarianceStamped {
            pub header: crate::std_msgs::msg::Header,
            pub pose: crate::geometry_msgs::msg::PoseWithCovariance,
        }
        impl_rmw!(PoseWithCovarianceStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Quaternion {
            pub x: f64,
            pub y: f64,
            pub z: f64,
            pub w: f64,
        }
        impl_rmw!(Quaternion);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct QuaternionStamped {
            pub header: crate::std_msgs::msg::Header,
            pub quaternion: crate::geometry_msgs::msg::Quaternion,
        }
        impl_rmw!(QuaternionStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Transform {
            pub translation: crate::geometry_msgs::msg::Vector3,
            pub rotation: crate::geometry_msgs::msg::Quaternion,
        }
        impl_rmw!(Transform);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct TransformStamped {
            pub header: crate::std_msgs::msg::Header,
            pub child_frame_id: rosidl_runtime_rs::String,
            pub transform: crate::geometry_msgs::msg::Transform,
        }
        impl_rmw!(TransformStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Twist {
            pub linear: crate::geometry_msgs::msg::Vector3,
            pub angular: crate::geometry_msgs::msg::Vector3,
        }
        impl_rmw!(Twist);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct TwistStamped {
            pub header: crate::std_msgs::msg::Header,
            pub twist: crate::geometry_msgs::msg::Twist,
        }
        impl_rmw!(TwistStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct TwistWithCovariance {
            pub twist: crate::geometry_msgs::msg::Twist,
            pub covariance: Vec<f64>,
        }
        impl_rmw!(TwistWithCovariance);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct TwistWithCovarianceStamped {
            pub header: crate::std_msgs::msg::Header,
            pub twist: crate::geometry_msgs::msg::TwistWithCovariance,
        }
        impl_rmw!(TwistWithCovarianceStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Vector3 {
            pub x: f64,
            pub y: f64,
            pub z: f64,
        }
        impl_rmw!(Vector3);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Vector3Stamped {
            pub header: crate::std_msgs::msg::Header,
            pub vector: crate::geometry_msgs::msg::Vector3,
        }
        impl_rmw!(Vector3Stamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct VelocityStamped {
            pub header: crate::std_msgs::msg::Header,
            pub body_frame_id: rosidl_runtime_rs::String,
            pub reference_frame_id: rosidl_runtime_rs::String,
            pub velocity: crate::geometry_msgs::msg::Twist,
        }
        impl_rmw!(VelocityStamped);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Wrench {
            pub force: crate::geometry_msgs::msg::Vector3,
            pub torque: crate::geometry_msgs::msg::Vector3,
        }
        impl_rmw!(Wrench);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct WrenchStamped {
            pub header: crate::std_msgs::msg::Header,
            pub wrench: crate::geometry_msgs::msg::Wrench,
        }
        impl_rmw!(WrenchStamped);

    }
}

pub mod nav_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Goals {
            pub header: crate::std_msgs::msg::Header,
            pub goals: Vec<crate::geometry_msgs::msg::PoseStamped>,
        }
        impl_rmw!(Goals);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct GridCells {
            pub header: crate::std_msgs::msg::Header,
            pub cell_width: f32,
            pub cell_height: f32,
            pub cells: Vec<crate::geometry_msgs::msg::Point>,
        }
        impl_rmw!(GridCells);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MapMetaData {
            pub map_load_time: crate::builtin_interfaces::msg::Time,
            pub resolution: f32,
            pub width: u32,
            pub height: u32,
            pub origin: crate::geometry_msgs::msg::Pose,
        }
        impl_rmw!(MapMetaData);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct OccupancyGrid {
            pub header: crate::std_msgs::msg::Header,
            pub info: crate::nav_msgs::msg::MapMetaData,
            pub data: Vec<i8>,
        }
        impl_rmw!(OccupancyGrid);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Odometry {
            pub header: crate::std_msgs::msg::Header,
            pub child_frame_id: rosidl_runtime_rs::String,
            pub pose: crate::geometry_msgs::msg::PoseWithCovariance,
            pub twist: crate::geometry_msgs::msg::TwistWithCovariance,
        }
        impl_rmw!(Odometry);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Path {
            pub header: crate::std_msgs::msg::Header,
            pub poses: Vec<crate::geometry_msgs::msg::PoseStamped>,
        }
        impl_rmw!(Path);

    }
}

pub mod sensor_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct BatteryState {
            pub header: crate::std_msgs::msg::Header,
            pub voltage: f32,
            pub current: f32,
            pub charge: f32,
            pub capacity: f32,
            pub design_capacity: f32,
            pub percentage: f32,
            pub power_supply_status: u32,
            pub power_supply_health: u32,
            pub power_supply_technology: u32,
            pub present: bool,
            pub cell_voltage: Vec<f32>,
            pub cell_temperature: Vec<f32>,
            pub location: rosidl_runtime_rs::String,
            pub serial_number: rosidl_runtime_rs::String,
            pub temperature: f32,
        }
        impl_rmw!(BatteryState);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct CameraInfo {
            pub header: crate::std_msgs::msg::Header,
            pub height: u32,
            pub width: u32,
            pub distortion_model: rosidl_runtime_rs::String,
            pub d: Vec<f64>,
            pub k: Vec<f64>,
            pub r: Vec<f64>,
            pub p: Vec<f64>,
            pub binning_x: u32,
            pub binning_y: u32,
            pub roi: crate::sensor_msgs::msg::RegionOfInterest,
        }
        impl_rmw!(CameraInfo);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct ChannelFloat32 {
            pub name: rosidl_runtime_rs::String,
            pub values: Vec<f32>,
        }
        impl_rmw!(ChannelFloat32);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct CompressedImage {
            pub header: crate::std_msgs::msg::Header,
            pub format: rosidl_runtime_rs::String,
            pub data: Vec<u8>,
        }
        impl_rmw!(CompressedImage);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct FluidPressure {
            pub header: crate::std_msgs::msg::Header,
            pub fluid_pressure: f64,
            pub variance: f64,
        }
        impl_rmw!(FluidPressure);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Illuminance {
            pub header: crate::std_msgs::msg::Header,
            pub illuminance: f64,
            pub variance: f64,
        }
        impl_rmw!(Illuminance);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Image {
            pub header: crate::std_msgs::msg::Header,
            pub height: u32,
            pub width: u32,
            pub encoding: rosidl_runtime_rs::String,
            pub is_bigendian: u8,
            pub step: u32,
            pub data: Vec<u8>,
        }
        impl_rmw!(Image);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Imu {
            pub header: crate::std_msgs::msg::Header,
            pub orientation: crate::geometry_msgs::msg::Quaternion,
            pub orientation_covariance: Vec<f64>,
            pub angular_velocity: crate::geometry_msgs::msg::Vector3,
            pub angular_velocity_covariance: Vec<f64>,
            pub linear_acceleration: crate::geometry_msgs::msg::Vector3,
            pub linear_acceleration_covariance: Vec<f64>,
        }
        impl_rmw!(Imu);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct JointState {
            pub header: crate::std_msgs::msg::Header,
            pub name: Vec<rosidl_runtime_rs::String>,
            pub position: Vec<f64>,
            pub velocity: Vec<f64>,
            pub effort: Vec<f64>,
        }
        impl_rmw!(JointState);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Joy {
            pub header: crate::std_msgs::msg::Header,
            pub axes: Vec<f32>,
            pub buttons: Vec<i32>,
        }
        impl_rmw!(Joy);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct JoyFeedback {
            pub type_: u32,
            pub id: u32,
            pub intensity: f32,
        }
        impl_rmw!(JoyFeedback);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct JoyFeedbackArray {
            pub array: Vec<crate::sensor_msgs::msg::JoyFeedback>,
        }
        impl_rmw!(JoyFeedbackArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct LaserEcho {
            pub echoes: Vec<f32>,
        }
        impl_rmw!(LaserEcho);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct LaserScan {
            pub header: crate::std_msgs::msg::Header,
            pub angle_min: f32,
            pub angle_max: f32,
            pub angle_increment: f32,
            pub time_increment: f32,
            pub scan_time: f32,
            pub range_min: f32,
            pub range_max: f32,
            pub ranges: Vec<f32>,
            pub intensities: Vec<f32>,
        }
        impl_rmw!(LaserScan);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MagneticField {
            pub header: crate::std_msgs::msg::Header,
            pub magnetic_field: crate::geometry_msgs::msg::Vector3,
            pub magnetic_field_covariance: Vec<f64>,
        }
        impl_rmw!(MagneticField);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MultiDOFJointState {
            pub header: crate::std_msgs::msg::Header,
            pub joint_names: Vec<rosidl_runtime_rs::String>,
            pub transforms: Vec<crate::geometry_msgs::msg::Transform>,
            pub twist: Vec<crate::geometry_msgs::msg::Twist>,
            pub wrench: Vec<crate::geometry_msgs::msg::Wrench>,
        }
        impl_rmw!(MultiDOFJointState);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MultiEchoLaserScan {
            pub header: crate::std_msgs::msg::Header,
            pub angle_min: f32,
            pub angle_max: f32,
            pub angle_increment: f32,
            pub time_increment: f32,
            pub scan_time: f32,
            pub range_min: f32,
            pub range_max: f32,
            pub ranges: Vec<crate::sensor_msgs::msg::LaserEcho>,
            pub intensities: Vec<crate::sensor_msgs::msg::LaserEcho>,
        }
        impl_rmw!(MultiEchoLaserScan);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct NavSatFix {
            pub header: crate::std_msgs::msg::Header,
            pub status: crate::sensor_msgs::msg::NavSatStatus,
            pub latitude: f64,
            pub longitude: f64,
            pub altitude: f64,
            pub position_covariance: Vec<f64>,
            pub position_covariance_type: u32,
        }
        impl_rmw!(NavSatFix);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct NavSatStatus {
            pub status: i8,
            pub service: u16,
        }
        impl_rmw!(NavSatStatus);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PointCloud {
            pub header: crate::std_msgs::msg::Header,
            pub points: Vec<crate::geometry_msgs::msg::Point32>,
            pub channels: Vec<crate::sensor_msgs::msg::ChannelFloat32>,
        }
        impl_rmw!(PointCloud);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PointCloud2 {
            pub header: crate::std_msgs::msg::Header,
            pub height: u32,
            pub width: u32,
            pub fields: Vec<crate::sensor_msgs::msg::PointField>,
            pub is_bigendian: bool,
            pub point_step: u32,
            pub row_step: u32,
            pub data: Vec<u8>,
            pub is_dense: bool,
        }
        impl_rmw!(PointCloud2);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct PointField {
            pub name: rosidl_runtime_rs::String,
            pub offset: u32,
            pub datatype: u8,
            pub count: u32,
        }
        impl_rmw!(PointField);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Range {
            pub header: crate::std_msgs::msg::Header,
            pub radiation_type: u32,
            pub field_of_view: f32,
            pub min_range: f32,
            pub max_range: f32,
            pub range: f32,
        }
        impl_rmw!(Range);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct RegionOfInterest {
            pub x_offset: u32,
            pub y_offset: u32,
            pub height: u32,
            pub width: u32,
            pub do_rectify: bool,
        }
        impl_rmw!(RegionOfInterest);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct RelativeHumidity {
            pub header: crate::std_msgs::msg::Header,
            pub relative_humidity: f64,
            pub variance: f64,
        }
        impl_rmw!(RelativeHumidity);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Temperature {
            pub header: crate::std_msgs::msg::Header,
            pub temperature: f64,
            pub variance: f64,
        }
        impl_rmw!(Temperature);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct TimeReference {
            pub header: crate::std_msgs::msg::Header,
            pub time_ref: crate::builtin_interfaces::msg::Time,
            pub source: rosidl_runtime_rs::String,
        }
        impl_rmw!(TimeReference);

    }
}

pub mod shape_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Mesh {
            pub triangles: Vec<crate::shape_msgs::msg::MeshTriangle>,
            pub vertices: Vec<crate::geometry_msgs::msg::Point>,
        }
        impl_rmw!(Mesh);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MeshTriangle {
            pub vertex_indices: Vec<u32>,
        }
        impl_rmw!(MeshTriangle);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Plane {
            pub coef: Vec<f64>,
        }
        impl_rmw!(Plane);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct SolidPrimitive {
            pub type_: u32,
            pub dimensions: Vec<f64>,
            pub polygon: crate::geometry_msgs::msg::Polygon,
        }
        impl_rmw!(SolidPrimitive);

    }
}

pub mod std_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Bool {
            pub data: bool,
        }
        impl_rmw!(Bool);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Byte {
            pub data: u8,
        }
        impl_rmw!(Byte);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct ByteMultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<u8>,
        }
        impl_rmw!(ByteMultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct ColorRGBA {
            pub r: f32,
            pub g: f32,
            pub b: f32,
            pub a: f32,
        }
        impl_rmw!(ColorRGBA);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Empty {

        }
        impl_rmw!(Empty);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Float32 {
            pub data: f32,
        }
        impl_rmw!(Float32);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Float32MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<f32>,
        }
        impl_rmw!(Float32MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Float64 {
            pub data: f64,
        }
        impl_rmw!(Float64);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Float64MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<f64>,
        }
        impl_rmw!(Float64MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Header {
            pub stamp: crate::builtin_interfaces::msg::Time,
            pub frame_id: rosidl_runtime_rs::String,
        }
        impl_rmw!(Header);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Int16 {
            pub data: i16,
        }
        impl_rmw!(Int16);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Int16MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<i32>,
        }
        impl_rmw!(Int16MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Int32 {
            pub data: i32,
        }
        impl_rmw!(Int32);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Int32MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<i32>,
        }
        impl_rmw!(Int32MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Int64 {
            pub data: i64,
        }
        impl_rmw!(Int64);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Int64MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<i64>,
        }
        impl_rmw!(Int64MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Int8 {
            pub data: i8,
        }
        impl_rmw!(Int8);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Int8MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<i32>,
        }
        impl_rmw!(Int8MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MultiArrayDimension {
            pub label: rosidl_runtime_rs::String,
            pub size: u32,
            pub stride: u32,
        }
        impl_rmw!(MultiArrayDimension);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MultiArrayLayout {
            pub dim: Vec<crate::std_msgs::msg::MultiArrayDimension>,
            pub data_offset: u32,
        }
        impl_rmw!(MultiArrayLayout);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct String {
            pub data: rosidl_runtime_rs::String,
        }
        impl_rmw!(String);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UInt16 {
            pub data: u16,
        }
        impl_rmw!(UInt16);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UInt16MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<u32>,
        }
        impl_rmw!(UInt16MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UInt32 {
            pub data: u32,
        }
        impl_rmw!(UInt32);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UInt32MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<u32>,
        }
        impl_rmw!(UInt32MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UInt64 {
            pub data: u64,
        }
        impl_rmw!(UInt64);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UInt64MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<u64>,
        }
        impl_rmw!(UInt64MultiArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UInt8 {
            pub data: u8,
        }
        impl_rmw!(UInt8);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UInt8MultiArray {
            pub layout: crate::std_msgs::msg::MultiArrayLayout,
            pub data: Vec<u32>,
        }
        impl_rmw!(UInt8MultiArray);

    }
}

pub mod stereo_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct DisparityImage {
            pub header: crate::std_msgs::msg::Header,
            pub image: crate::sensor_msgs::msg::Image,
            pub f: f32,
            pub t: f32,
            pub valid_window: crate::sensor_msgs::msg::RegionOfInterest,
            pub min_disparity: f32,
            pub max_disparity: f32,
            pub delta_d: f32,
        }
        impl_rmw!(DisparityImage);

    }
}

pub mod tf2_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct TFMessage {
            pub transforms: Vec<crate::geometry_msgs::msg::TransformStamped>,
        }
        impl_rmw!(TFMessage);

    }
}

pub mod trajectory_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct JointTrajectory {
            pub header: crate::std_msgs::msg::Header,
            pub joint_names: Vec<rosidl_runtime_rs::String>,
            pub points: Vec<crate::trajectory_msgs::msg::JointTrajectoryPoint>,
        }
        impl_rmw!(JointTrajectory);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct JointTrajectoryPoint {
            pub positions: Vec<f64>,
            pub velocities: Vec<f64>,
            pub accelerations: Vec<f64>,
            pub effort: Vec<f64>,
            pub time_from_start: crate::builtin_interfaces::msg::Duration,
        }
        impl_rmw!(JointTrajectoryPoint);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MultiDOFJointTrajectory {
            pub header: crate::std_msgs::msg::Header,
            pub joint_names: Vec<rosidl_runtime_rs::String>,
            pub points: Vec<crate::trajectory_msgs::msg::MultiDOFJointTrajectoryPoint>,
        }
        impl_rmw!(MultiDOFJointTrajectory);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MultiDOFJointTrajectoryPoint {
            pub transforms: Vec<crate::geometry_msgs::msg::Transform>,
            pub velocities: Vec<crate::geometry_msgs::msg::Twist>,
            pub accelerations: Vec<crate::geometry_msgs::msg::Twist>,
            pub time_from_start: crate::builtin_interfaces::msg::Duration,
        }
        impl_rmw!(MultiDOFJointTrajectoryPoint);

    }
}

pub mod visualization_msgs {
    pub mod msg {

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct ImageMarker {
            pub header: crate::std_msgs::msg::Header,
            pub ns: rosidl_runtime_rs::String,
            pub id: i32,
            pub type_: i32,
            pub action: i32,
            pub position: crate::geometry_msgs::msg::Point,
            pub scale: f32,
            pub outline_color: crate::std_msgs::msg::ColorRGBA,
            pub filled: u32,
            pub fill_color: crate::std_msgs::msg::ColorRGBA,
            pub lifetime: crate::builtin_interfaces::msg::Duration,
            pub points: Vec<crate::geometry_msgs::msg::Point>,
            pub outline_colors: Vec<crate::std_msgs::msg::ColorRGBA>,
        }
        impl_rmw!(ImageMarker);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct InteractiveMarker {
            pub header: crate::std_msgs::msg::Header,
            pub pose: crate::geometry_msgs::msg::Pose,
            pub name: rosidl_runtime_rs::String,
            pub description: rosidl_runtime_rs::String,
            pub scale: f32,
            pub menu_entries: Vec<crate::visualization_msgs::msg::MenuEntry>,
            pub controls: Vec<crate::visualization_msgs::msg::InteractiveMarkerControl>,
        }
        impl_rmw!(InteractiveMarker);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct InteractiveMarkerControl {
            pub name: rosidl_runtime_rs::String,
            pub orientation: crate::geometry_msgs::msg::Quaternion,
            pub orientation_mode: u32,
            pub interaction_mode: u32,
            pub always_visible: bool,
            pub markers: Vec<crate::visualization_msgs::msg::Marker>,
            pub independent_marker_orientation: bool,
            pub description: rosidl_runtime_rs::String,
        }
        impl_rmw!(InteractiveMarkerControl);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct InteractiveMarkerFeedback {
            pub header: crate::std_msgs::msg::Header,
            pub client_id: rosidl_runtime_rs::String,
            pub marker_name: rosidl_runtime_rs::String,
            pub control_name: rosidl_runtime_rs::String,
            pub event_type: u32,
            pub pose: crate::geometry_msgs::msg::Pose,
            pub menu_entry_id: u32,
            pub mouse_point: crate::geometry_msgs::msg::Point,
            pub mouse_point_valid: bool,
        }
        impl_rmw!(InteractiveMarkerFeedback);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct InteractiveMarkerInit {
            pub server_id: rosidl_runtime_rs::String,
            pub seq_num: u64,
            pub markers: Vec<crate::visualization_msgs::msg::InteractiveMarker>,
        }
        impl_rmw!(InteractiveMarkerInit);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct InteractiveMarkerPose {
            pub header: crate::std_msgs::msg::Header,
            pub pose: crate::geometry_msgs::msg::Pose,
            pub name: rosidl_runtime_rs::String,
        }
        impl_rmw!(InteractiveMarkerPose);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct InteractiveMarkerUpdate {
            pub server_id: rosidl_runtime_rs::String,
            pub seq_num: u64,
            pub type_: u32,
            pub markers: Vec<crate::visualization_msgs::msg::InteractiveMarker>,
            pub poses: Vec<crate::visualization_msgs::msg::InteractiveMarkerPose>,
            pub erases: Vec<rosidl_runtime_rs::String>,
        }
        impl_rmw!(InteractiveMarkerUpdate);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Marker {
            pub header: crate::std_msgs::msg::Header,
            pub ns: rosidl_runtime_rs::String,
            pub id: i32,
            pub type_: i32,
            pub action: i32,
            pub pose: crate::geometry_msgs::msg::Pose,
            pub scale: crate::geometry_msgs::msg::Vector3,
            pub color: crate::std_msgs::msg::ColorRGBA,
            pub lifetime: crate::builtin_interfaces::msg::Duration,
            pub frame_locked: bool,
            pub points: Vec<crate::geometry_msgs::msg::Point>,
            pub colors: Vec<crate::std_msgs::msg::ColorRGBA>,
            pub texture_resource: rosidl_runtime_rs::String,
            pub texture: crate::sensor_msgs::msg::CompressedImage,
            pub uv_coordinates: Vec<crate::visualization_msgs::msg::UVCoordinate>,
            pub text: rosidl_runtime_rs::String,
            pub mesh_resource: rosidl_runtime_rs::String,
            pub mesh_file: crate::visualization_msgs::msg::MeshFile,
            pub mesh_use_embedded_materials: bool,
        }
        impl_rmw!(Marker);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MarkerArray {
            pub markers: Vec<crate::visualization_msgs::msg::Marker>,
        }
        impl_rmw!(MarkerArray);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MenuEntry {
            pub id: u32,
            pub parent_id: u32,
            pub title: rosidl_runtime_rs::String,
            pub command: rosidl_runtime_rs::String,
            pub command_type: u32,
        }
        impl_rmw!(MenuEntry);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct MeshFile {
            pub filename: rosidl_runtime_rs::String,
            pub data: Vec<u8>,
        }
        impl_rmw!(MeshFile);


        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct UVCoordinate {
            pub u: f32,
            pub v: f32,
        }
        impl_rmw!(UVCoordinate);

    }
}
