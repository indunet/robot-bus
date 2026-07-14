//! Generated ROS 2–style protobuf message types from [`proto/`](../../proto).
//!
//! The bus still treats wire bodies as opaque bytes; this module is for
//! callers that want typed encode/decode of those payloads.

pub mod builtin_interfaces {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/builtin_interfaces.v1.rs"));
    }
}

pub mod std_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/std_msgs.v1.rs"));
    }
}

pub mod geometry_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/geometry_msgs.v1.rs"));
    }
}

pub mod sensor_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/sensor_msgs.v1.rs"));
    }
}

pub mod nav_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/nav_msgs.v1.rs"));
    }
}
