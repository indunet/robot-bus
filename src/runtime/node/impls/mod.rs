use std::sync::Arc;

use crate::runtime::topology_register::TopologyEndpointGuard;

use super::Node;

mod actions;
mod services;
mod topics;

impl Node {
    fn start_topology_guard(&self, kind: &str, name: &str) -> Arc<TopologyEndpointGuard> {
        let guard = TopologyEndpointGuard::start(
            self.options.service_frontend.as_deref(),
            &self.options.host,
            &self.options.transport,
            &self.name,
            kind,
            name,
        );
        self.control_plane.remember_topology(&guard);
        guard
    }
}
