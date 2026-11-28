//! ROS 2–style service / action type bindings for typed Node APIs.
//!
//! Prost generates Request/Response and Goal/Feedback/Result as separate
//! messages. Marker types (e.g. [`crate::std_srvs::srv::v1::SetBool`]) pair them
//! via these traits so `create_client::<SetBool>` matches rclrs / rclcpp.

use prost::{Message, Name};

/// Failure from [`WireMessage::from_wire`].
#[derive(Debug)]
pub struct WireError {
    message: String,
}

impl WireError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WireError {}

/// Bytes a typed publisher, subscription, service, or action can carry.
///
/// Generated protobuf messages implement this. A codec that is not protobuf
/// implements it directly. The broker never inspects the bytes.
pub trait WireMessage: Send + Sync + 'static {
    fn to_wire(&self) -> Vec<u8>;
    fn from_wire(bytes: &[u8]) -> Result<Self, WireError>
    where
        Self: Sized;
    /// Protobuf full name when this value has one. Used for console registration.
    fn wire_type_name() -> Option<String> {
        None
    }
}

impl<T> WireMessage for T
where
    T: Message + Name + Default + Send + Sync + 'static,
{
    fn to_wire(&self) -> Vec<u8> {
        Message::encode_to_vec(self)
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, WireError> {
        Message::decode(bytes).map_err(|err| WireError::new(err.to_string()))
    }

    fn wire_type_name() -> Option<String> {
        Some(T::full_name())
    }
}

/// ROS 2–style service type: pairs a Request and Response message.
pub trait Service: Send + Sync + 'static {
    type Request: WireMessage + Default;
    type Response: WireMessage + Default;
}

/// ROS 2–style action type: pairs Goal, Feedback, and Result messages.
pub trait Action: Send + Sync + 'static {
    type Goal: WireMessage + Default;
    type Feedback: WireMessage + Default;
    type Result: WireMessage + Default;
}

/// Typed action execution outcome (maps to FEEDBACK* then RESULT on the wire).
pub struct ActionOutcome<A: Action> {
    pub feedbacks: Vec<A::Feedback>,
    pub result: A::Result,
}
