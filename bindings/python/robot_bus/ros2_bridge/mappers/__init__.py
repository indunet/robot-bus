"""Built-in and duck-typed ROS 2 ↔ robot-bus mappers."""

from robot_bus.ros2_bridge.mappers.fibonacci import FibonacciActionMapper
from robot_bus.ros2_bridge.mappers.image import SensorMsgsImageMapper
from robot_bus.ros2_bridge.mappers.set_bool import SetBoolServiceMapper
from robot_bus.ros2_bridge.mappers.string import StdMsgsStringMapper
from robot_bus.ros2_bridge.mappers.trigger import TriggerServiceMapper

from robot_bus.ros2_bridge.mappers.tf2_msgs.tf_message import Tf2MsgsTfMessageMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.scene_update import FoxgloveMsgsSceneUpdateMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.cylinder_primitive import FoxgloveMsgsCylinderPrimitiveMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.location_fix import FoxgloveMsgsLocationFixMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.raw_image import FoxgloveMsgsRawImageMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.poses_in_frame import FoxgloveMsgsPosesInFrameMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.odometry import FoxgloveMsgsOdometryMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.compressed_point_cloud import FoxgloveMsgsCompressedPointCloudMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector2 import FoxgloveMsgsVector2Mapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.sphere_primitive import FoxgloveMsgsSpherePrimitiveMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.frame_transforms import FoxgloveMsgsFrameTransformsMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.triangle_list_primitive import FoxgloveMsgsTriangleListPrimitiveMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.compressed_image import FoxgloveMsgsCompressedImageMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point3_in_frame import FoxgloveMsgsPoint3InFrameMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.camera_calibration import FoxgloveMsgsCameraCalibrationMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point_cloud import FoxgloveMsgsPointCloudMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.text_primitive import FoxgloveMsgsTextPrimitiveMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.vector3 import FoxgloveMsgsVector3Mapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.location_fixes import FoxgloveMsgsLocationFixesMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.quaternion import FoxgloveMsgsQuaternionMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.packed_element_field import FoxgloveMsgsPackedElementFieldMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point2 import FoxgloveMsgsPoint2Mapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.laser_scan import FoxgloveMsgsLaserScanMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose import FoxgloveMsgsPoseMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.color import FoxgloveMsgsColorMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.text_annotation import FoxgloveMsgsTextAnnotationMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.key_value_pair import FoxgloveMsgsKeyValuePairMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.raw_audio import FoxgloveMsgsRawAudioMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.circle_annotation import FoxgloveMsgsCircleAnnotationMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.grid import FoxgloveMsgsGridMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.joint_state import FoxgloveMsgsJointStateMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.compressed_audio import FoxgloveMsgsCompressedAudioMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.points_annotation import FoxgloveMsgsPointsAnnotationMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.voxel_grid import FoxgloveMsgsVoxelGridMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.scene_entity_deletion import FoxgloveMsgsSceneEntityDeletionMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.event import FoxgloveMsgsEventMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.pose_in_frame import FoxgloveMsgsPoseInFrameMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.log import FoxgloveMsgsLogMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.geo_json import FoxgloveMsgsGeoJsonMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.arrow_primitive import FoxgloveMsgsArrowPrimitiveMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.cube_primitive import FoxgloveMsgsCubePrimitiveMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.compressed_video import FoxgloveMsgsCompressedVideoMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.image_annotations import FoxgloveMsgsImageAnnotationsMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.line_primitive import FoxgloveMsgsLinePrimitiveMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.joint_states import FoxgloveMsgsJointStatesMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.frame_transform import FoxgloveMsgsFrameTransformMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.scene_entity import FoxgloveMsgsSceneEntityMapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.point3 import FoxgloveMsgsPoint3Mapper
from robot_bus.ros2_bridge.mappers.foxglove_msgs.model_primitive import FoxgloveMsgsModelPrimitiveMapper
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
from robot_bus.ros2_bridge.mappers.geometry_msgs.velocity_with_covariance_stamped import GeometryMsgsVelocityWithCovarianceStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.wrench_stamped import GeometryMsgsWrenchStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.transform_stamped import GeometryMsgsTransformStampedMapper
from robot_bus.ros2_bridge.mappers.geometry_msgs.polygon_stamped import GeometryMsgsPolygonStampedMapper
from robot_bus.ros2_bridge.mappers.apriltag_msgs.april_tag_detection import ApriltagMsgsAprilTagDetectionMapper
from robot_bus.ros2_bridge.mappers.apriltag_msgs.point import ApriltagMsgsPointMapper
from robot_bus.ros2_bridge.mappers.apriltag_msgs.april_tag_detection_array import ApriltagMsgsAprilTagDetectionArrayMapper
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
from robot_bus.ros2_bridge.mappers.control_msgs.joint_trajectory_controller_state import ControlMsgsJointTrajectoryControllerStateMapper
from robot_bus.ros2_bridge.mappers.control_msgs.joint_component_tolerance import ControlMsgsJointComponentToleranceMapper
from robot_bus.ros2_bridge.mappers.control_msgs.motion_primitive import ControlMsgsMotionPrimitiveMapper
from robot_bus.ros2_bridge.mappers.control_msgs.single_dof_state import ControlMsgsSingleDofStateMapper
from robot_bus.ros2_bridge.mappers.control_msgs.multi_dof_command import ControlMsgsMultiDofCommandMapper
from robot_bus.ros2_bridge.mappers.control_msgs.gripper_command import ControlMsgsGripperCommandMapper
from robot_bus.ros2_bridge.mappers.control_msgs.single_dof_state_stamped import ControlMsgsSingleDofStateStampedMapper
from robot_bus.ros2_bridge.mappers.control_msgs.mecanum_drive_controller_state import ControlMsgsMecanumDriveControllerStateMapper
from robot_bus.ros2_bridge.mappers.control_msgs.pid_state import ControlMsgsPidStateMapper
from robot_bus.ros2_bridge.mappers.control_msgs.motion_primitive_sequence import ControlMsgsMotionPrimitiveSequenceMapper
from robot_bus.ros2_bridge.mappers.control_msgs.dynamic_interface_group_values import ControlMsgsDynamicInterfaceGroupValuesMapper
from robot_bus.ros2_bridge.mappers.control_msgs.multi_dof_state_stamped import ControlMsgsMultiDofStateStampedMapper
from robot_bus.ros2_bridge.mappers.control_msgs.joint_jog import ControlMsgsJointJogMapper
from robot_bus.ros2_bridge.mappers.control_msgs.steering_controller_status import ControlMsgsSteeringControllerStatusMapper
from robot_bus.ros2_bridge.mappers.control_msgs.joint_controller_state import ControlMsgsJointControllerStateMapper
from robot_bus.ros2_bridge.mappers.control_msgs.joint_tolerance import ControlMsgsJointToleranceMapper
from robot_bus.ros2_bridge.mappers.control_msgs.dynamic_joint_state import ControlMsgsDynamicJointStateMapper
from robot_bus.ros2_bridge.mappers.control_msgs.admittance_controller_state import ControlMsgsAdmittanceControllerStateMapper
from robot_bus.ros2_bridge.mappers.control_msgs.interface_value import ControlMsgsInterfaceValueMapper
from robot_bus.ros2_bridge.mappers.control_msgs.motion_argument import ControlMsgsMotionArgumentMapper
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
from robot_bus.ros2_bridge.mappers.nav2_msgs.route_edge import Nav2MsgsRouteEdgeMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.behavior_tree_status_change import Nav2MsgsBehaviorTreeStatusChangeMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.behavior_tree_log import Nav2MsgsBehaviorTreeLogMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.route_node import Nav2MsgsRouteNodeMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.particle import Nav2MsgsParticleMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.costmap import Nav2MsgsCostmapMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.particle_cloud import Nav2MsgsParticleCloudMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.costmap_meta_data import Nav2MsgsCostmapMetaDataMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.voxel_grid import Nav2MsgsVoxelGridMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.route import Nav2MsgsRouteMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.speed_limit import Nav2MsgsSpeedLimitMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.collision_monitor_state import Nav2MsgsCollisionMonitorStateMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.edge_cost import Nav2MsgsEdgeCostMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.costmap_filter_info import Nav2MsgsCostmapFilterInfoMapper
from robot_bus.ros2_bridge.mappers.nav2_msgs.missed_waypoint import Nav2MsgsMissedWaypointMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.trajectory_point import NavMsgsTrajectoryPointMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.occupancy_grid import NavMsgsOccupancyGridMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.odometry import NavMsgsOdometryMapper
from robot_bus.ros2_bridge.mappers.nav_msgs.trajectory import NavMsgsTrajectoryMapper
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
    "FoxgloveMsgsSceneUpdateMapper",
    "FoxgloveMsgsCylinderPrimitiveMapper",
    "FoxgloveMsgsLocationFixMapper",
    "FoxgloveMsgsRawImageMapper",
    "FoxgloveMsgsPosesInFrameMapper",
    "FoxgloveMsgsOdometryMapper",
    "FoxgloveMsgsCompressedPointCloudMapper",
    "FoxgloveMsgsVector2Mapper",
    "FoxgloveMsgsSpherePrimitiveMapper",
    "FoxgloveMsgsFrameTransformsMapper",
    "FoxgloveMsgsTriangleListPrimitiveMapper",
    "FoxgloveMsgsCompressedImageMapper",
    "FoxgloveMsgsPoint3InFrameMapper",
    "FoxgloveMsgsCameraCalibrationMapper",
    "FoxgloveMsgsPointCloudMapper",
    "FoxgloveMsgsTextPrimitiveMapper",
    "FoxgloveMsgsVector3Mapper",
    "FoxgloveMsgsLocationFixesMapper",
    "FoxgloveMsgsQuaternionMapper",
    "FoxgloveMsgsPackedElementFieldMapper",
    "FoxgloveMsgsPoint2Mapper",
    "FoxgloveMsgsLaserScanMapper",
    "FoxgloveMsgsPoseMapper",
    "FoxgloveMsgsColorMapper",
    "FoxgloveMsgsTextAnnotationMapper",
    "FoxgloveMsgsKeyValuePairMapper",
    "FoxgloveMsgsRawAudioMapper",
    "FoxgloveMsgsCircleAnnotationMapper",
    "FoxgloveMsgsGridMapper",
    "FoxgloveMsgsJointStateMapper",
    "FoxgloveMsgsCompressedAudioMapper",
    "FoxgloveMsgsPointsAnnotationMapper",
    "FoxgloveMsgsVoxelGridMapper",
    "FoxgloveMsgsSceneEntityDeletionMapper",
    "FoxgloveMsgsEventMapper",
    "FoxgloveMsgsPoseInFrameMapper",
    "FoxgloveMsgsLogMapper",
    "FoxgloveMsgsGeoJsonMapper",
    "FoxgloveMsgsArrowPrimitiveMapper",
    "FoxgloveMsgsCubePrimitiveMapper",
    "FoxgloveMsgsCompressedVideoMapper",
    "FoxgloveMsgsImageAnnotationsMapper",
    "FoxgloveMsgsLinePrimitiveMapper",
    "FoxgloveMsgsJointStatesMapper",
    "FoxgloveMsgsFrameTransformMapper",
    "FoxgloveMsgsSceneEntityMapper",
    "FoxgloveMsgsPoint3Mapper",
    "FoxgloveMsgsModelPrimitiveMapper",
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
    "GeometryMsgsVelocityWithCovarianceStampedMapper",
    "GeometryMsgsWrenchStampedMapper",
    "GeometryMsgsTransformStampedMapper",
    "GeometryMsgsPolygonStampedMapper",
    "ApriltagMsgsAprilTagDetectionMapper",
    "ApriltagMsgsPointMapper",
    "ApriltagMsgsAprilTagDetectionArrayMapper",
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
    "ControlMsgsJointTrajectoryControllerStateMapper",
    "ControlMsgsJointComponentToleranceMapper",
    "ControlMsgsMotionPrimitiveMapper",
    "ControlMsgsSingleDofStateMapper",
    "ControlMsgsMultiDofCommandMapper",
    "ControlMsgsGripperCommandMapper",
    "ControlMsgsSingleDofStateStampedMapper",
    "ControlMsgsMecanumDriveControllerStateMapper",
    "ControlMsgsPidStateMapper",
    "ControlMsgsMotionPrimitiveSequenceMapper",
    "ControlMsgsDynamicInterfaceGroupValuesMapper",
    "ControlMsgsMultiDofStateStampedMapper",
    "ControlMsgsJointJogMapper",
    "ControlMsgsSteeringControllerStatusMapper",
    "ControlMsgsJointControllerStateMapper",
    "ControlMsgsJointToleranceMapper",
    "ControlMsgsDynamicJointStateMapper",
    "ControlMsgsAdmittanceControllerStateMapper",
    "ControlMsgsInterfaceValueMapper",
    "ControlMsgsMotionArgumentMapper",
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
    "Nav2MsgsRouteEdgeMapper",
    "Nav2MsgsBehaviorTreeStatusChangeMapper",
    "Nav2MsgsBehaviorTreeLogMapper",
    "Nav2MsgsRouteNodeMapper",
    "Nav2MsgsParticleMapper",
    "Nav2MsgsCostmapMapper",
    "Nav2MsgsParticleCloudMapper",
    "Nav2MsgsCostmapMetaDataMapper",
    "Nav2MsgsVoxelGridMapper",
    "Nav2MsgsRouteMapper",
    "Nav2MsgsSpeedLimitMapper",
    "Nav2MsgsCollisionMonitorStateMapper",
    "Nav2MsgsEdgeCostMapper",
    "Nav2MsgsCostmapFilterInfoMapper",
    "Nav2MsgsMissedWaypointMapper",
    "NavMsgsTrajectoryPointMapper",
    "NavMsgsOccupancyGridMapper",
    "NavMsgsOdometryMapper",
    "NavMsgsTrajectoryMapper",
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
