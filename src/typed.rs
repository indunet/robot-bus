//! ROS 2–style service / action type bindings for typed Node APIs.
//!
//! Prost generates Request/Response and Goal/Feedback/Result as separate
//! messages. Marker types (e.g. [`crate::std_srvs::srv::v1::SetBool`]) pair them
//! via these traits so `create_client::<SetBool>` matches rclrs / rclcpp.

use prost::Message;

/// ROS 2–style service type: pairs a Request and Response message.
pub trait Service: Send + Sync + 'static {
    type Request: Message + Default + Send + Sync + 'static;
    type Response: Message + Default + Send + Sync + 'static;
}

/// ROS 2–style action type: pairs Goal, Feedback, and Result messages.
pub trait Action: Send + Sync + 'static {
    type Goal: Message + Default + Send + Sync + 'static;
    type Feedback: Message + Default + Send + Sync + 'static;
    type Result: Message + Default + Send + Sync + 'static;
}

/// Typed action execution outcome (maps to FEEDBACK* then RESULT on the wire).
pub struct ActionOutcome<A: Action> {
    pub feedbacks: Vec<A::Feedback>,
    pub result: A::Result,
}
