//! `std_srvs/srv/{Trigger,SetBool}` bindings for rclrs (system typesupport).

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SetBool_Request {
    pub data: bool,
}

impl Default for SetBool_Request {
    fn default() -> Self {
        <Self as rosidl_runtime_rs::Message>::from_rmw_message(rmw::SetBool_Request::default())
    }
}

impl rosidl_runtime_rs::Message for SetBool_Request {
    type RmwMsg = rmw::SetBool_Request;

    fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
        match msg_cow {
            Cow::Owned(msg) => Cow::Owned(Self::RmwMsg { data: msg.data }),
            Cow::Borrowed(msg) => Cow::Owned(Self::RmwMsg { data: msg.data }),
        }
    }

    fn from_rmw_message(msg: Self::RmwMsg) -> Self {
        Self { data: msg.data }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SetBool_Response {
    pub success: bool,
    pub message: String,
}

impl Default for SetBool_Response {
    fn default() -> Self {
        <Self as rosidl_runtime_rs::Message>::from_rmw_message(rmw::SetBool_Response::default())
    }
}

impl rosidl_runtime_rs::Message for SetBool_Response {
    type RmwMsg = rmw::SetBool_Response;

    fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
        match msg_cow {
            Cow::Owned(msg) => Cow::Owned(Self::RmwMsg {
                success: msg.success,
                message: msg.message.as_str().into(),
            }),
            Cow::Borrowed(msg) => Cow::Owned(Self::RmwMsg {
                success: msg.success,
                message: msg.message.as_str().into(),
            }),
        }
    }

    fn from_rmw_message(msg: Self::RmwMsg) -> Self {
        Self {
            success: msg.success,
            message: msg.message.to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Trigger_Request {
    pub structure_needs_at_least_one_member: u8,
}

impl Default for Trigger_Request {
    fn default() -> Self {
        <Self as rosidl_runtime_rs::Message>::from_rmw_message(rmw::Trigger_Request::default())
    }
}

impl rosidl_runtime_rs::Message for Trigger_Request {
    type RmwMsg = rmw::Trigger_Request;

    fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
        match msg_cow {
            Cow::Owned(msg) => Cow::Owned(Self::RmwMsg {
                structure_needs_at_least_one_member: msg.structure_needs_at_least_one_member,
            }),
            Cow::Borrowed(msg) => Cow::Owned(Self::RmwMsg {
                structure_needs_at_least_one_member: msg.structure_needs_at_least_one_member,
            }),
        }
    }

    fn from_rmw_message(msg: Self::RmwMsg) -> Self {
        Self {
            structure_needs_at_least_one_member: msg.structure_needs_at_least_one_member,
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Trigger_Response {
    pub success: bool,
    pub message: String,
}

impl Default for Trigger_Response {
    fn default() -> Self {
        <Self as rosidl_runtime_rs::Message>::from_rmw_message(rmw::Trigger_Response::default())
    }
}

impl rosidl_runtime_rs::Message for Trigger_Response {
    type RmwMsg = rmw::Trigger_Response;

    fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
        match msg_cow {
            Cow::Owned(msg) => Cow::Owned(Self::RmwMsg {
                success: msg.success,
                message: msg.message.as_str().into(),
            }),
            Cow::Borrowed(msg) => Cow::Owned(Self::RmwMsg {
                success: msg.success,
                message: msg.message.as_str().into(),
            }),
        }
    }

    fn from_rmw_message(msg: Self::RmwMsg) -> Self {
        Self {
            success: msg.success,
            message: msg.message.to_string(),
        }
    }
}

#[link(name = "std_srvs__rosidl_typesupport_c")]
unsafe extern "C" {
    fn rosidl_typesupport_c__get_service_type_support_handle__std_srvs__srv__SetBool()
    -> *const std::ffi::c_void;
    fn rosidl_typesupport_c__get_service_type_support_handle__std_srvs__srv__Trigger()
    -> *const std::ffi::c_void;
}

/// Corresponds to `std_srvs/srv/SetBool`.
pub struct SetBool;

impl rosidl_runtime_rs::Service for SetBool {
    type Request = SetBool_Request;
    type Response = SetBool_Response;

    fn get_type_support() -> *const std::ffi::c_void {
        // SAFETY: typesupport handle getter has no preconditions.
        unsafe { rosidl_typesupport_c__get_service_type_support_handle__std_srvs__srv__SetBool() }
    }
}

/// Corresponds to `std_srvs/srv/Trigger`.
pub struct Trigger;

impl rosidl_runtime_rs::Service for Trigger {
    type Request = Trigger_Request;
    type Response = Trigger_Response;

    fn get_type_support() -> *const std::ffi::c_void {
        // SAFETY: typesupport handle getter has no preconditions.
        unsafe { rosidl_typesupport_c__get_service_type_support_handle__std_srvs__srv__Trigger() }
    }
}

pub mod rmw {
    use std::borrow::Cow;

    #[link(name = "std_srvs__rosidl_typesupport_c")]
    unsafe extern "C" {
        fn rosidl_typesupport_c__get_message_type_support_handle__std_srvs__srv__SetBool_Request()
        -> *const std::ffi::c_void;
        fn rosidl_typesupport_c__get_message_type_support_handle__std_srvs__srv__SetBool_Response()
        -> *const std::ffi::c_void;
        fn rosidl_typesupport_c__get_message_type_support_handle__std_srvs__srv__Trigger_Request()
        -> *const std::ffi::c_void;
        fn rosidl_typesupport_c__get_message_type_support_handle__std_srvs__srv__Trigger_Response()
        -> *const std::ffi::c_void;
        fn rosidl_typesupport_c__get_service_type_support_handle__std_srvs__srv__SetBool()
        -> *const std::ffi::c_void;
        fn rosidl_typesupport_c__get_service_type_support_handle__std_srvs__srv__Trigger()
        -> *const std::ffi::c_void;
    }

    #[link(name = "std_srvs__rosidl_generator_c")]
    unsafe extern "C" {
        fn std_srvs__srv__SetBool_Request__init(msg: *mut SetBool_Request) -> bool;
        fn std_srvs__srv__SetBool_Request__Sequence__init(
            seq: *mut rosidl_runtime_rs::Sequence<SetBool_Request>,
            size: usize,
        ) -> bool;
        fn std_srvs__srv__SetBool_Request__Sequence__fini(
            seq: *mut rosidl_runtime_rs::Sequence<SetBool_Request>,
        );
        fn std_srvs__srv__SetBool_Request__Sequence__copy(
            in_seq: &rosidl_runtime_rs::Sequence<SetBool_Request>,
            out_seq: *mut rosidl_runtime_rs::Sequence<SetBool_Request>,
        ) -> bool;

        fn std_srvs__srv__SetBool_Response__init(msg: *mut SetBool_Response) -> bool;
        fn std_srvs__srv__SetBool_Response__Sequence__init(
            seq: *mut rosidl_runtime_rs::Sequence<SetBool_Response>,
            size: usize,
        ) -> bool;
        fn std_srvs__srv__SetBool_Response__Sequence__fini(
            seq: *mut rosidl_runtime_rs::Sequence<SetBool_Response>,
        );
        fn std_srvs__srv__SetBool_Response__Sequence__copy(
            in_seq: &rosidl_runtime_rs::Sequence<SetBool_Response>,
            out_seq: *mut rosidl_runtime_rs::Sequence<SetBool_Response>,
        ) -> bool;

        fn std_srvs__srv__Trigger_Request__init(msg: *mut Trigger_Request) -> bool;
        fn std_srvs__srv__Trigger_Request__Sequence__init(
            seq: *mut rosidl_runtime_rs::Sequence<Trigger_Request>,
            size: usize,
        ) -> bool;
        fn std_srvs__srv__Trigger_Request__Sequence__fini(
            seq: *mut rosidl_runtime_rs::Sequence<Trigger_Request>,
        );
        fn std_srvs__srv__Trigger_Request__Sequence__copy(
            in_seq: &rosidl_runtime_rs::Sequence<Trigger_Request>,
            out_seq: *mut rosidl_runtime_rs::Sequence<Trigger_Request>,
        ) -> bool;

        fn std_srvs__srv__Trigger_Response__init(msg: *mut Trigger_Response) -> bool;
        fn std_srvs__srv__Trigger_Response__Sequence__init(
            seq: *mut rosidl_runtime_rs::Sequence<Trigger_Response>,
            size: usize,
        ) -> bool;
        fn std_srvs__srv__Trigger_Response__Sequence__fini(
            seq: *mut rosidl_runtime_rs::Sequence<Trigger_Response>,
        );
        fn std_srvs__srv__Trigger_Response__Sequence__copy(
            in_seq: &rosidl_runtime_rs::Sequence<Trigger_Response>,
            out_seq: *mut rosidl_runtime_rs::Sequence<Trigger_Response>,
        ) -> bool;
    }

    #[repr(C)]
    #[derive(Clone, Debug, PartialEq, PartialOrd)]
    pub struct SetBool_Request {
        pub data: bool,
    }

    impl Default for SetBool_Request {
        fn default() -> Self {
            unsafe {
                let mut msg = std::mem::zeroed();
                if !std_srvs__srv__SetBool_Request__init(&mut msg as *mut _) {
                    panic!("Call to std_srvs__srv__SetBool_Request__init() failed");
                }
                msg
            }
        }
    }

    impl rosidl_runtime_rs::SequenceAlloc for SetBool_Request {
        fn sequence_init(seq: &mut rosidl_runtime_rs::Sequence<Self>, size: usize) -> bool {
            unsafe { std_srvs__srv__SetBool_Request__Sequence__init(seq as *mut _, size) }
        }
        fn sequence_fini(seq: &mut rosidl_runtime_rs::Sequence<Self>) {
            unsafe { std_srvs__srv__SetBool_Request__Sequence__fini(seq as *mut _) }
        }
        fn sequence_copy(
            in_seq: &rosidl_runtime_rs::Sequence<Self>,
            out_seq: &mut rosidl_runtime_rs::Sequence<Self>,
        ) -> bool {
            unsafe { std_srvs__srv__SetBool_Request__Sequence__copy(in_seq, out_seq as *mut _) }
        }
    }

    impl rosidl_runtime_rs::Message for SetBool_Request {
        type RmwMsg = Self;
        fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
            msg_cow
        }
        fn from_rmw_message(msg: Self::RmwMsg) -> Self {
            msg
        }
    }

    impl rosidl_runtime_rs::RmwMessage for SetBool_Request {
        const TYPE_NAME: &'static str = "std_srvs/srv/SetBool_Request";
        fn get_type_support() -> *const std::ffi::c_void {
            unsafe {
                rosidl_typesupport_c__get_message_type_support_handle__std_srvs__srv__SetBool_Request()
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Debug, PartialEq, PartialOrd)]
    pub struct SetBool_Response {
        pub success: bool,
        pub message: rosidl_runtime_rs::String,
    }

    impl Default for SetBool_Response {
        fn default() -> Self {
            unsafe {
                let mut msg = std::mem::zeroed();
                if !std_srvs__srv__SetBool_Response__init(&mut msg as *mut _) {
                    panic!("Call to std_srvs__srv__SetBool_Response__init() failed");
                }
                msg
            }
        }
    }

    impl rosidl_runtime_rs::SequenceAlloc for SetBool_Response {
        fn sequence_init(seq: &mut rosidl_runtime_rs::Sequence<Self>, size: usize) -> bool {
            unsafe { std_srvs__srv__SetBool_Response__Sequence__init(seq as *mut _, size) }
        }
        fn sequence_fini(seq: &mut rosidl_runtime_rs::Sequence<Self>) {
            unsafe { std_srvs__srv__SetBool_Response__Sequence__fini(seq as *mut _) }
        }
        fn sequence_copy(
            in_seq: &rosidl_runtime_rs::Sequence<Self>,
            out_seq: &mut rosidl_runtime_rs::Sequence<Self>,
        ) -> bool {
            unsafe { std_srvs__srv__SetBool_Response__Sequence__copy(in_seq, out_seq as *mut _) }
        }
    }

    impl rosidl_runtime_rs::Message for SetBool_Response {
        type RmwMsg = Self;
        fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
            msg_cow
        }
        fn from_rmw_message(msg: Self::RmwMsg) -> Self {
            msg
        }
    }

    impl rosidl_runtime_rs::RmwMessage for SetBool_Response {
        const TYPE_NAME: &'static str = "std_srvs/srv/SetBool_Response";
        fn get_type_support() -> *const std::ffi::c_void {
            unsafe {
                rosidl_typesupport_c__get_message_type_support_handle__std_srvs__srv__SetBool_Response()
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Debug, PartialEq, PartialOrd)]
    pub struct Trigger_Request {
        pub structure_needs_at_least_one_member: u8,
    }

    impl Default for Trigger_Request {
        fn default() -> Self {
            unsafe {
                let mut msg = std::mem::zeroed();
                if !std_srvs__srv__Trigger_Request__init(&mut msg as *mut _) {
                    panic!("Call to std_srvs__srv__Trigger_Request__init() failed");
                }
                msg
            }
        }
    }

    impl rosidl_runtime_rs::SequenceAlloc for Trigger_Request {
        fn sequence_init(seq: &mut rosidl_runtime_rs::Sequence<Self>, size: usize) -> bool {
            unsafe { std_srvs__srv__Trigger_Request__Sequence__init(seq as *mut _, size) }
        }
        fn sequence_fini(seq: &mut rosidl_runtime_rs::Sequence<Self>) {
            unsafe { std_srvs__srv__Trigger_Request__Sequence__fini(seq as *mut _) }
        }
        fn sequence_copy(
            in_seq: &rosidl_runtime_rs::Sequence<Self>,
            out_seq: &mut rosidl_runtime_rs::Sequence<Self>,
        ) -> bool {
            unsafe { std_srvs__srv__Trigger_Request__Sequence__copy(in_seq, out_seq as *mut _) }
        }
    }

    impl rosidl_runtime_rs::Message for Trigger_Request {
        type RmwMsg = Self;
        fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
            msg_cow
        }
        fn from_rmw_message(msg: Self::RmwMsg) -> Self {
            msg
        }
    }

    impl rosidl_runtime_rs::RmwMessage for Trigger_Request {
        const TYPE_NAME: &'static str = "std_srvs/srv/Trigger_Request";
        fn get_type_support() -> *const std::ffi::c_void {
            unsafe {
                rosidl_typesupport_c__get_message_type_support_handle__std_srvs__srv__Trigger_Request()
            }
        }
    }

    #[repr(C)]
    #[derive(Clone, Debug, PartialEq, PartialOrd)]
    pub struct Trigger_Response {
        pub success: bool,
        pub message: rosidl_runtime_rs::String,
    }

    impl Default for Trigger_Response {
        fn default() -> Self {
            unsafe {
                let mut msg = std::mem::zeroed();
                if !std_srvs__srv__Trigger_Response__init(&mut msg as *mut _) {
                    panic!("Call to std_srvs__srv__Trigger_Response__init() failed");
                }
                msg
            }
        }
    }

    impl rosidl_runtime_rs::SequenceAlloc for Trigger_Response {
        fn sequence_init(seq: &mut rosidl_runtime_rs::Sequence<Self>, size: usize) -> bool {
            unsafe { std_srvs__srv__Trigger_Response__Sequence__init(seq as *mut _, size) }
        }
        fn sequence_fini(seq: &mut rosidl_runtime_rs::Sequence<Self>) {
            unsafe { std_srvs__srv__Trigger_Response__Sequence__fini(seq as *mut _) }
        }
        fn sequence_copy(
            in_seq: &rosidl_runtime_rs::Sequence<Self>,
            out_seq: &mut rosidl_runtime_rs::Sequence<Self>,
        ) -> bool {
            unsafe { std_srvs__srv__Trigger_Response__Sequence__copy(in_seq, out_seq as *mut _) }
        }
    }

    impl rosidl_runtime_rs::Message for Trigger_Response {
        type RmwMsg = Self;
        fn into_rmw_message(msg_cow: Cow<'_, Self>) -> Cow<'_, Self::RmwMsg> {
            msg_cow
        }
        fn from_rmw_message(msg: Self::RmwMsg) -> Self {
            msg
        }
    }

    impl rosidl_runtime_rs::RmwMessage for Trigger_Response {
        const TYPE_NAME: &'static str = "std_srvs/srv/Trigger_Response";
        fn get_type_support() -> *const std::ffi::c_void {
            unsafe {
                rosidl_typesupport_c__get_message_type_support_handle__std_srvs__srv__Trigger_Response()
            }
        }
    }

    pub struct SetBool;

    #[allow(dead_code)]
    impl rosidl_runtime_rs::Service for SetBool {
        type Request = SetBool_Request;
        type Response = SetBool_Response;

        fn get_type_support() -> *const std::ffi::c_void {
            unsafe {
                rosidl_typesupport_c__get_service_type_support_handle__std_srvs__srv__SetBool()
            }
        }
    }

    pub struct Trigger;

    #[allow(dead_code)]
    impl rosidl_runtime_rs::Service for Trigger {
        type Request = Trigger_Request;
        type Response = Trigger_Response;

        fn get_type_support() -> *const std::ffi::c_void {
            unsafe {
                rosidl_typesupport_c__get_service_type_support_handle__std_srvs__srv__Trigger()
            }
        }
    }
}
