"""Built-in and duck-typed ROS 2 ↔ robot-bus mappers."""

from robot_bus.ros2_bridge.mappers.fibonacci import FibonacciActionMapper
from robot_bus.ros2_bridge.mappers.image import SensorMsgsImageMapper
from robot_bus.ros2_bridge.mappers.set_bool import SetBoolServiceMapper
from robot_bus.ros2_bridge.mappers.string import StdMsgsStringMapper
from robot_bus.ros2_bridge.mappers.trigger import TriggerServiceMapper

from robot_bus.ros2_bridge.mappers.tf2_msgs.tf_message import Tf2MsgsTfMessageMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.inertia import GeometryMsgsInertiaMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.polygon_instance import GeometryMsgsPolygonInstanceMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist import GeometryMsgsTwistMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.accel import GeometryMsgsAccelMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.point_stamped import GeometryMsgsPointStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.accel_with_covariance_stamped import GeometryMsgsAccelWithCovarianceStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose2_d import GeometryMsgsPose2DMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist_stamped import GeometryMsgsTwistStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.polygon_instance_stamped import GeometryMsgsPolygonInstanceStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_array import GeometryMsgsPoseArrayMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3_stamped import GeometryMsgsVector3StampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_stamped import GeometryMsgsPoseStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.vector3 import GeometryMsgsVector3Mapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.quaternion import GeometryMsgsQuaternionMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_with_covariance_stamped import GeometryMsgsPoseWithCovarianceStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.accel_with_covariance import GeometryMsgsAccelWithCovarianceMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist_with_covariance import GeometryMsgsTwistWithCovarianceMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose import GeometryMsgsPoseMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.pose_with_covariance import GeometryMsgsPoseWithCovarianceMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform import GeometryMsgsTransformMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.point32 import GeometryMsgsPoint32Mapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.inertia_stamped import GeometryMsgsInertiaStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.point import GeometryMsgsPointMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.velocity_stamped import GeometryMsgsVelocityStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.twist_with_covariance_stamped import GeometryMsgsTwistWithCovarianceStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.accel_stamped import GeometryMsgsAccelStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.wrench import GeometryMsgsWrenchMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.quaternion_stamped import GeometryMsgsQuaternionStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.polygon import GeometryMsgsPolygonMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.wrench_stamped import GeometryMsgsWrenchStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform_stamped import GeometryMsgsTransformStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.polygon_stamped import GeometryMsgsPolygonStampedMapper
from robot_bus.ros2_bridge.mappers.std_msgs.u_int8_multi_array import StdMsgsUInt8MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.int16 import StdMsgsInt16Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.int64_multi_array import StdMsgsInt64MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.float64_multi_array import StdMsgsFloat64MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.int16_multi_array import StdMsgsInt16MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.color_rgba import StdMsgsColorRgbaMapper
from robot_bus.ros2_bridge.mappers.std_msgs.byte_multi_array import StdMsgsByteMultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.float64 import StdMsgsFloat64Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.multi_array_layout import StdMsgsMultiArrayLayoutMapper
from robot_bus.ros2_bridge.mappers.std_msgs.int32_multi_array import StdMsgsInt32MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.u_int32 import StdMsgsUInt32Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.float32 import StdMsgsFloat32Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.int8 import StdMsgsInt8Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.u_int64_multi_array import StdMsgsUInt64MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.byte import StdMsgsByteMapper
from robot_bus.ros2_bridge.mappers.std_msgs.u_int64 import StdMsgsUInt64Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.bool import StdMsgsBoolMapper
from robot_bus.ros2_bridge.mappers.std_msgs.u_int16 import StdMsgsUInt16Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.int32 import StdMsgsInt32Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.u_int8 import StdMsgsUInt8Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.float32_multi_array import StdMsgsFloat32MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.u_int16_multi_array import StdMsgsUInt16MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.u_int32_multi_array import StdMsgsUInt32MultiArrayMapper
from robot_bus.ros2_bridge.mappers.std_msgs.int64 import StdMsgsInt64Mapper
from robot_bus.ros2_bridge.mappers.std_msgs.header import StdMsgsHeaderMapper
from robot_bus.ros2_bridge.mappers.std_msgs.multi_array_dimension import StdMsgsMultiArrayDimensionMapper
from robot_bus.ros2_bridge.mappers.std_msgs.empty import StdMsgsEmptyMapper
from robot_bus.ros2_bridge.mappers.std_msgs.int8_multi_array import StdMsgsInt8MultiArrayMapper
from robot_bus.ros2_bridge.mappers.unique_identifier_msgs.uuid import UniqueIdentifierMsgsUuidMapper
from robot_bus.ros2_bridge.mappers.diagnostic_msgs.diagnostic_array import DiagnosticMsgsDiagnosticArrayMapper
from robot_bus.ros2_bridge.mappers.diagnostic_msgs.key_value import DiagnosticMsgsKeyValueMapper
from robot_bus.ros2_bridge.mappers.diagnostic_msgs.diagnostic_status import DiagnosticMsgsDiagnosticStatusMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.time_reference import SensorMsgsTimeReferenceMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.imu import SensorMsgsImuMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.multi_dof_joint_state import SensorMsgsMultiDofJointStateMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.illuminance import SensorMsgsIlluminanceMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.region_of_interest import SensorMsgsRegionOfInterestMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.temperature import SensorMsgsTemperatureMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.joy import SensorMsgsJoyMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.relative_humidity import SensorMsgsRelativeHumidityMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.compressed_image import SensorMsgsCompressedImageMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.channel_float32 import SensorMsgsChannelFloat32Mapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.point_cloud import SensorMsgsPointCloudMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.magnetic_field import SensorMsgsMagneticFieldMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.point_field import SensorMsgsPointFieldMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.laser_echo import SensorMsgsLaserEchoMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.battery_state import SensorMsgsBatteryStateMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.laser_scan import SensorMsgsLaserScanMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.joint_state import SensorMsgsJointStateMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.joy_feedback import SensorMsgsJoyFeedbackMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.joy_feedback_array import SensorMsgsJoyFeedbackArrayMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.point_cloud2 import SensorMsgsPointCloud2Mapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.nav_sat_status import SensorMsgsNavSatStatusMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.fluid_pressure import SensorMsgsFluidPressureMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.range import SensorMsgsRangeMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.nav_sat_fix import SensorMsgsNavSatFixMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.camera_info import SensorMsgsCameraInfoMapper
from robot_bus.ros2_bridge.mappers.sensor_msgs.multi_echo_laser_scan import SensorMsgsMultiEchoLaserScanMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.marker import VisualizationMsgsMarkerMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker_update import VisualizationMsgsInteractiveMarkerUpdateMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.mesh_file import VisualizationMsgsMeshFileMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker_pose import VisualizationMsgsInteractiveMarkerPoseMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.menu_entry import VisualizationMsgsMenuEntryMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker_control import VisualizationMsgsInteractiveMarkerControlMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.image_marker import VisualizationMsgsImageMarkerMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker_init import VisualizationMsgsInteractiveMarkerInitMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.marker_array import VisualizationMsgsMarkerArrayMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker_feedback import VisualizationMsgsInteractiveMarkerFeedbackMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.uv_coordinate import VisualizationMsgsUvCoordinateMapper
from robot_bus.ros2_bridge.mappers.visualization_msgs.interactive_marker import VisualizationMsgsInteractiveMarkerMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.occupancy_grid import NavMsgsOccupancyGridMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.odometry import NavMsgsOdometryMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.path import NavMsgsPathMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.map_meta_data import NavMsgsMapMetaDataMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.grid_cells import NavMsgsGridCellsMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.goals import NavMsgsGoalsMapper
from robot_bus.ros2_bridge.mappers.trajectory_msgs.joint_trajectory import TrajectoryMsgsJointTrajectoryMapper
from robot_bus.ros2_bridge.mappers.trajectory_msgs.multi_dof_joint_trajectory import TrajectoryMsgsMultiDofJointTrajectoryMapper
from robot_bus.ros2_bridge.mappers.trajectory_msgs.multi_dof_joint_trajectory_point import TrajectoryMsgsMultiDofJointTrajectoryPointMapper
from robot_bus.ros2_bridge.mappers.trajectory_msgs.joint_trajectory_point import TrajectoryMsgsJointTrajectoryPointMapper
from robot_bus.ros2_bridge.mappers.builtin_interfaces.time import BuiltinInterfacesTimeMapper
from robot_bus.ros2_bridge.mappers.builtin_interfaces.duration import BuiltinInterfacesDurationMapper
from robot_bus.ros2_bridge.mappers.stereo_msgs.disparity_image import StereoMsgsDisparityImageMapper
from robot_bus.ros2_bridge.mappers.shape_msgs.mesh import ShapeMsgsMeshMapper
from robot_bus.ros2_bridge.mappers.shape_msgs.mesh_triangle import ShapeMsgsMeshTriangleMapper
from robot_bus.ros2_bridge.mappers.shape_msgs.solid_primitive import ShapeMsgsSolidPrimitiveMapper
from robot_bus.ros2_bridge.mappers.shape_msgs.plane import ShapeMsgsPlaneMapper
from robot_bus.ros2_bridge.mappers.action_msgs.goal_status import ActionMsgsGoalStatusMapper
from robot_bus.ros2_bridge.mappers.action_msgs.goal_status_array import ActionMsgsGoalStatusArrayMapper
from robot_bus.ros2_bridge.mappers.action_msgs.goal_info import ActionMsgsGoalInfoMapper

__all__ = [
    "FibonacciActionMapper",
    "SensorMsgsImageMapper",
    "SetBoolServiceMapper",
    "StdMsgsStringMapper",
    "TriggerServiceMapper",
    "Tf2MsgsTfMessageMapper",
    "GeometryMsgsInertiaMapper",
    "GeometryMsgsPolygonInstanceMapper",
    "GeometryMsgsTwistMapper",
    "GeometryMsgsAccelMapper",
    "GeometryMsgsPointStampedMapper",
    "GeometryMsgsAccelWithCovarianceStampedMapper",
    "GeometryMsgsPose2DMapper",
    "GeometryMsgsTwistStampedMapper",
    "GeometryMsgsPolygonInstanceStampedMapper",
    "GeometryMsgsPoseArrayMapper",
    "GeometryMsgsVector3StampedMapper",
    "GeometryMsgsPoseStampedMapper",
    "GeometryMsgsVector3Mapper",
    "GeometryMsgsQuaternionMapper",
    "GeometryMsgsPoseWithCovarianceStampedMapper",
    "GeometryMsgsAccelWithCovarianceMapper",
    "GeometryMsgsTwistWithCovarianceMapper",
    "GeometryMsgsPoseMapper",
    "GeometryMsgsPoseWithCovarianceMapper",
    "GeometryMsgsTransformMapper",
    "GeometryMsgsPoint32Mapper",
    "GeometryMsgsInertiaStampedMapper",
    "GeometryMsgsPointMapper",
    "GeometryMsgsVelocityStampedMapper",
    "GeometryMsgsTwistWithCovarianceStampedMapper",
    "GeometryMsgsAccelStampedMapper",
    "GeometryMsgsWrenchMapper",
    "GeometryMsgsQuaternionStampedMapper",
    "GeometryMsgsPolygonMapper",
    "GeometryMsgsWrenchStampedMapper",
    "GeometryMsgsTransformStampedMapper",
    "GeometryMsgsPolygonStampedMapper",
    "StdMsgsUInt8MultiArrayMapper",
    "StdMsgsInt16Mapper",
    "StdMsgsInt64MultiArrayMapper",
    "StdMsgsFloat64MultiArrayMapper",
    "StdMsgsInt16MultiArrayMapper",
    "StdMsgsColorRgbaMapper",
    "StdMsgsByteMultiArrayMapper",
    "StdMsgsFloat64Mapper",
    "StdMsgsMultiArrayLayoutMapper",
    "StdMsgsInt32MultiArrayMapper",
    "StdMsgsUInt32Mapper",
    "StdMsgsFloat32Mapper",
    "StdMsgsInt8Mapper",
    "StdMsgsUInt64MultiArrayMapper",
    "StdMsgsByteMapper",
    "StdMsgsUInt64Mapper",
    "StdMsgsBoolMapper",
    "StdMsgsUInt16Mapper",
    "StdMsgsInt32Mapper",
    "StdMsgsUInt8Mapper",
    "StdMsgsFloat32MultiArrayMapper",
    "StdMsgsUInt16MultiArrayMapper",
    "StdMsgsUInt32MultiArrayMapper",
    "StdMsgsInt64Mapper",
    "StdMsgsHeaderMapper",
    "StdMsgsMultiArrayDimensionMapper",
    "StdMsgsEmptyMapper",
    "StdMsgsInt8MultiArrayMapper",
    "UniqueIdentifierMsgsUuidMapper",
    "DiagnosticMsgsDiagnosticArrayMapper",
    "DiagnosticMsgsKeyValueMapper",
    "DiagnosticMsgsDiagnosticStatusMapper",
    "SensorMsgsTimeReferenceMapper",
    "SensorMsgsImuMapper",
    "SensorMsgsMultiDofJointStateMapper",
    "SensorMsgsIlluminanceMapper",
    "SensorMsgsRegionOfInterestMapper",
    "SensorMsgsTemperatureMapper",
    "SensorMsgsJoyMapper",
    "SensorMsgsRelativeHumidityMapper",
    "SensorMsgsCompressedImageMapper",
    "SensorMsgsChannelFloat32Mapper",
    "SensorMsgsPointCloudMapper",
    "SensorMsgsMagneticFieldMapper",
    "SensorMsgsPointFieldMapper",
    "SensorMsgsLaserEchoMapper",
    "SensorMsgsBatteryStateMapper",
    "SensorMsgsLaserScanMapper",
    "SensorMsgsJointStateMapper",
    "SensorMsgsJoyFeedbackMapper",
    "SensorMsgsJoyFeedbackArrayMapper",
    "SensorMsgsPointCloud2Mapper",
    "SensorMsgsNavSatStatusMapper",
    "SensorMsgsFluidPressureMapper",
    "SensorMsgsRangeMapper",
    "SensorMsgsNavSatFixMapper",
    "SensorMsgsCameraInfoMapper",
    "SensorMsgsMultiEchoLaserScanMapper",
    "VisualizationMsgsMarkerMapper",
    "VisualizationMsgsInteractiveMarkerUpdateMapper",
    "VisualizationMsgsMeshFileMapper",
    "VisualizationMsgsInteractiveMarkerPoseMapper",
    "VisualizationMsgsMenuEntryMapper",
    "VisualizationMsgsInteractiveMarkerControlMapper",
    "VisualizationMsgsImageMarkerMapper",
    "VisualizationMsgsInteractiveMarkerInitMapper",
    "VisualizationMsgsMarkerArrayMapper",
    "VisualizationMsgsInteractiveMarkerFeedbackMapper",
    "VisualizationMsgsUvCoordinateMapper",
    "VisualizationMsgsInteractiveMarkerMapper",
    "NavMsgsOccupancyGridMapper",
    "NavMsgsOdometryMapper",
    "NavMsgsPathMapper",
    "NavMsgsMapMetaDataMapper",
    "NavMsgsGridCellsMapper",
    "NavMsgsGoalsMapper",
    "TrajectoryMsgsJointTrajectoryMapper",
    "TrajectoryMsgsMultiDofJointTrajectoryMapper",
    "TrajectoryMsgsMultiDofJointTrajectoryPointMapper",
    "TrajectoryMsgsJointTrajectoryPointMapper",
    "BuiltinInterfacesTimeMapper",
    "BuiltinInterfacesDurationMapper",
    "StereoMsgsDisparityImageMapper",
    "ShapeMsgsMeshMapper",
    "ShapeMsgsMeshTriangleMapper",
    "ShapeMsgsSolidPrimitiveMapper",
    "ShapeMsgsPlaneMapper",
    "ActionMsgsGoalStatusMapper",
    "ActionMsgsGoalStatusArrayMapper",
    "ActionMsgsGoalInfoMapper",
]
