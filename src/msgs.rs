//! Generated ROS 2–style protobuf message and service types from [`proto/`](../../proto).
//!
//! Layout mirrors ROS packages: `proto/<pkg>/{msg|srv}/v1/*.proto`, exposed as
//! `robot_bus::msgs::<pkg>::{msg|srv}::v1::...`.
//!
//! The bus still treats wire bodies as opaque bytes; this module is for
//! callers that want typed encode/decode of those payloads.
//!
//! Service (`.srv`) definitions are Request/Response message pairs (e.g.
//! `std_srvs::srv::v1::SetBoolRequest`), not gRPC `service`/`rpc` stubs.

pub mod builtin_interfaces {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/builtin_interfaces.msg.v1.rs"));
        }
    }
}

pub mod std_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/std_msgs.msg.v1.rs"));
        }
    }
}

pub mod std_srvs {
    pub mod srv {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/std_srvs.srv.v1.rs"));
        }
    }
}

pub mod geometry_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/geometry_msgs.msg.v1.rs"));
        }
    }
}

pub mod sensor_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/sensor_msgs.msg.v1.rs"));
        }
    }
}

pub mod nav_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/nav_msgs.msg.v1.rs"));
        }
    }
    pub mod srv {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/nav_msgs.srv.v1.rs"));
        }
    }
}

pub mod tf2_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/tf2_msgs.msg.v1.rs"));
        }
    }
}

pub mod unique_identifier_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/unique_identifier_msgs.msg.v1.rs"));
        }
    }
}

pub mod diagnostic_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/diagnostic_msgs.msg.v1.rs"));
        }
    }
}

pub mod trajectory_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/trajectory_msgs.msg.v1.rs"));
        }
    }
}

pub mod shape_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/shape_msgs.msg.v1.rs"));
        }
    }
}

pub mod visualization_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/visualization_msgs.msg.v1.rs"));
        }
    }
}

pub mod control_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/control_msgs.msg.v1.rs"));
        }
    }
}

pub mod nav2_msgs {
    pub mod msg {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/nav2_msgs.msg.v1.rs"));
        }
    }
}
