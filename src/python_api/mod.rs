//! PyO3 bindings for the v1 Python SDK surface.

mod broker;
mod clients;
mod node;
mod pub_sub;
mod runtime;
mod util;

use pyo3::prelude::*;

use broker::{PyRobotBusBroker, run_broker};
use clients::{PyActionGoalHandle, PyNodeActionClient, PyNodeServiceClient};
use node::PyNode;
use pub_sub::{PyPublisher, PySubscriber, PyTopicPublisher};
use runtime::{
    PyActionServerHandle, PyCallbackGroup, PyCallbackGroupType, PyContext, PyMultiThreadedExecutor,
    PyServiceHandle, PyShutdownHandle, PySingleThreadedExecutor, PySubscriptionHandle,
    PyTimerHandle,
};
use util::{message_xpub_endpoint, message_xsub_endpoint, ros2_available};

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPublisher>()?;
    m.add_class::<PySubscriber>()?;
    m.add_class::<PyContext>()?;
    m.add_class::<PyCallbackGroupType>()?;
    m.add_class::<PyCallbackGroup>()?;
    m.add_class::<PySingleThreadedExecutor>()?;
    m.add_class::<PyMultiThreadedExecutor>()?;
    m.add_class::<PyNode>()?;
    m.add_class::<PyTopicPublisher>()?;
    m.add_class::<PyNodeServiceClient>()?;
    m.add_class::<PyNodeActionClient>()?;
    m.add_class::<PyActionGoalHandle>()?;
    m.add_class::<PyShutdownHandle>()?;
    m.add_class::<PyTimerHandle>()?;
    m.add_class::<PySubscriptionHandle>()?;
    m.add_class::<PyServiceHandle>()?;
    m.add_class::<PyActionServerHandle>()?;
    m.add_class::<PyRobotBusBroker>()?;
    m.add_function(wrap_pyfunction!(message_xsub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(message_xpub_endpoint, m)?)?;
    m.add_function(wrap_pyfunction!(run_broker, m)?)?;
    m.add_function(wrap_pyfunction!(ros2_available, m)?)?;
    // Ros2Bridge is implemented in pure Python (rclpy); see robot_bus.ros2_bridge.
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
