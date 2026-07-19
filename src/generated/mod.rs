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
}

pub mod nav2_msgs {
    pub mod msg {
        pub mod v1 {
            include!("nav2_msgs/msg/v1/_includes.rs");
        }
    }
}
