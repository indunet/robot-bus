//! PyO3 bindings for [`crate::ros2_bridge`] (`feature = "ros2"` + `extension-module`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyType};
use rclrs::{DynamicMessage, DynamicMessageView, DynamicMessageViewMut};

use crate::errors::BusError;
use crate::ros2_bridge::mapper_support::{
    self as support, nested_view, read_bool, read_byte_seq, read_f64, read_i64, read_string,
    with_nested_mut, write_bool, write_byte_seq, write_f64, write_i64, write_string,
};
use crate::ros2_bridge::{
    Direction, Ros2Bridge, Ros2BridgeBuilder, TopicMapper, lookup_action_mapper,
    lookup_service_mapper, lookup_topic_mapper_arc,
};

fn bus_err(err: BusError) -> PyErr {
    PyRuntimeError::new_err(err.to_string())
}

fn parse_direction(value: &Bound<'_, PyAny>) -> PyResult<Direction> {
    if let Ok(d) = value.extract::<PyDirection>() {
        return Ok(d.inner());
    }
    if let Ok(s) = value.extract::<String>() {
        return match s.as_str() {
            "ros2_to_bus" | "Ros2ToBus" => Ok(Direction::Ros2ToBus),
            "bus_to_ros2" | "BusToRos2" => Ok(Direction::BusToRos2),
            other => Err(PyRuntimeError::new_err(format!(
                "direction must be Ros2ToBus/BusToRos2, got {other:?}"
            ))),
        };
    }
    if let Ok(i) = value.extract::<i32>() {
        return match i {
            0 => Ok(Direction::Ros2ToBus),
            1 => Ok(Direction::BusToRos2),
            other => Err(PyRuntimeError::new_err(format!(
                "direction must be 0 or 1, got {other}"
            ))),
        };
    }
    Err(PyRuntimeError::new_err(
        "direction must be Direction, str, or int",
    ))
}

#[pyclass(name = "Direction", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyDirection {
    #[pyo3(name = "Ros2ToBus")]
    Ros2ToBus = 0,
    #[pyo3(name = "BusToRos2")]
    BusToRos2 = 1,
}

impl PyDirection {
    fn inner(self) -> Direction {
        match self {
            Self::Ros2ToBus => Direction::Ros2ToBus,
            Self::BusToRos2 => Direction::BusToRos2,
        }
    }
}

/// Borrowed DynamicMessage during a Python TopicMapper callback.
#[pyclass(name = "DynMsg", unsendable)]
struct PyDynMsg {
    ptr: usize,
    writable: bool,
    alive: Arc<AtomicBool>,
}

impl PyDynMsg {
    fn view<'a>(&self) -> PyResult<&'a DynamicMessage> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(PyRuntimeError::new_err(
                "DynMsg is only valid during the mapper callback",
            ));
        }
        Ok(unsafe { &*(self.ptr as *const DynamicMessage) })
    }

    fn view_mut<'a>(&self) -> PyResult<&'a mut DynamicMessage> {
        if !self.writable {
            return Err(PyRuntimeError::new_err("DynMsg is read-only in ros_to_bus"));
        }
        if !self.alive.load(Ordering::Acquire) {
            return Err(PyRuntimeError::new_err(
                "DynMsg is only valid during the mapper callback",
            ));
        }
        Ok(unsafe { &mut *(self.ptr as *mut DynamicMessage) })
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

#[pymethods]
impl PyDynMsg {
    fn has_field(&self, path: &str) -> PyResult<bool> {
        let msg = self.view()?;
        let view = msg.view();
        let mut found = false;
        let _ = walk_read(&view, path, &mut |v, leaf| {
            found = support::has_field(v, leaf);
            Ok(())
        });
        Ok(found)
    }

    fn get_string(&self, path: &str) -> PyResult<String> {
        let msg = self.view()?;
        let view = msg.view();
        let mut value = String::new();
        walk_read(&view, path, &mut |v, leaf| {
            value = read_string(v, leaf)?;
            Ok(())
        })
        .map_err(bus_err)?;
        Ok(value)
    }

    fn set_string(&self, path: &str, value: &str) -> PyResult<()> {
        let msg = self.view_mut()?;
        let mut view = msg.view_mut();
        walk_write(&mut view, path, &mut |v, leaf| write_string(v, leaf, value)).map_err(bus_err)
    }

    fn get_bool(&self, path: &str) -> PyResult<bool> {
        let msg = self.view()?;
        let view = msg.view();
        let mut value = false;
        walk_read(&view, path, &mut |v, leaf| {
            value = read_bool(v, leaf)?;
            Ok(())
        })
        .map_err(bus_err)?;
        Ok(value)
    }

    fn set_bool(&self, path: &str, value: bool) -> PyResult<()> {
        let msg = self.view_mut()?;
        let mut view = msg.view_mut();
        walk_write(&mut view, path, &mut |v, leaf| write_bool(v, leaf, value)).map_err(bus_err)
    }

    fn get_i64(&self, path: &str) -> PyResult<i64> {
        let msg = self.view()?;
        let view = msg.view();
        let mut value = 0i64;
        walk_read(&view, path, &mut |v, leaf| {
            value = read_i64(v, leaf)?;
            Ok(())
        })
        .map_err(bus_err)?;
        Ok(value)
    }

    fn set_i64(&self, path: &str, value: i64) -> PyResult<()> {
        let msg = self.view_mut()?;
        let mut view = msg.view_mut();
        walk_write(&mut view, path, &mut |v, leaf| write_i64(v, leaf, value)).map_err(bus_err)
    }

    fn get_f64(&self, path: &str) -> PyResult<f64> {
        let msg = self.view()?;
        let view = msg.view();
        let mut value = 0.0f64;
        walk_read(&view, path, &mut |v, leaf| {
            value = read_f64(v, leaf)?;
            Ok(())
        })
        .map_err(bus_err)?;
        Ok(value)
    }

    fn set_f64(&self, path: &str, value: f64) -> PyResult<()> {
        let msg = self.view_mut()?;
        let mut view = msg.view_mut();
        walk_write(&mut view, path, &mut |v, leaf| write_f64(v, leaf, value)).map_err(bus_err)
    }

    fn get_bytes<'py>(&self, py: Python<'py>, path: &str) -> PyResult<Bound<'py, PyBytes>> {
        let msg = self.view()?;
        let view = msg.view();
        let mut value = Vec::new();
        walk_read(&view, path, &mut |v, leaf| {
            value = read_byte_seq(v, leaf)?;
            Ok(())
        })
        .map_err(bus_err)?;
        Ok(PyBytes::new(py, &value))
    }

    fn set_bytes(&self, path: &str, value: &[u8]) -> PyResult<()> {
        let msg = self.view_mut()?;
        let mut view = msg.view_mut();
        walk_write(&mut view, path, &mut |v, leaf| write_byte_seq(v, leaf, value)).map_err(bus_err)
    }
}

struct PyCallbackTopicMapper {
    type_name: String,
    obj: Py<PyAny>,
}

impl TopicMapper for PyCallbackTopicMapper {
    fn type_name(&self) -> &str {
        &self.type_name
    }

    fn ros_to_bus(&self, msg: &DynamicMessage) -> Result<Vec<u8>, BusError> {
        Python::with_gil(|py| {
            let alive = Arc::new(AtomicBool::new(true));
            let dyn_msg = PyDynMsg {
                ptr: msg as *const DynamicMessage as usize,
                writable: false,
                alive: Arc::clone(&alive),
            };
            let result = (|| {
                let bound = Bound::new(py, dyn_msg)?;
                let out = self.obj.bind(py).call_method1("ros_to_bus", (bound,))?;
                let bytes: Vec<u8> = out.extract()?;
                Ok::<_, PyErr>(bytes)
            })();
            alive.store(false, Ordering::Release);
            result.map_err(|e| BusError::Protocol(format!("Python TopicMapper.ros_to_bus: {e}")))
        })
    }

    fn bus_to_ros(&self, payload: &[u8]) -> Result<DynamicMessage, BusError> {
        Python::with_gil(|py| {
            let mut msg = support::new_message(&self.type_name)?;
            let alive = Arc::new(AtomicBool::new(true));
            let dyn_msg = PyDynMsg {
                ptr: &mut msg as *mut DynamicMessage as usize,
                writable: true,
                alive: Arc::clone(&alive),
            };
            let result = (|| {
                let bound = Bound::new(py, dyn_msg)?;
                let payload_obj = PyBytes::new(py, payload);
                self.obj
                    .bind(py)
                    .call_method1("bus_to_ros", (payload_obj, bound))?;
                Ok::<_, PyErr>(())
            })();
            alive.store(false, Ordering::Release);
            result.map_err(|e| BusError::Protocol(format!("Python TopicMapper.bus_to_ros: {e}")))?;
            Ok(msg)
        })
    }
}

fn topic_mapper_from_py(py: Python<'_>, arg: &Bound<'_, PyAny>) -> PyResult<Arc<dyn TopicMapper>> {
    let _ = py;
    if let Ok(name) = arg.extract::<String>() {
        return lookup_topic_mapper_arc(&name).map_err(bus_err);
    }
    if !arg.hasattr("type_name")? || !arg.hasattr("ros_to_bus")? || !arg.hasattr("bus_to_ros")? {
        return Err(PyRuntimeError::new_err(
            "topic mapper must be a type name str or an object with type_name/ros_to_bus/bus_to_ros",
        ));
    }
    let type_name: String = arg.call_method0("type_name")?.extract()?;
    Ok(Arc::new(PyCallbackTopicMapper {
        type_name,
        obj: arg.clone().unbind(),
    }))
}

/// Resolve a service codec: type-name str, builtin tag str, or object with `type_name()`.
///
/// Custom convert methods (`request_ros_to_bus`, …) are accepted for API shape parity with
/// topics, but until Track B only builtin typed backends are wired — unknown type names error.
fn service_mapper_from_py(py: Python<'_>, arg: &Bound<'_, PyAny>) -> PyResult<String> {
    let _ = py;
    if let Ok(name) = arg.extract::<String>() {
        let _ = lookup_service_mapper(&name).map_err(bus_err)?;
        return Ok(name);
    }
    if !arg.hasattr("type_name")? {
        return Err(PyRuntimeError::new_err(
            "service mapper must be a type name str or an object with type_name()",
        ));
    }
    let type_name: String = arg.call_method0("type_name")?.extract()?;
    let _ = lookup_service_mapper(&type_name).map_err(|e| {
        PyRuntimeError::new_err(format!(
            "{e}; custom ServiceMapper convert methods need Track B dynamic service support"
        ))
    })?;
    Ok(type_name)
}

fn action_mapper_from_py(py: Python<'_>, arg: &Bound<'_, PyAny>) -> PyResult<String> {
    let _ = py;
    if let Ok(name) = arg.extract::<String>() {
        let _ = lookup_action_mapper(&name).map_err(bus_err)?;
        return Ok(name);
    }
    if !arg.hasattr("type_name")? {
        return Err(PyRuntimeError::new_err(
            "action mapper must be a type name str or an object with type_name()",
        ));
    }
    let type_name: String = arg.call_method0("type_name")?.extract()?;
    let _ = lookup_action_mapper(&type_name).map_err(|e| {
        PyRuntimeError::new_err(format!(
            "{e}; custom ActionMapper convert methods need Track B dynamic action support"
        ))
    })?;
    Ok(type_name)
}

#[pyclass(name = "Ros2BridgeBuilder", unsendable)]
struct PyRos2BridgeBuilder {
    inner: Option<Ros2BridgeBuilder>,
}

impl PyRos2BridgeBuilder {
    fn take(&mut self) -> PyResult<Ros2BridgeBuilder> {
        self.inner
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("Ros2BridgeBuilder already consumed"))
    }

    fn from_inner(inner: Ros2BridgeBuilder) -> Self {
        Self {
            inner: Some(inner),
        }
    }
}

#[pymethods]
impl PyRos2BridgeBuilder {
    fn bus_tcp(slf: Py<Self>, py: Python<'_>, host: &str) -> PyResult<Py<Self>> {
        {
            let mut this = slf.borrow_mut(py);
            let b = this.take()?;
            this.inner = Some(b.bus_tcp(host));
        }
        Ok(slf)
    }

    fn bus_ipc(slf: Py<Self>, py: Python<'_>) -> PyResult<Py<Self>> {
        {
            let mut this = slf.borrow_mut(py);
            let b = this.take()?;
            this.inner = Some(b.bus_ipc());
        }
        Ok(slf)
    }

    fn bus_ipc_at(slf: Py<Self>, py: Python<'_>, dir: &str) -> PyResult<Py<Self>> {
        {
            let mut this = slf.borrow_mut(py);
            let b = this.take()?;
            this.inner = Some(b.bus_ipc_at(dir));
        }
        Ok(slf)
    }

    #[pyo3(signature = (api_url = "http://127.0.0.1:15570", timeout_secs = None, broker_id = None))]
    fn bus_discover(
        slf: Py<Self>,
        py: Python<'_>,
        api_url: &str,
        timeout_secs: Option<f64>,
        broker_id: Option<String>,
    ) -> PyResult<Py<Self>> {
        {
            let mut this = slf.borrow_mut(py);
            let b = this.take()?;
            this.inner = Some(
                b.bus_discover_ex(api_url, timeout_secs, broker_id)
                    .map_err(bus_err)?,
            );
        }
        Ok(slf)
    }

    fn route(
        slf: Py<Self>,
        py: Python<'_>,
        ros_topic: String,
        bus_topic: String,
    ) -> PyResult<PyRouteBuilder> {
        let mut this = slf.borrow_mut(py);
        let parent = this.take()?;
        Ok(PyRouteBuilder {
            parent: Some(parent),
            ros_topic,
            bus_topic,
            mapper: None,
            direction: Direction::Ros2ToBus,
        })
    }

    fn service(
        slf: Py<Self>,
        py: Python<'_>,
        ros_service: String,
        bus_service: String,
    ) -> PyResult<PyServiceBuilder> {
        let mut this = slf.borrow_mut(py);
        let parent = this.take()?;
        Ok(PyServiceBuilder {
            parent: Some(parent),
            ros_service,
            bus_service,
            type_name: None,
            direction: Direction::Ros2ToBus,
            timeout: None,
        })
    }

    fn action(
        slf: Py<Self>,
        py: Python<'_>,
        ros_action: String,
        bus_action: String,
    ) -> PyResult<PyActionBuilder> {
        let mut this = slf.borrow_mut(py);
        let parent = this.take()?;
        Ok(PyActionBuilder {
            parent: Some(parent),
            ros_action,
            bus_action,
            type_name: None,
            direction: Direction::Ros2ToBus,
            timeout: None,
        })
    }

    #[pyo3(signature = (ros_topic, bus_topic, mapper, direction = None))]
    fn add_route(
        slf: Py<Self>,
        py: Python<'_>,
        ros_topic: String,
        bus_topic: String,
        mapper: &Bound<'_, PyAny>,
        direction: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<Self>> {
        let direction = match direction {
            Some(d) => parse_direction(d)?,
            None => Direction::Ros2ToBus,
        };
        let mapper = topic_mapper_from_py(py, mapper)?;
        {
            let mut this = slf.borrow_mut(py);
            let b = this.take()?;
            this.inner = Some(
                b.add_route_mapper(ros_topic, bus_topic, mapper, direction)
                    .map_err(bus_err)?,
            );
        }
        Ok(slf)
    }

    #[pyo3(signature = (ros_service, bus_service, type_name, direction = None))]
    fn add_service(
        slf: Py<Self>,
        py: Python<'_>,
        ros_service: String,
        bus_service: String,
        type_name: &str,
        direction: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<Self>> {
        let direction = match direction {
            Some(d) => parse_direction(d)?,
            None => Direction::Ros2ToBus,
        };
        {
            let mut this = slf.borrow_mut(py);
            let b = this.take()?;
            this.inner = Some(
                b.add_service(ros_service, bus_service, type_name, direction)
                    .map_err(bus_err)?,
            );
        }
        Ok(slf)
    }

    #[pyo3(signature = (ros_action, bus_action, type_name, direction = None))]
    fn add_action(
        slf: Py<Self>,
        py: Python<'_>,
        ros_action: String,
        bus_action: String,
        type_name: &str,
        direction: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<Self>> {
        let direction = match direction {
            Some(d) => parse_direction(d)?,
            None => Direction::Ros2ToBus,
        };
        {
            let mut this = slf.borrow_mut(py);
            let b = this.take()?;
            this.inner = Some(
                b.add_action(ros_action, bus_action, type_name, direction)
                    .map_err(bus_err)?,
            );
        }
        Ok(slf)
    }

    fn build(slf: Py<Self>, py: Python<'_>) -> PyResult<PyRos2Bridge> {
        let mut this = slf.borrow_mut(py);
        let b = this.take()?;
        let bridge = b.build().map_err(bus_err)?;
        Ok(PyRos2Bridge {
            inner: Some(bridge),
        })
    }
}

#[pyclass(name = "Ros2BridgeRoute", unsendable)]
struct PyRouteBuilder {
    parent: Option<Ros2BridgeBuilder>,
    ros_topic: String,
    bus_topic: String,
    mapper: Option<Py<PyAny>>,
    direction: Direction,
}

#[pymethods]
impl PyRouteBuilder {
    fn mapper(slf: Py<Self>, py: Python<'_>, mapper: Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        slf.borrow_mut(py).mapper = Some(mapper.unbind());
        Ok(slf)
    }

    fn direction(slf: Py<Self>, py: Python<'_>, direction: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        slf.borrow_mut(py).direction = parse_direction(direction)?;
        Ok(slf)
    }

    fn add(slf: Py<Self>, py: Python<'_>) -> PyResult<PyRos2BridgeBuilder> {
        let mut this = slf.borrow_mut(py);
        let parent = this
            .parent
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("route builder already consumed"))?;
        let mapper_obj = this.mapper.take().ok_or_else(|| {
            PyRuntimeError::new_err("ros2 bridge route: call .mapper(...) before .add()")
        })?;
        let mapper = topic_mapper_from_py(py, mapper_obj.bind(py))?;
        let next = parent
            .add_route_mapper(
                this.ros_topic.clone(),
                this.bus_topic.clone(),
                mapper,
                this.direction,
            )
            .map_err(bus_err)?;
        Ok(PyRos2BridgeBuilder::from_inner(next))
    }
}

#[pyclass(name = "Ros2BridgeService", unsendable)]
struct PyServiceBuilder {
    parent: Option<Ros2BridgeBuilder>,
    ros_service: String,
    bus_service: String,
    type_name: Option<String>,
    direction: Direction,
    timeout: Option<Duration>,
}

#[pymethods]
impl PyServiceBuilder {
    fn mapper(slf: Py<Self>, py: Python<'_>, mapper: Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let type_name = service_mapper_from_py(py, &mapper)?;
        slf.borrow_mut(py).type_name = Some(type_name);
        Ok(slf)
    }

    fn type_name(slf: Py<Self>, py: Python<'_>, type_name: &str) -> PyResult<Py<Self>> {
        let _ = lookup_service_mapper(type_name).map_err(bus_err)?;
        slf.borrow_mut(py).type_name = Some(type_name.to_string());
        Ok(slf)
    }

    fn direction(slf: Py<Self>, py: Python<'_>, direction: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        slf.borrow_mut(py).direction = parse_direction(direction)?;
        Ok(slf)
    }

    fn timeout(slf: Py<Self>, py: Python<'_>, secs: f64) -> PyResult<Py<Self>> {
        if secs <= 0.0 {
            return Err(PyRuntimeError::new_err("timeout must be > 0 seconds"));
        }
        slf.borrow_mut(py).timeout = Some(Duration::from_secs_f64(secs));
        Ok(slf)
    }

    fn add(slf: Py<Self>, py: Python<'_>) -> PyResult<PyRos2BridgeBuilder> {
        let mut this = slf.borrow_mut(py);
        let parent = this
            .parent
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("service builder already consumed"))?;
        let type_name = this.type_name.take().ok_or_else(|| {
            PyRuntimeError::new_err("ros2 bridge service: call .mapper(...) before .add()")
        })?;
        let timeout = this
            .timeout
            .unwrap_or(crate::ros2_bridge::SERVICE_CALL_TIMEOUT);
        let next = parent
            .add_service_with_timeout(
                this.ros_service.clone(),
                this.bus_service.clone(),
                type_name,
                this.direction,
                timeout,
            )
            .map_err(bus_err)?;
        Ok(PyRos2BridgeBuilder::from_inner(next))
    }
}

#[pyclass(name = "Ros2BridgeAction", unsendable)]
struct PyActionBuilder {
    parent: Option<Ros2BridgeBuilder>,
    ros_action: String,
    bus_action: String,
    type_name: Option<String>,
    direction: Direction,
    timeout: Option<Duration>,
}

#[pymethods]
impl PyActionBuilder {
    fn mapper(slf: Py<Self>, py: Python<'_>, mapper: Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let type_name = action_mapper_from_py(py, &mapper)?;
        slf.borrow_mut(py).type_name = Some(type_name);
        Ok(slf)
    }

    fn type_name(slf: Py<Self>, py: Python<'_>, type_name: &str) -> PyResult<Py<Self>> {
        let _ = lookup_action_mapper(type_name).map_err(bus_err)?;
        slf.borrow_mut(py).type_name = Some(type_name.to_string());
        Ok(slf)
    }

    fn direction(slf: Py<Self>, py: Python<'_>, direction: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        slf.borrow_mut(py).direction = parse_direction(direction)?;
        Ok(slf)
    }

    fn timeout(slf: Py<Self>, py: Python<'_>, secs: f64) -> PyResult<Py<Self>> {
        if secs <= 0.0 {
            return Err(PyRuntimeError::new_err("timeout must be > 0 seconds"));
        }
        slf.borrow_mut(py).timeout = Some(Duration::from_secs_f64(secs));
        Ok(slf)
    }

    fn add(slf: Py<Self>, py: Python<'_>) -> PyResult<PyRos2BridgeBuilder> {
        let mut this = slf.borrow_mut(py);
        let parent = this
            .parent
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("action builder already consumed"))?;
        let type_name = this.type_name.take().ok_or_else(|| {
            PyRuntimeError::new_err("ros2 bridge action: call .mapper(...) before .add()")
        })?;
        let timeout = this
            .timeout
            .unwrap_or(crate::ros2_bridge::ACTION_CALL_TIMEOUT);
        let next = parent
            .add_action_with_timeout(
                this.ros_action.clone(),
                this.bus_action.clone(),
                type_name,
                this.direction,
                timeout,
            )
            .map_err(bus_err)?;
        Ok(PyRos2BridgeBuilder::from_inner(next))
    }
}

#[pyclass(name = "Ros2Bridge", unsendable)]
struct PyRos2Bridge {
    inner: Option<Ros2Bridge>,
}

#[pymethods]
impl PyRos2Bridge {
    #[classmethod]
    fn new(_cls: &Bound<'_, PyType>, name: String) -> PyRos2BridgeBuilder {
        PyRos2BridgeBuilder::from_inner(Ros2Bridge::new(name))
    }

    #[classmethod]
    fn from_yaml(_cls: &Bound<'_, PyType>, path: &str) -> PyResult<Self> {
        let bridge = Ros2Bridge::from_yaml(path).map_err(bus_err)?;
        Ok(Self {
            inner: Some(bridge),
        })
    }

    fn spin(&mut self, py: Python<'_>) -> PyResult<()> {
        let bridge = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Ros2Bridge already closed"))?;
        loop {
            if let Err(err) = py.check_signals() {
                if err.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py) {
                    return Ok(());
                }
                return Err(err);
            }
            let ptr = bridge as *mut Ros2Bridge as usize;
            py.allow_threads(|| {
                let b = unsafe { &mut *(ptr as *mut Ros2Bridge) };
                b.spin_once(Duration::from_millis(10))
            })
            .map_err(bus_err)?;
        }
    }

    #[pyo3(signature = (timeout_secs = 0.01))]
    fn spin_once(&mut self, py: Python<'_>, timeout_secs: f64) -> PyResult<()> {
        let bridge = self
            .inner
            .as_mut()
            .ok_or_else(|| PyRuntimeError::new_err("Ros2Bridge already closed"))?;
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let ptr = bridge as *mut Ros2Bridge as usize;
        py.allow_threads(|| {
            let b = unsafe { &mut *(ptr as *mut Ros2Bridge) };
            b.spin_once(timeout)
        })
        .map_err(bus_err)
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDirection>()?;
    m.add_class::<PyDynMsg>()?;
    m.add_class::<PyRos2BridgeBuilder>()?;
    m.add_class::<PyRouteBuilder>()?;
    m.add_class::<PyServiceBuilder>()?;
    m.add_class::<PyActionBuilder>()?;
    m.add_class::<PyRos2Bridge>()?;
    m.add("StdMsgsStringMapper", "std_msgs/msg/String")?;
    m.add("SensorMsgsImageMapper", "sensor_msgs/msg/Image")?;
    m.add("SensorMsgsImuMapper", "sensor_msgs/msg/Imu")?;
    m.add("TriggerServiceMapper", "std_srvs/srv/Trigger")?;
    m.add("SetBoolServiceMapper", "std_srvs/srv/SetBool")?;
    m.add(
        "FibonacciActionMapper",
        "example_interfaces/action/Fibonacci",
    )?;
    Ok(())
}
