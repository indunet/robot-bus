//! Field-complete stubs for rclrs 0.8 `use_ros_shim` (no C typesupport).

use std::borrow::Cow;

use rosidl_runtime_rs::{Message, Sequence, Service, String as RosString};

pub(crate) fn shim_null_ts() -> *const std::ffi::c_void {
    std::ptr::null()
}

macro_rules! impl_rmw {
    ($name:ident) => {
        impl ::rosidl_runtime_rs::Message for $name {
            type RmwMsg = Self;
            fn into_rmw_message(
                msg_cow: ::std::borrow::Cow<'_, Self>,
            ) -> ::std::borrow::Cow<'_, Self::RmwMsg> {
                msg_cow
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                msg
            }
        }
        impl ::rosidl_runtime_rs::RmwMessage for $name {
            const TYPE_NAME: &'static str = stringify!($name);
            fn get_type_support() -> *const ::std::ffi::c_void {
                $crate::shim::shim_null_ts()
            }
        }
        impl ::rosidl_runtime_rs::SequenceAlloc for $name {
            fn sequence_init(
                _: &mut ::rosidl_runtime_rs::Sequence<Self>,
                _: usize,
            ) -> bool {
                true
            }
            fn sequence_fini(_: &mut ::rosidl_runtime_rs::Sequence<Self>) {}
            fn sequence_copy(
                _: &::rosidl_runtime_rs::Sequence<Self>,
                _: &mut ::rosidl_runtime_rs::Sequence<Self>,
            ) -> bool {
                true
            }
        }
    };
}

pub mod unique_identifier_msgs {
    use super::*;

    pub mod msg {
        use super::*;

        #[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
        pub struct UUID {
            pub uuid: [u8; 16],
        }

        impl Message for UUID {
            type RmwMsg = rmw::UUID;
            fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                Cow::Owned(rmw::UUID {
                    uuid: msg_cow.into_owned().uuid,
                })
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                Self { uuid: msg.uuid }
            }
        }

        pub mod rmw {
            #[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
            pub struct UUID {
                pub uuid: [u8; 16],
            }
            impl_rmw!(UUID);
        }
    }
}

pub mod builtin_interfaces {
    use super::*;

    pub mod msg {
        use super::*;

        #[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
        pub struct Time {
            pub sec: i32,
            pub nanosec: u32,
        }

        impl Message for Time {
            type RmwMsg = rmw::Time;
            fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                let msg = msg_cow.into_owned();
                Cow::Owned(rmw::Time {
                    sec: msg.sec,
                    nanosec: msg.nanosec,
                })
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                Self {
                    sec: msg.sec,
                    nanosec: msg.nanosec,
                }
            }
        }

        pub mod rmw {
            #[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
            pub struct Time {
                pub sec: i32,
                pub nanosec: u32,
            }
            impl_rmw!(Time);
        }
    }
}

pub mod action_msgs {
    use super::*;
    use crate::builtin_interfaces;
    use crate::unique_identifier_msgs;

    pub mod msg {
        use super::*;

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct GoalInfo {
            pub goal_id: unique_identifier_msgs::msg::UUID,
            pub stamp: builtin_interfaces::msg::Time,
        }

        impl Message for GoalInfo {
            type RmwMsg = rmw::GoalInfo;
            fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                let msg = msg_cow.into_owned();
                Cow::Owned(rmw::GoalInfo {
                    goal_id: unique_identifier_msgs::msg::UUID::into_rmw_message(Cow::Owned(
                        msg.goal_id,
                    ))
                    .into_owned(),
                    stamp: builtin_interfaces::msg::Time::into_rmw_message(Cow::Owned(msg.stamp))
                        .into_owned(),
                })
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                Self {
                    goal_id: unique_identifier_msgs::msg::UUID::from_rmw_message(msg.goal_id),
                    stamp: builtin_interfaces::msg::Time::from_rmw_message(msg.stamp),
                }
            }
        }

        pub mod rmw {
            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct GoalInfo {
                pub goal_id: crate::unique_identifier_msgs::msg::rmw::UUID,
                pub stamp: crate::builtin_interfaces::msg::rmw::Time,
            }
            impl_rmw!(GoalInfo);
        }
    }

    pub mod srv {
        use super::*;

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct CancelGoal_Request {
            pub goal_info: super::msg::GoalInfo,
        }

        impl Message for CancelGoal_Request {
            type RmwMsg = rmw::CancelGoal_Request;
            fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                let msg = msg_cow.into_owned();
                Cow::Owned(rmw::CancelGoal_Request {
                    goal_info: super::msg::GoalInfo::into_rmw_message(Cow::Owned(msg.goal_info))
                        .into_owned(),
                })
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                Self {
                    goal_info: super::msg::GoalInfo::from_rmw_message(msg.goal_info),
                }
            }
        }

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct CancelGoal_Response {
            pub return_code: i8,
            pub goals_canceling: Vec<super::msg::GoalInfo>,
        }

        impl Message for CancelGoal_Response {
            type RmwMsg = rmw::CancelGoal_Response;
            fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                let msg = msg_cow.into_owned();
                Cow::Owned(rmw::CancelGoal_Response {
                    return_code: msg.return_code,
                    goals_canceling: msg
                        .goals_canceling
                        .into_iter()
                        .map(|g| super::msg::GoalInfo::into_rmw_message(Cow::Owned(g)).into_owned())
                        .collect(),
                })
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                Self {
                    return_code: msg.return_code,
                    goals_canceling: msg
                        .goals_canceling
                        .into_iter()
                        .map(super::msg::GoalInfo::from_rmw_message)
                        .collect(),
                }
            }
        }

        pub struct CancelGoal;
        impl Service for CancelGoal {
            type Request = CancelGoal_Request;
            type Response = CancelGoal_Response;
            fn get_type_support() -> *const std::ffi::c_void {
                crate::shim::shim_null_ts()
            }
        }

        pub mod rmw {
            use crate::action_msgs::msg::rmw::GoalInfo;
            use rosidl_runtime_rs::Sequence;

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct CancelGoal_Request {
                pub goal_info: GoalInfo,
            }
            impl_rmw!(CancelGoal_Request);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct CancelGoal_Response {
                pub return_code: i8,
                pub goals_canceling: Sequence<GoalInfo>,
            }
            impl_rmw!(CancelGoal_Response);
        }
    }
}

pub mod rosgraph_msgs {
    use super::*;
    use crate::builtin_interfaces;

    pub mod msg {
        use super::*;

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Clock {
            pub clock: builtin_interfaces::msg::Time,
        }

        impl Message for Clock {
            type RmwMsg = rmw::Clock;
            fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                let msg = msg_cow.into_owned();
                Cow::Owned(rmw::Clock {
                    clock: builtin_interfaces::msg::Time::into_rmw_message(Cow::Owned(msg.clock))
                        .into_owned(),
                })
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                Self {
                    clock: builtin_interfaces::msg::Time::from_rmw_message(msg.clock),
                }
            }
        }

        pub mod rmw {
            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct Clock {
                pub clock: crate::builtin_interfaces::msg::rmw::Time,
            }
            impl_rmw!(Clock);
        }
    }
}

pub mod rcl_interfaces {
    use super::*;

    pub mod msg {
        use super::*;

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct ParameterType;

        impl ParameterType {
            pub const PARAMETER_NOT_SET: u8 = 0;
            pub const PARAMETER_BOOL: u8 = 1;
            pub const PARAMETER_INTEGER: u8 = 2;
            pub const PARAMETER_DOUBLE: u8 = 3;
            pub const PARAMETER_STRING: u8 = 4;
            pub const PARAMETER_BYTE_ARRAY: u8 = 5;
            pub const PARAMETER_BOOL_ARRAY: u8 = 6;
            pub const PARAMETER_INTEGER_ARRAY: u8 = 7;
            pub const PARAMETER_DOUBLE_ARRAY: u8 = 8;
            pub const PARAMETER_STRING_ARRAY: u8 = 9;
        }

        impl Message for ParameterType {
            type RmwMsg = rmw::ParameterType;
            fn into_rmw_message(_: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                Cow::Owned(rmw::ParameterType::default())
            }
            fn from_rmw_message(_: Self::RmwMsg) -> Self {
                Self
            }
        }

        pub mod rmw {
            use rosidl_runtime_rs::{BoundedSequence, Sequence, String as RosString};

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct ParameterType {
                pub structure_needs_at_least_one_member: u8,
            }
            impl ParameterType {
                pub const PARAMETER_NOT_SET: u8 = 0;
                pub const PARAMETER_BOOL: u8 = 1;
                pub const PARAMETER_INTEGER: u8 = 2;
                pub const PARAMETER_DOUBLE: u8 = 3;
                pub const PARAMETER_STRING: u8 = 4;
                pub const PARAMETER_BYTE_ARRAY: u8 = 5;
                pub const PARAMETER_BOOL_ARRAY: u8 = 6;
                pub const PARAMETER_INTEGER_ARRAY: u8 = 7;
                pub const PARAMETER_DOUBLE_ARRAY: u8 = 8;
                pub const PARAMETER_STRING_ARRAY: u8 = 9;
            }
            impl_rmw!(ParameterType);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct ParameterValue {
                pub type_: u8,
                pub bool_value: bool,
                pub integer_value: i64,
                pub double_value: f64,
                pub string_value: RosString,
                pub byte_array_value: Sequence<u8>,
                pub bool_array_value: Sequence<bool>,
                pub integer_array_value: Sequence<i64>,
                pub double_array_value: Sequence<f64>,
                pub string_array_value: Sequence<RosString>,
            }
            impl_rmw!(ParameterValue);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct FloatingPointRange {
                pub from_value: f64,
                pub to_value: f64,
                pub step: f64,
            }
            impl_rmw!(FloatingPointRange);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct IntegerRange {
                pub from_value: i64,
                pub to_value: i64,
                pub step: u64,
            }
            impl_rmw!(IntegerRange);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct ParameterDescriptor {
                pub name: RosString,
                pub type_: u8,
                pub description: RosString,
                pub additional_constraints: RosString,
                pub read_only: bool,
                pub dynamic_typing: bool,
                pub floating_point_range: BoundedSequence<FloatingPointRange, 1>,
                pub integer_range: BoundedSequence<IntegerRange, 1>,
            }
            impl_rmw!(ParameterDescriptor);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct Parameter {
                pub name: RosString,
                pub value: ParameterValue,
            }
            impl_rmw!(Parameter);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct SetParametersResult {
                pub successful: bool,
                pub reason: RosString,
            }
            impl_rmw!(SetParametersResult);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct ListParametersResult {
                pub names: Sequence<RosString>,
                pub prefixes: Sequence<RosString>,
            }
            impl_rmw!(ListParametersResult);
        }
    }

    pub mod srv {
        pub mod rmw {
            use super::super::super::*;
            use crate::rcl_interfaces::msg::rmw::*;

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct DescribeParameters_Request {
                pub names: Sequence<RosString>,
            }
            impl_rmw!(DescribeParameters_Request);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct DescribeParameters_Response {
                pub descriptors: Sequence<ParameterDescriptor>,
            }
            impl_rmw!(DescribeParameters_Response);

            pub struct DescribeParameters;
            impl Service for DescribeParameters {
                type Request = DescribeParameters_Request;
                type Response = DescribeParameters_Response;
                fn get_type_support() -> *const std::ffi::c_void {
                    crate::shim::shim_null_ts()
                }
            }

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct GetParameters_Request {
                pub names: Sequence<RosString>,
            }
            impl_rmw!(GetParameters_Request);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct GetParameters_Response {
                pub values: Sequence<ParameterValue>,
            }
            impl_rmw!(GetParameters_Response);

            pub struct GetParameters;
            impl Service for GetParameters {
                type Request = GetParameters_Request;
                type Response = GetParameters_Response;
                fn get_type_support() -> *const std::ffi::c_void {
                    crate::shim::shim_null_ts()
                }
            }

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct GetParameterTypes_Request {
                pub names: Sequence<RosString>,
            }
            impl_rmw!(GetParameterTypes_Request);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct GetParameterTypes_Response {
                pub types: Sequence<u8>,
            }
            impl_rmw!(GetParameterTypes_Response);

            pub struct GetParameterTypes;
            impl Service for GetParameterTypes {
                type Request = GetParameterTypes_Request;
                type Response = GetParameterTypes_Response;
                fn get_type_support() -> *const std::ffi::c_void {
                    crate::shim::shim_null_ts()
                }
            }

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct ListParameters_Request {
                pub prefixes: Sequence<RosString>,
                pub depth: u64,
            }
            impl ListParameters_Request {
                pub const DEPTH_RECURSIVE: u64 = 0;
            }
            impl_rmw!(ListParameters_Request);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct ListParameters_Response {
                pub result: ListParametersResult,
            }
            impl_rmw!(ListParameters_Response);

            pub struct ListParameters;
            impl Service for ListParameters {
                type Request = ListParameters_Request;
                type Response = ListParameters_Response;
                fn get_type_support() -> *const std::ffi::c_void {
                    crate::shim::shim_null_ts()
                }
            }

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct SetParameters_Request {
                pub parameters: Sequence<Parameter>,
            }
            impl_rmw!(SetParameters_Request);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct SetParameters_Response {
                pub results: Sequence<SetParametersResult>,
            }
            impl_rmw!(SetParameters_Response);

            pub struct SetParameters;
            impl Service for SetParameters {
                type Request = SetParameters_Request;
                type Response = SetParameters_Response;
                fn get_type_support() -> *const std::ffi::c_void {
                    crate::shim::shim_null_ts()
                }
            }

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct SetParametersAtomically_Request {
                pub parameters: Sequence<Parameter>,
            }
            impl_rmw!(SetParametersAtomically_Request);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct SetParametersAtomically_Response {
                pub result: SetParametersResult,
            }
            impl_rmw!(SetParametersAtomically_Response);

            pub struct SetParametersAtomically;
            impl Service for SetParametersAtomically {
                type Request = SetParametersAtomically_Request;
                type Response = SetParametersAtomically_Response;
                fn get_type_support() -> *const std::ffi::c_void {
                    crate::shim::shim_null_ts()
                }
            }
        }
    }
}

pub mod example_interfaces {
    use super::*;

    pub mod srv {
        use super::*;

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct AddTwoInts_Request {
            pub a: i64,
            pub b: i64,
        }
        impl Message for AddTwoInts_Request {
            type RmwMsg = rmw::AddTwoInts_Request;
            fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                let msg = msg_cow.into_owned();
                Cow::Owned(rmw::AddTwoInts_Request { a: msg.a, b: msg.b })
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                Self { a: msg.a, b: msg.b }
            }
        }

        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct AddTwoInts_Response {
            pub sum: i64,
        }
        impl Message for AddTwoInts_Response {
            type RmwMsg = rmw::AddTwoInts_Response;
            fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
                Cow::Owned(rmw::AddTwoInts_Response {
                    sum: msg_cow.into_owned().sum,
                })
            }
            fn from_rmw_message(msg: Self::RmwMsg) -> Self {
                Self { sum: msg.sum }
            }
        }

        pub struct AddTwoInts;
        impl Service for AddTwoInts {
            type Request = AddTwoInts_Request;
            type Response = AddTwoInts_Response;
            fn get_type_support() -> *const std::ffi::c_void {
                crate::shim::shim_null_ts()
            }
        }

        pub mod rmw {
            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct AddTwoInts_Request {
                pub a: i64,
                pub b: i64,
            }
            impl_rmw!(AddTwoInts_Request);

            #[derive(Clone, Debug, Default, PartialEq)]
            pub struct AddTwoInts_Response {
                pub sum: i64,
            }
            impl_rmw!(AddTwoInts_Response);
        }
    }

    pub mod action {
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Fibonacci_Goal {
            pub order: i32,
        }
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Fibonacci_Feedback {
            pub sequence: Vec<i32>,
        }
        #[derive(Clone, Debug, Default, PartialEq)]
        pub struct Fibonacci_Result {
            pub sequence: Vec<i32>,
        }
        pub struct Fibonacci;
    }
}
