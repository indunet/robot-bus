//! C ABI for in-process ROS 2 ↔ robot-bus topic/service/action bridge.
//!
//! When built without `--features ros2`, symbols still export but return errors
//! pointing at `robot-bus-cpp-ros2-<distro>` / rebuild with the feature.

use std::os::raw::{c_char, c_int};
use std::ptr;

use crate::{err, set_error};

#[cfg(feature = "ros2")]
use crate::{bus_err, clear_error, cstr_opt, cstr_req, ok};

#[cfg_attr(feature = "ros2", allow(dead_code))]
pub(crate) const ROS2_UNAVAILABLE: &str = "ROS 2 bridge unavailable: install robot-bus-cpp-ros2-humble \
or robot-bus-cpp-ros2-jazzy (matching your ROS distro), or rebuild robot_bus_c with \
--features ros2 after sourcing ROS 2";

#[repr(C)]
pub struct RobotBusRos2BridgeBuilder {
    _private: [u8; 0],
}

#[repr(C)]
pub struct RobotBusRos2Bridge {
    _private: [u8; 0],
}

#[path = "ros2_topic_mapper_ffi.rs"]
mod topic_mapper_ffi;

#[cfg(feature = "ros2")]
pub(crate) mod imp {
    use super::*;
    use std::time::Duration;

    use robot_bus::ros2_bridge::{Direction, Ros2Bridge, Ros2BridgeBuilder};

    pub(crate) struct BuilderInner {
        pub inner: Option<Ros2BridgeBuilder>,
    }

    pub struct BridgeInner {
        pub bridge: Ros2Bridge,
    }

    pub(crate) fn parse_direction(d: c_int) -> Result<Direction, c_int> {
        match d {
            0 => Ok(Direction::Ros2ToBus),
            1 => Ok(Direction::BusToRos2),
            other => Err(err(format!(
                "invalid ros2 direction {other}; use ROBOT_BUS_ROUTE_DIR_ROS2_TO_BUS (0) or BUS_TO_ROS2 (1)"
            ))),
        }
    }

    pub(crate) fn take_builder(b: &mut BuilderInner) -> Result<Ros2BridgeBuilder, c_int> {
        b.inner
            .take()
            .ok_or_else(|| err("Ros2Bridge builder already consumed"))
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_available() -> c_int {
        1
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_from_yaml(path: *const c_char) -> *mut RobotBusRos2Bridge {
        let path = match cstr_req(path) {
            Ok(p) => p,
            Err(_) => return ptr::null_mut(),
        };
        match Ros2Bridge::from_yaml(path) {
            Ok(bridge) => {
                clear_error();
                Box::into_raw(Box::new(BridgeInner { bridge })) as *mut RobotBusRos2Bridge
            }
            Err(e) => {
                set_error(e.to_string());
                ptr::null_mut()
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_new(
        name: *const c_char,
    ) -> *mut RobotBusRos2BridgeBuilder {
        let name = match cstr_req(name) {
            Ok(n) => n,
            Err(_) => return ptr::null_mut(),
        };
        clear_error();
        Box::into_raw(Box::new(BuilderInner {
            inner: Some(Ros2Bridge::new(name)),
        })) as *mut RobotBusRos2BridgeBuilder
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_free(b: *mut RobotBusRos2BridgeBuilder) {
        if !b.is_null() {
            unsafe {
                drop(Box::from_raw(b as *mut BuilderInner));
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_bus_tcp(
        b: *mut RobotBusRos2BridgeBuilder,
        host: *const c_char,
    ) -> c_int {
        if b.is_null() {
            return err("null Ros2Bridge builder");
        }
        let host = match cstr_req(host) {
            Ok(h) => h,
            Err(e) => return e,
        };
        let inner = unsafe { &mut *(b as *mut BuilderInner) };
        let builder = match take_builder(inner) {
            Ok(x) => x,
            Err(e) => return e,
        };
        inner.inner = Some(builder.bus_tcp(host));
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_bus_ipc(
        b: *mut RobotBusRos2BridgeBuilder,
    ) -> c_int {
        if b.is_null() {
            return err("null Ros2Bridge builder");
        }
        let inner = unsafe { &mut *(b as *mut BuilderInner) };
        let builder = match take_builder(inner) {
            Ok(x) => x,
            Err(e) => return e,
        };
        inner.inner = Some(builder.bus_ipc());
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_bus_ipc_at(
        b: *mut RobotBusRos2BridgeBuilder,
        dir: *const c_char,
    ) -> c_int {
        if b.is_null() {
            return err("null Ros2Bridge builder");
        }
        let dir = match cstr_req(dir) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let inner = unsafe { &mut *(b as *mut BuilderInner) };
        let builder = match take_builder(inner) {
            Ok(x) => x,
            Err(e) => return e,
        };
        inner.inner = Some(builder.bus_ipc_at(dir));
        ok()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_bus_discover(
        b: *mut RobotBusRos2BridgeBuilder,
        api_url: *const c_char,
        timeout_secs: f64,
        broker_id: *const c_char,
    ) -> c_int {
        if b.is_null() {
            return err("null Ros2Bridge builder");
        }
        let timeout = if timeout_secs > 0.0 {
            Some(timeout_secs)
        } else {
            None
        };
        let api_url = cstr_opt(api_url)
            .map(|s| s.to_string())
            .unwrap_or_else(|| robot_bus::DiscoverOpts::default().api_url);
        let broker_id = cstr_opt(broker_id).map(|s| s.to_string());
        let inner = unsafe { &mut *(b as *mut BuilderInner) };
        let builder = match take_builder(inner) {
            Ok(x) => x,
            Err(e) => return e,
        };
        match builder.bus_discover_ex(api_url, timeout, broker_id) {
            Ok(next) => {
                inner.inner = Some(next);
                ok()
            }
            Err(e) => {
                // Builder was taken; put a fresh empty one is wrong — leave None.
                bus_err(e)
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_route(
        b: *mut RobotBusRos2BridgeBuilder,
        ros_topic: *const c_char,
        bus_topic: *const c_char,
        type_name: *const c_char,
        direction: c_int,
    ) -> c_int {
        if b.is_null() {
            return err("null Ros2Bridge builder");
        }
        let ros_topic = match cstr_req(ros_topic) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let bus_topic = match cstr_req(bus_topic) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let type_name = match cstr_req(type_name) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let direction = match parse_direction(direction) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let inner = unsafe { &mut *(b as *mut BuilderInner) };
        let builder = match take_builder(inner) {
            Ok(x) => x,
            Err(e) => return e,
        };
        match builder.add_route(
            ros_topic.to_string(),
            bus_topic.to_string(),
            type_name.to_string(),
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
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_service(
        b: *mut RobotBusRos2BridgeBuilder,
        ros_service: *const c_char,
        bus_service: *const c_char,
        type_name: *const c_char,
        direction: c_int,
    ) -> c_int {
        robot_bus_ros2_bridge_builder_add_service_ex(
            b,
            ros_service,
            bus_service,
            type_name,
            direction,
            0.0,
        )
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_service_ex(
        b: *mut RobotBusRos2BridgeBuilder,
        ros_service: *const c_char,
        bus_service: *const c_char,
        type_name: *const c_char,
        direction: c_int,
        timeout_secs: f64,
    ) -> c_int {
        if b.is_null() {
            return err("null Ros2Bridge builder");
        }
        let ros_service = match cstr_req(ros_service) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let bus_service = match cstr_req(bus_service) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let type_name = match cstr_req(type_name) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let direction = match parse_direction(direction) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let timeout = if timeout_secs > 0.0 {
            Duration::from_secs_f64(timeout_secs)
        } else {
            robot_bus::ros2_bridge::SERVICE_CALL_TIMEOUT
        };
        let inner = unsafe { &mut *(b as *mut BuilderInner) };
        let builder = match take_builder(inner) {
            Ok(x) => x,
            Err(e) => return e,
        };
        match builder.add_service_with_timeout(
            ros_service.to_string(),
            bus_service.to_string(),
            type_name.to_string(),
            direction,
            timeout,
        ) {
            Ok(next) => {
                inner.inner = Some(next);
                ok()
            }
            Err(e) => bus_err(e),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_action(
        b: *mut RobotBusRos2BridgeBuilder,
        ros_action: *const c_char,
        bus_action: *const c_char,
        type_name: *const c_char,
        direction: c_int,
    ) -> c_int {
        robot_bus_ros2_bridge_builder_add_action_ex(
            b,
            ros_action,
            bus_action,
            type_name,
            direction,
            0.0,
        )
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_action_ex(
        b: *mut RobotBusRos2BridgeBuilder,
        ros_action: *const c_char,
        bus_action: *const c_char,
        type_name: *const c_char,
        direction: c_int,
        timeout_secs: f64,
    ) -> c_int {
        if b.is_null() {
            return err("null Ros2Bridge builder");
        }
        let ros_action = match cstr_req(ros_action) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let bus_action = match cstr_req(bus_action) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let type_name = match cstr_req(type_name) {
            Ok(t) => t,
            Err(e) => return e,
        };
        let direction = match parse_direction(direction) {
            Ok(d) => d,
            Err(e) => return e,
        };
        let timeout = if timeout_secs > 0.0 {
            Duration::from_secs_f64(timeout_secs)
        } else {
            robot_bus::ros2_bridge::ACTION_CALL_TIMEOUT
        };
        let inner = unsafe { &mut *(b as *mut BuilderInner) };
        let builder = match take_builder(inner) {
            Ok(x) => x,
            Err(e) => return e,
        };
        match builder.add_action_with_timeout(
            ros_action.to_string(),
            bus_action.to_string(),
            type_name.to_string(),
            direction,
            timeout,
        ) {
            Ok(next) => {
                inner.inner = Some(next);
                ok()
            }
            Err(e) => bus_err(e),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_build(
        b: *mut RobotBusRos2BridgeBuilder,
    ) -> *mut RobotBusRos2Bridge {
        if b.is_null() {
            set_error("null Ros2Bridge builder");
            return ptr::null_mut();
        }
        let inner = unsafe { &mut *(b as *mut BuilderInner) };
        let builder = match take_builder(inner) {
            Ok(x) => x,
            Err(_) => return ptr::null_mut(),
        };
        match builder.build() {
            Ok(bridge) => {
                clear_error();
                // Builder handle remains for caller to free (inner is None).
                Box::into_raw(Box::new(BridgeInner { bridge })) as *mut RobotBusRos2Bridge
            }
            Err(e) => {
                set_error(e.to_string());
                ptr::null_mut()
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_free(bridge: *mut RobotBusRos2Bridge) {
        if !bridge.is_null() {
            unsafe {
                drop(Box::from_raw(bridge as *mut BridgeInner));
            }
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_spin(bridge: *mut RobotBusRos2Bridge) -> c_int {
        if bridge.is_null() {
            return err("null Ros2Bridge");
        }
        let inner = unsafe { &mut *(bridge as *mut BridgeInner) };
        match inner.bridge.spin() {
            Ok(()) => ok(),
            Err(e) => bus_err(e),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_spin_once(
        bridge: *mut RobotBusRos2Bridge,
        timeout_secs: f64,
    ) -> c_int {
        if bridge.is_null() {
            return err("null Ros2Bridge");
        }
        let timeout = if timeout_secs < 0.0 {
            Duration::from_millis(10)
        } else {
            Duration::from_secs_f64(timeout_secs)
        };
        let inner = unsafe { &mut *(bridge as *mut BridgeInner) };
        match inner.bridge.spin_once(timeout) {
            Ok(()) => ok(),
            Err(e) => bus_err(e),
        }
    }
}

#[cfg(not(feature = "ros2"))]
mod imp {
    use super::*;

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_available() -> c_int {
        0
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_from_yaml(_path: *const c_char) -> *mut RobotBusRos2Bridge {
        set_error(ROS2_UNAVAILABLE);
        ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_new(
        _name: *const c_char,
    ) -> *mut RobotBusRos2BridgeBuilder {
        set_error(ROS2_UNAVAILABLE);
        ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_free(_b: *mut RobotBusRos2BridgeBuilder) {}

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_bus_tcp(
        _b: *mut RobotBusRos2BridgeBuilder,
        _host: *const c_char,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_bus_ipc(
        _b: *mut RobotBusRos2BridgeBuilder,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_bus_ipc_at(
        _b: *mut RobotBusRos2BridgeBuilder,
        _dir: *const c_char,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_bus_discover(
        _b: *mut RobotBusRos2BridgeBuilder,
        _api_url: *const c_char,
        _timeout_secs: f64,
        _broker_id: *const c_char,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_route(
        _b: *mut RobotBusRos2BridgeBuilder,
        _ros_topic: *const c_char,
        _bus_topic: *const c_char,
        _type_name: *const c_char,
        _direction: c_int,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_service(
        _b: *mut RobotBusRos2BridgeBuilder,
        _ros_service: *const c_char,
        _bus_service: *const c_char,
        _type_name: *const c_char,
        _direction: c_int,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_service_ex(
        _b: *mut RobotBusRos2BridgeBuilder,
        _ros_service: *const c_char,
        _bus_service: *const c_char,
        _type_name: *const c_char,
        _direction: c_int,
        _timeout_secs: f64,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_action(
        _b: *mut RobotBusRos2BridgeBuilder,
        _ros_action: *const c_char,
        _bus_action: *const c_char,
        _type_name: *const c_char,
        _direction: c_int,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_add_action_ex(
        _b: *mut RobotBusRos2BridgeBuilder,
        _ros_action: *const c_char,
        _bus_action: *const c_char,
        _type_name: *const c_char,
        _direction: c_int,
        _timeout_secs: f64,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_builder_build(
        _b: *mut RobotBusRos2BridgeBuilder,
    ) -> *mut RobotBusRos2Bridge {
        set_error(ROS2_UNAVAILABLE);
        ptr::null_mut()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_free(_bridge: *mut RobotBusRos2Bridge) {}

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_spin(_bridge: *mut RobotBusRos2Bridge) -> c_int {
        err(ROS2_UNAVAILABLE)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn robot_bus_ros2_bridge_spin_once(
        _bridge: *mut RobotBusRos2Bridge,
        _timeout_secs: f64,
    ) -> c_int {
        err(ROS2_UNAVAILABLE)
    }
}
