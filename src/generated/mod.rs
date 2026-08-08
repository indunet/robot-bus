//! Generated ROS 2–style protobuf message and service types from [`proto/`](../../proto).
//!
//! On-disk stubs mirror C++/Python: `generated/<pkg>/{msg|srv|action}/v1/<stem>.rs`
//! (gitignored; one file per `.proto`). Run `just gen-rust` before building.
//!
//! Public API re-exports each package at the crate root:
//! `robot_bus::<pkg>::{msg|srv|action}::v1::...`.
//!
//! The bus still treats wire bodies as opaque bytes; these modules are for
//! callers that want typed encode/decode of those payloads.
//!
//! Service (`.srv`) definitions are Request/Response message pairs (e.g.
//! `std_srvs::srv::v1::SetBoolRequest`), not gRPC `service`/`rpc` stubs.
//! Marker types (e.g. [`std_srvs::srv::v1::SetBool`]) implement [`crate::typed::Service`].
//! Action markers (e.g. [`crate::action::v1::Fibonacci`]) implement
//! [`crate::typed::Action`].

pub mod apriltag_msgs {
    pub mod msg {
        pub mod v1 {
            include!("apriltag_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod builtin_interfaces {
    pub mod msg {
        pub mod v1 {
            include!("builtin_interfaces/msg/v1/_includes.rs");
        }
    }
}

pub mod std_msgs {
    pub mod msg {
        pub mod v1 {
            include!("std_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod std_srvs {
    pub mod srv {
        pub mod v1 {
            include!("std_srvs/srv/v1/_includes.rs");

            use crate::typed::Service;

            /// ROS 2 `std_srvs/srv/Empty` type marker.
            pub struct Empty;
            impl Service for Empty {
                type Request = EmptyRequest;
                type Response = EmptyResponse;
            }

            /// ROS 2 `std_srvs/srv/Trigger` type marker.
            pub struct Trigger;
            impl Service for Trigger {
                type Request = TriggerRequest;
                type Response = TriggerResponse;
            }

            /// ROS 2 `std_srvs/srv/SetBool` type marker.
            pub struct SetBool;
            impl Service for SetBool {
                type Request = SetBoolRequest;
                type Response = SetBoolResponse;
            }
        }
    }
}

pub mod geometry_msgs {
    pub mod msg {
        pub mod v1 {
            include!("geometry_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod sensor_msgs {
    pub mod msg {
        pub mod v1 {
            include!("sensor_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod nav_msgs {
    pub mod msg {
        pub mod v1 {
            include!("nav_msgs/msg/v1/_includes.rs");
        }
    }
    pub mod srv {
        pub mod v1 {
            include!("nav_msgs/srv/v1/_includes.rs");

            use crate::typed::Service;

            /// ROS 2 `nav_msgs/srv/GetMap` type marker.
            pub struct GetMap;
            impl Service for GetMap {
                type Request = GetMapRequest;
                type Response = GetMapResponse;
            }

            /// ROS 2 `nav_msgs/srv/GetPlan` type marker.
            pub struct GetPlan;
            impl Service for GetPlan {
                type Request = GetPlanRequest;
                type Response = GetPlanResponse;
            }

            /// ROS 2 `nav_msgs/srv/SetMap` type marker.
            pub struct SetMap;
            impl Service for SetMap {
                type Request = SetMapRequest;
                type Response = SetMapResponse;
            }

            /// ROS 2 `nav_msgs/srv/LoadMap` type marker.
            pub struct LoadMap;
            impl Service for LoadMap {
                type Request = LoadMapRequest;
                type Response = LoadMapResponse;
            }
        }
    }
}

pub mod stereo_msgs {
    pub mod msg {
        pub mod v1 {
            include!("stereo_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod action_msgs {
    pub mod msg {
        pub mod v1 {
            include!("action_msgs/msg/v1/_includes.rs");
        }
    }
    pub mod srv {
        pub mod v1 {
            include!("action_msgs/srv/v1/_includes.rs");

            use crate::typed::Service;

            /// ROS 2 `action_msgs/srv/CancelGoal` type marker.
            pub struct CancelGoal;
            impl Service for CancelGoal {
                type Request = CancelGoalRequest;
                type Response = CancelGoalResponse;
            }
        }
    }
}

pub mod action {
    pub mod v1 {
        include!("robot_bus_interface/action/v1/_includes.rs");

        use crate::typed::Action;

        /// Demo action type marker (`robot_bus_interface/action/Fibonacci`).
        pub struct Fibonacci;
        impl Action for Fibonacci {
            type Goal = FibonacciGoal;
            type Feedback = FibonacciFeedback;
            type Result = FibonacciResult;
        }

        /// 单点导航 (`robot_bus_interface/action/PointNavigation`).
        pub struct PointNavigation;
        impl Action for PointNavigation {
            type Goal = PointNavigationGoal;
            type Feedback = PointNavigationFeedback;
            type Result = PointNavigationResult;
        }

        /// 多途经点导航 (`robot_bus_interface/action/MultiWaypointNavigation`).
        pub struct MultiWaypointNavigation;
        impl Action for MultiWaypointNavigation {
            type Goal = MultiWaypointNavigationGoal;
            type Feedback = MultiWaypointNavigationFeedback;
            type Result = MultiWaypointNavigationResult;
        }
    }
}

/// robot-bus interface messages (`robot_bus_interface.msg.v1`).
pub mod robot_bus_interface {
    pub mod msg {
        pub mod v1 {
            include!("robot_bus_interface/msg/v1/_includes.rs");
        }
    }
    pub mod srv {
        pub mod v1 {
            include!("robot_bus_interface/srv/v1/_includes.rs");

            use crate::typed::Service;

            /// 复位 (`robot_bus_interface/srv/Reset`).
            pub struct Reset;
            impl Service for Reset {
                type Request = ResetRequest;
                type Response = ResetResponse;
            }
        }
    }
}

pub mod tf2_msgs {
    pub mod msg {
        pub mod v1 {
            include!("tf2_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod unique_identifier_msgs {
    pub mod msg {
        pub mod v1 {
            include!("unique_identifier_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod diagnostic_msgs {
    pub mod msg {
        pub mod v1 {
            include!("diagnostic_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod foxglove_msgs {
    pub mod msg {
        pub mod v1 {
            include!("foxglove_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod trajectory_msgs {
    pub mod msg {
        pub mod v1 {
            include!("trajectory_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod shape_msgs {
    pub mod msg {
        pub mod v1 {
            include!("shape_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod visualization_msgs {
    pub mod msg {
        pub mod v1 {
            include!("visualization_msgs/msg/v1/_includes.rs");
        }
    }
}

pub mod control_msgs {
    pub mod msg {
        pub mod v1 {
            include!("control_msgs/msg/v1/_includes.rs");
        }
    }
    pub mod action {
        pub mod v1 {
            include!("control_msgs/action/v1/_includes.rs");

            use crate::typed::Action;

            /// ROS 2 `control_msgs/action/FollowJointTrajectory` type marker.
            pub struct FollowJointTrajectory;
            impl Action for FollowJointTrajectory {
                type Goal = FollowJointTrajectoryGoal;
                type Feedback = FollowJointTrajectoryFeedback;
                type Result = FollowJointTrajectoryResult;
            }

            /// ROS 2 `control_msgs/action/GripperCommand` type marker.
            pub struct GripperCommand;
            impl Action for GripperCommand {
                type Goal = GripperCommandGoal;
                type Feedback = GripperCommandFeedback;
                type Result = GripperCommandResult;
            }

            /// ROS 2 `control_msgs/action/PointHead` type marker.
            pub struct PointHead;
            impl Action for PointHead {
                type Goal = PointHeadGoal;
                type Feedback = PointHeadFeedback;
                type Result = PointHeadResult;
            }

            /// ROS 2 `control_msgs/action/SingleJointPosition` type marker.
            pub struct SingleJointPosition;
            impl Action for SingleJointPosition {
                type Goal = SingleJointPositionGoal;
                type Feedback = SingleJointPositionFeedback;
                type Result = SingleJointPositionResult;
            }

            /// ROS 2 `control_msgs/action/JointTrajectory` type marker.
            pub struct JointTrajectory;
            impl Action for JointTrajectory {
                type Goal = JointTrajectoryGoal;
                type Feedback = JointTrajectoryFeedback;
                type Result = JointTrajectoryResult;
            }

            /// ROS 2 `control_msgs/action/ParallelGripperCommand` type marker.
            pub struct ParallelGripperCommand;
            impl Action for ParallelGripperCommand {
                type Goal = ParallelGripperCommandGoal;
                type Feedback = ParallelGripperCommandFeedback;
                type Result = ParallelGripperCommandResult;
            }

            /// ROS 2 `control_msgs/action/ExecuteMotionPrimitiveSequence` type marker.
            pub struct ExecuteMotionPrimitiveSequence;
            impl Action for ExecuteMotionPrimitiveSequence {
                type Goal = ExecuteMotionPrimitiveSequenceGoal;
                type Feedback = ExecuteMotionPrimitiveSequenceFeedback;
                type Result = ExecuteMotionPrimitiveSequenceResult;
            }
        }
    }
}

pub mod nav2_msgs {
    pub mod msg {
        pub mod v1 {
            include!("nav2_msgs/msg/v1/_includes.rs");
        }
    }
    pub mod action {
        pub mod v1 {
            include!("nav2_msgs/action/v1/_includes.rs");

            use crate::typed::Action;

            /// ROS 2 `nav2_msgs/action/NavigateToPose` type marker.
            pub struct NavigateToPose;
            impl Action for NavigateToPose {
                type Goal = NavigateToPoseGoal;
                type Feedback = NavigateToPoseFeedback;
                type Result = NavigateToPoseResult;
            }

            /// ROS 2 `nav2_msgs/action/NavigateThroughPoses` type marker.
            pub struct NavigateThroughPoses;
            impl Action for NavigateThroughPoses {
                type Goal = NavigateThroughPosesGoal;
                type Feedback = NavigateThroughPosesFeedback;
                type Result = NavigateThroughPosesResult;
            }

            /// ROS 2 `nav2_msgs/action/FollowPath` type marker.
            pub struct FollowPath;
            impl Action for FollowPath {
                type Goal = FollowPathGoal;
                type Feedback = FollowPathFeedback;
                type Result = FollowPathResult;
            }

            /// ROS 2 `nav2_msgs/action/ComputePathToPose` type marker.
            pub struct ComputePathToPose;
            impl Action for ComputePathToPose {
                type Goal = ComputePathToPoseGoal;
                type Feedback = ComputePathToPoseFeedback;
                type Result = ComputePathToPoseResult;
            }

            /// ROS 2 `nav2_msgs/action/ComputePathThroughPoses` type marker.
            pub struct ComputePathThroughPoses;
            impl Action for ComputePathThroughPoses {
                type Goal = ComputePathThroughPosesGoal;
                type Feedback = ComputePathThroughPosesFeedback;
                type Result = ComputePathThroughPosesResult;
            }

            /// ROS 2 `nav2_msgs/action/Spin` type marker.
            pub struct Spin;
            impl Action for Spin {
                type Goal = SpinGoal;
                type Feedback = SpinFeedback;
                type Result = SpinResult;
            }

            /// ROS 2 `nav2_msgs/action/BackUp` type marker.
            pub struct BackUp;
            impl Action for BackUp {
                type Goal = BackUpGoal;
                type Feedback = BackUpFeedback;
                type Result = BackUpResult;
            }

            /// ROS 2 `nav2_msgs/action/Wait` type marker.
            pub struct Wait;
            impl Action for Wait {
                type Goal = WaitGoal;
                type Feedback = WaitFeedback;
                type Result = WaitResult;
            }

            /// ROS 2 `nav2_msgs/action/FollowWaypoints` type marker.
            pub struct FollowWaypoints;
            impl Action for FollowWaypoints {
                type Goal = FollowWaypointsGoal;
                type Feedback = FollowWaypointsFeedback;
                type Result = FollowWaypointsResult;
            }

            /// ROS 2 `nav2_msgs/action/SmoothPath` type marker.
            pub struct SmoothPath;
            impl Action for SmoothPath {
                type Goal = SmoothPathGoal;
                type Feedback = SmoothPathFeedback;
                type Result = SmoothPathResult;
            }

            /// ROS 2 `nav2_msgs/action/DriveOnHeading` type marker.
            pub struct DriveOnHeading;
            impl Action for DriveOnHeading {
                type Goal = DriveOnHeadingGoal;
                type Feedback = DriveOnHeadingFeedback;
                type Result = DriveOnHeadingResult;
            }

            /// ROS 2 `nav2_msgs/action/AssistedTeleop` type marker.
            pub struct AssistedTeleop;
            impl Action for AssistedTeleop {
                type Goal = AssistedTeleopGoal;
                type Feedback = AssistedTeleopFeedback;
                type Result = AssistedTeleopResult;
            }
        }
    }
}
