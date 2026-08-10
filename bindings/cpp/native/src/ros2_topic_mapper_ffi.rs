//! C ABI for C++-authored topic mappers (`RobotBusRos2TopicMapperVtable` + DynMsg field access).

use std::os::raw::{c_char, c_int};

use crate::ffi::err;

#[repr(C)]
pub struct RobotBusRos2DynMsg {
    _private: [u8; 0],
}

/// Convert ROS DynamicMessage → bus payload bytes.
///
/// On success: return 0 and set `*out_bus` / `*out_len` to a buffer allocated with
/// `robot_bus_alloc_bytes` (Rust takes ownership after the callback returns). On failure:
/// return non-zero (optionally call `robot_bus_set_error` first).
pub type RobotBusRos2TopicRosToBusFn = Option<
    unsafe extern "C" fn(
        ros_msg: *const RobotBusRos2DynMsg,
        out_bus: *mut *mut u8,
        out_len: *mut usize,
        user: *mut std::ffi::c_void,
    ) -> c_int,
>;

/// Convert bus payload → fill an empty ROS DynamicMessage of the mapper's type.
pub type RobotBusRos2TopicBusToRosFn = Option<
    unsafe extern "C" fn(
        bus_payload: *const u8,
        bus_len: usize,
        ros_msg: *mut RobotBusRos2DynMsg,
        user: *mut std::ffi::c_void,
    ) -> c_int,
>;

pub type RobotBusRos2TopicMapperDropFn =
    Option<unsafe extern "C" fn(user: *mut std::ffi::c_void)>;

#[repr(C)]
pub struct RobotBusRos2TopicMapperVtable {
    pub type_name: *const c_char,
    pub ros_to_bus: RobotBusRos2TopicRosToBusFn,
    pub bus_to_ros: RobotBusRos2TopicBusToRosFn,
    pub drop_user: RobotBusRos2TopicMapperDropFn,
    pub user: *mut std::ffi::c_void,
}

#[cfg(feature = "ros2")]
mod imp {
    use super::*;
    use std::ptr;
    use std::sync::Arc;

    use robot_bus::errors::BusError;
    use robot_bus::ros2_bridge::mapper_support::{
        self as support, nested_view, read_bool, read_byte_seq, read_f64, read_i64, read_string,
        with_nested_mut, write_bool, write_byte_seq, write_f64, write_i64, write_string,
    };
    use robot_bus::ros2_bridge::TopicMapper;
    use rclrs::{DynamicMessage, DynamicMessageView, DynamicMessageViewMut};

    use crate::ffi::{bytes_slice, bus_err, cstr_req, dup_bytes, dup_string, ok};
    use crate::ros2_bridge::{RobotBusRos2BridgeBuilder, imp as bridge_imp};

    struct FfiTopicMapper {
        type_name: String,
        ros_to_bus: unsafe extern "C" fn(
            *const RobotBusRos2DynMsg,
            *mut *mut u8,
            *mut usize,
            *mut std::ffi::c_void,
        ) -> c_int,
        bus_to_ros: unsafe extern "C" fn(
            *const u8,
            usize,
            *mut RobotBusRos2DynMsg,
            *mut std::ffi::c_void,
        ) -> c_int,
        drop_user: Option<unsafe extern "C" fn(*mut std::ffi::c_void)>,
        user: *mut std::ffi::c_void,
    }

    // Safety: caller guarantees `user` + callbacks are usable from bridge worker threads.
    unsafe impl Send for FfiTopicMapper {}
    unsafe impl Sync for FfiTopicMapper {}

    impl Drop for FfiTopicMapper {
        fn drop(&mut self) {
            if let Some(drop_fn) = self.drop_user {
                unsafe { drop_fn(self.user) };
            }
        }
    }

    impl TopicMapper for FfiTopicMapper {
        fn type_name(&self) -> &str {
            &self.type_name
        }

        fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>, BusError> {
            let mut out_ptr: *mut u8 = ptr::null_mut();
            let mut out_len: usize = 0;
            let rc = unsafe {
                (self.ros_to_bus)(
                    msg as *const DynamicMessage as *const RobotBusRos2DynMsg,
                    &mut out_ptr,
                    &mut out_len,
                    self.user,
                )
            };
            if rc != 0 {
                return Err(BusError::Protocol(format!(
                    "C++ TopicMapper::ros_to_bus failed for {}: {}",
                    self.type_name,
                    crate::ffi::last_error_message().unwrap_or_else(|| format!("code {rc}"))
                )));
            }
            if out_len == 0 {
                return Ok(Vec::new());
            }
            if out_ptr.is_null() {
                return Err(BusError::Protocol(format!(
                    "C++ TopicMapper::ros_to_bus for {} returned null buffer",
                    self.type_name
                )));
            }
            Ok(unsafe { Vec::from_raw_parts(out_ptr, out_len, out_len) })
        }

        fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage, BusError> {
            let mut msg = support::new_message(&self.type_name)?;
            let rc = unsafe {
                (self.bus_to_ros)(
                    payload.as_ptr(),
                    payload.len(),
                    &mut msg as *mut DynamicMessage as *mut RobotBusRos2DynMsg,
                    self.user,
                )
            };
            if rc != 0 {
                return Err(BusError::Protocol(format!(
                    "C++ TopicMapper::bus_to_ros failed for {}: {}",
                    self.type_name,
                    crate::ffi::last_error_message().unwrap_or_else(|| format!("code {rc}"))
                )));
            }
            Ok(msg)
        }
    }

    fn walk_read<R>(
        view: &DynamicMessageView<'_>,
        path: &str,
        f: &mut dyn FnMut(&DynamicMessageView<'_>, &str) -> Result<R, BusError>,
    ) -> Result<R, BusError> {
        if let Some((head, rest)) = path.split_once('.') {
            let nested = nested_view(view, head)?.ok_or_else(|| {
                BusError::Protocol(format!("missing nested field `{head}` on path `{path}`"))
            })?;
            walk_read(&nested, rest, f)
        } else {
            f(view, path)
        }
    }

    fn walk_write(
        view: &mut DynamicMessageViewMut<'_>,
        path: &str,
        f: &mut dyn FnMut(&mut DynamicMessageViewMut<'_>, &str) -> Result<(), BusError>,
    ) -> Result<(), BusError> {
        if let Some((head, rest)) = path.split_once('.') {
            with_nested_mut(view, head, |nested| walk_write(nested, rest, f))
        } else {
            f(view, path)
        }
    }

    fn as_msg<'a>(p: *const RobotBusRos2DynMsg) -> Result<&'a DynamicMessage, c_int> {
        if p.is_null() {
            return Err(err("null RobotBusRos2DynMsg"));
        }
        Ok(unsafe { &*(p as *const DynamicMessage) })
    }

    fn as_msg_mut<'a>(p: *mut RobotBusRos2DynMsg) -> Result<&'a mut DynamicMessage, c_int> {
        if p.is_null() {
            return Err(err("null RobotBusRos2DynMsg"));
        }
        Ok(unsafe { &mut *(p as *mut DynamicMessage) })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_route_mapper(
        b: *mut RobotBusRos2BridgeBuilder,
        ros_topic: *const c_char,
        bus_topic: *const c_char,
        mapper: *const RobotBusRos2TopicMapperVtable,
        direction: c_int,
    ) -> c_int {
        if b.is_null() {
            return err("null Ros2Bridge builder");
        }
        if mapper.is_null() {
            return err("null topic mapper vtable");
        }
        let ros_topic = match cstr_req(ros_topic) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let bus_topic = match cstr_req(bus_topic) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let direction = match bridge_imp::parse_direction(direction) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let vt = unsafe { &*mapper };
        let type_name = match cstr_req(vt.type_name) {
            Ok(t) => t.to_string(),
            Err(e) => return e,
        };
        let Some(ros_to_bus) = vt.ros_to_bus else {
            return err("topic mapper missing ros_to_bus");
        };
        let Some(bus_to_ros) = vt.bus_to_ros else {
            return err("topic mapper missing bus_to_ros");
        };
        let ffi = FfiTopicMapper {
            type_name,
            ros_to_bus,
            bus_to_ros,
            drop_user: vt.drop_user,
            user: vt.user,
        };
        let inner = unsafe { &mut *(b as *mut bridge_imp::BuilderInner) };
        let builder = match bridge_imp::take_builder(inner) {
            Ok(x) => x,
            Err(e) => return e,
        };
        match builder.add_route_mapper(
            ros_topic.to_string(),
            bus_topic.to_string(),
            Arc::new(ffi) as Arc<dyn TopicMapper>,
            direction,
        ) {
            Ok(next) => {
                inner.inner = Some(next);
                ok()
            }
            Err(e) => bus_err(e),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_has_field(
        msg: *const RobotBusRos2DynMsg,
        field_path: *const c_char,
    ) -> c_int {
        let msg = match as_msg(msg) {
            Ok(m) => m,
            Err(_) => return 0,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(_) => return 0,
        };
        let view = msg.view();
        let mut found = false;
        let _ = walk_read(&view, path, &mut |v, leaf| {
            found = support::has_field(v, leaf);
            Ok(())
        });
        if found { 1 } else { 0 }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_string(
        msg: *const RobotBusRos2DynMsg,
        field_path: *const c_char,
        out: *mut *mut c_char,
    ) -> c_int {
        if out.is_null() {
            return err("null out");
        }
        let msg = match as_msg(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let view = msg.view();
        let mut value = String::new();
        if let Err(e) = walk_read(&view, path, &mut |v, leaf| {
            value = read_string(v, leaf)?;
            Ok(())
        }) {
            return bus_err(e);
        }
        unsafe { *out = dup_string(&value) };
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_string(
        msg: *mut RobotBusRos2DynMsg,
        field_path: *const c_char,
        value: *const c_char,
    ) -> c_int {
        let msg = match as_msg_mut(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let value = match cstr_req(value) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let mut view = msg.view_mut();
        if let Err(e) = walk_write(&mut view, path, &mut |v, leaf| write_string(v, leaf, value)) {
            return bus_err(e);
        }
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_bool(
        msg: *const RobotBusRos2DynMsg,
        field_path: *const c_char,
        out: *mut c_int,
    ) -> c_int {
        if out.is_null() {
            return err("null out");
        }
        let msg = match as_msg(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let view = msg.view();
        let mut value = false;
        if let Err(e) = walk_read(&view, path, &mut |v, leaf| {
            value = read_bool(v, leaf)?;
            Ok(())
        }) {
            return bus_err(e);
        }
        unsafe { *out = if value { 1 } else { 0 } };
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_bool(
        msg: *mut RobotBusRos2DynMsg,
        field_path: *const c_char,
        value: c_int,
    ) -> c_int {
        let msg = match as_msg_mut(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let mut view = msg.view_mut();
        if let Err(e) = walk_write(&mut view, path, &mut |v, leaf| {
            write_bool(v, leaf, value != 0)
        }) {
            return bus_err(e);
        }
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_i64(
        msg: *const RobotBusRos2DynMsg,
        field_path: *const c_char,
        out: *mut i64,
    ) -> c_int {
        if out.is_null() {
            return err("null out");
        }
        let msg = match as_msg(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let view = msg.view();
        let mut value = 0i64;
        if let Err(e) = walk_read(&view, path, &mut |v, leaf| {
            value = read_i64(v, leaf)?;
            Ok(())
        }) {
            return bus_err(e);
        }
        unsafe { *out = value };
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_i64(
        msg: *mut RobotBusRos2DynMsg,
        field_path: *const c_char,
        value: i64,
    ) -> c_int {
        let msg = match as_msg_mut(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let mut view = msg.view_mut();
        if let Err(e) = walk_write(&mut view, path, &mut |v, leaf| write_i64(v, leaf, value)) {
            return bus_err(e);
        }
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_f64(
        msg: *const RobotBusRos2DynMsg,
        field_path: *const c_char,
        out: *mut f64,
    ) -> c_int {
        if out.is_null() {
            return err("null out");
        }
        let msg = match as_msg(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let view = msg.view();
        let mut value = 0.0f64;
        if let Err(e) = walk_read(&view, path, &mut |v, leaf| {
            value = read_f64(v, leaf)?;
            Ok(())
        }) {
            return bus_err(e);
        }
        unsafe { *out = value };
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_f64(
        msg: *mut RobotBusRos2DynMsg,
        field_path: *const c_char,
        value: f64,
    ) -> c_int {
        let msg = match as_msg_mut(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let mut view = msg.view_mut();
        if let Err(e) = walk_write(&mut view, path, &mut |v, leaf| write_f64(v, leaf, value)) {
            return bus_err(e);
        }
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_bytes(
        msg: *const RobotBusRos2DynMsg,
        field_path: *const c_char,
        out_data: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int {
        if out_data.is_null() || out_len.is_null() {
            return err("null out");
        }
        let msg = match as_msg(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let view = msg.view();
        let mut value = Vec::new();
        if let Err(e) = walk_read(&view, path, &mut |v, leaf| {
            value = read_byte_seq(v, leaf)?;
            Ok(())
        }) {
            return bus_err(e);
        }
        let (ptr, len) = dup_bytes(&value);
        unsafe {
            *out_data = ptr;
            *out_len = len;
        }
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_bytes(
        msg: *mut RobotBusRos2DynMsg,
        field_path: *const c_char,
        data: *const u8,
        len: usize,
    ) -> c_int {
        let msg = match as_msg_mut(msg) {
            Ok(m) => m,
            Err(e) => return e,
        };
        let path = match cstr_req(field_path) {
            Ok(p) => p,
            Err(e) => return e,
        };
        let bytes = match bytes_slice(data, len) {
            Ok(b) => b,
            Err(e) => return e,
        };
        let mut view = msg.view_mut();
        if let Err(e) = walk_write(&mut view, path, &mut |v, leaf| write_byte_seq(v, leaf, bytes))
        {
            return bus_err(e);
        }
        ok()
    }
}

#[cfg(not(feature = "ros2"))]
mod imp {
    use super::*;
    use crate::ros2_bridge::{RobotBusRos2BridgeBuilder, ROS2_UNAVAILABLE};

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_route_mapper(
        _b: *mut RobotBusRos2BridgeBuilder,
        _ros_topic: *const c_char,
        _bus_topic: *const c_char,
        _mapper: *const RobotBusRos2TopicMapperVtable,
        _direction: c_int,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_has_field(
        _msg: *const RobotBusRos2DynMsg,
        _field_path: *const c_char,
    ) -> c_int {
        0
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_string(
        _msg: *const RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _out: *mut *mut c_char,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_string(
        _msg: *mut RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _value: *const c_char,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_bool(
        _msg: *const RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _out: *mut c_int,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_bool(
        _msg: *mut RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _value: c_int,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_i64(
        _msg: *const RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _out: *mut i64,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_i64(
        _msg: *mut RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _value: i64,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_f64(
        _msg: *const RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _out: *mut f64,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_f64(
        _msg: *mut RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _value: f64,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_get_bytes(
        _msg: *const RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _out_data: *mut *mut u8,
        _out_len: *mut usize,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_dyn_msg_set_bytes(
        _msg: *mut RobotBusRos2DynMsg,
        _field_path: *const c_char,
        _data: *const u8,
        _len: usize,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }
}

#[allow(unused_imports)]
use imp::*;
