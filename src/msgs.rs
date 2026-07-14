//! Generated ROS 2–style protobuf message and service types from [`proto/`](../../proto).
//!
//! The bus still treats wire bodies as opaque bytes; this module is for
//! callers that want typed encode/decode of those payloads.
//!
//! Service (`.srv`) definitions are Request/Response message pairs (e.g.
//! `std_srvs::v1::SetBoolRequest`), not gRPC `service`/`rpc` stubs.

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

pub mod std_srvs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/std_srvs.v1.rs"));
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

pub mod tf2_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/tf2_msgs.v1.rs"));
    }
}

pub mod unique_identifier_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/unique_identifier_msgs.v1.rs"));
    }
}

pub mod diagnostic_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/diagnostic_msgs.v1.rs"));
    }
}

pub mod trajectory_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/trajectory_msgs.v1.rs"));
    }
}

pub mod shape_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/shape_msgs.v1.rs"));
    }
}

pub mod visualization_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/visualization_msgs.v1.rs"));
    }
}

pub mod control_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/control_msgs.v1.rs"));
    }
}

pub mod nav2_msgs {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/nav2_msgs.v1.rs"));
    }
}
