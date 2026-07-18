//! Generated ROS 2–style protobuf message and service types from [`proto/`](../../proto).
//!
//! Internal layout mirrors ROS packages under this module so prost `super::…`
//! paths resolve. Public API re-exports each package at the crate root:
//! `robot_bus::<pkg>::{msg|srv|action}::v1::...`.
//!
//! Stubs live under [`generated/`](generated/) (gitignored). Run
//! `just gen-rust` / `scripts/generate_rust_msgs.py` before building; CI and
//! publish workflows generate them into the package so crates.io consumers do
//! not need `protoc`.
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
            include!("generated/builtin_interfaces.msg.v1.rs");
        }
    }
}

pub mod std_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/std_msgs.msg.v1.rs");
        }
    }
}

pub mod std_srvs {
    pub mod srv {
        pub mod v1 {
            include!("generated/std_srvs.srv.v1.rs");

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
            include!("generated/geometry_msgs.msg.v1.rs");
        }
    }
}

pub mod sensor_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/sensor_msgs.msg.v1.rs");
        }
    }
}

pub mod nav_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/nav_msgs.msg.v1.rs");
        }
    }
    pub mod srv {
        pub mod v1 {
            include!("generated/nav_msgs.srv.v1.rs");

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
        include!("generated/robot_bus_interface.action.v1.rs");

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
            include!("generated/tf2_msgs.msg.v1.rs");
        }
    }
}

pub mod unique_identifier_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/unique_identifier_msgs.msg.v1.rs");
        }
    }
}

pub mod diagnostic_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/diagnostic_msgs.msg.v1.rs");
        }
    }
}

pub mod foxglove_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/foxglove_msgs.msg.v1.rs");
        }
    }
}

pub mod trajectory_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/trajectory_msgs.msg.v1.rs");
        }
    }
}

pub mod shape_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/shape_msgs.msg.v1.rs");
        }
    }
}

pub mod visualization_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/visualization_msgs.msg.v1.rs");
        }
    }
}

pub mod control_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/control_msgs.msg.v1.rs");
        }
    }
}

pub mod nav2_msgs {
    pub mod msg {
        pub mod v1 {
            include!("generated/nav2_msgs.msg.v1.rs");
        }
    }
}
