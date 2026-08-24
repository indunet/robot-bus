//! Node-side ledger of console metadata to restore after a broker restart.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::thread;
use std::time::Duration;

use crate::runtime::topic_type_register;
use crate::runtime::topology_register::TopologyEndpointGuard;

const KEEP_ALIVE: Duration = Duration::from_secs(10);

struct Endpoints {
    service_frontend: Option<String>,
    host: String,
    transport: String,
}

struct Inner {
    endpoints: Endpoints,
    topic_types: HashMap<String, String>,
    topology: Vec<Weak<TopologyEndpointGuard>>,
    keep_alive_started: bool,
}

/// Remembers topic-type and topology registrations for this node.
///
/// Re-pushes them when the session returns to Connected, and every 10s as a
/// fallback for nodes that do not observe HTTP liveness (explicit endpoints).
pub(crate) struct ControlPlaneLedger {
    inner: Mutex<Inner>,
    stop: AtomicBool,
}

impl ControlPlaneLedger {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                endpoints: Endpoints {
                    service_frontend: None,
                    host: "localhost".into(),
                    transport: "tcp".into(),
                },
                topic_types: HashMap::new(),
                topology: Vec::new(),
                keep_alive_started: false,
            }),
            stop: AtomicBool::new(false),
        })
    }

    pub(crate) fn update_endpoints(
        &self,
        service_frontend: Option<&str>,
        host: &str,
        transport: &str,
    ) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.endpoints.service_frontend = service_frontend
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            inner.endpoints.host = host.to_string();
            inner.endpoints.transport = transport.to_string();
        }
    }

    pub(crate) fn remember_topic_type(self: &Arc<Self>, topic: &str, type_name: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            inner
                .topic_types
                .insert(topic.to_string(), type_name.to_string());
        }
        self.ensure_keep_alive();
    }

    pub(crate) fn remember_topology(&self, guard: &Arc<TopologyEndpointGuard>) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.topology.retain(|w| w.strong_count() > 0);
            inner.topology.push(Arc::downgrade(guard));
        }
    }

    /// Re-register remembered metadata. Runs on a helper thread so the session
    /// loop is not blocked by control-plane RPCs.
    pub(crate) fn restore(self: &Arc<Self>) {
        let this = Arc::clone(self);
        thread::Builder::new()
            .name("rbus-meta-restore".into())
            .spawn(move || this.restore_now())
            .ok();
    }

    fn restore_now(&self) {
        let (endpoints, types, guards) = {
            let Ok(mut inner) = self.inner.lock() else {
                return;
            };
            inner.topology.retain(|w| w.strong_count() > 0);
            let guards: Vec<Arc<TopologyEndpointGuard>> =
                inner.topology.iter().filter_map(|w| w.upgrade()).collect();
            (
                Endpoints {
                    service_frontend: inner.endpoints.service_frontend.clone(),
                    host: inner.endpoints.host.clone(),
                    transport: inner.endpoints.transport.clone(),
                },
                inner.topic_types.clone(),
                guards,
            )
        };
        for (topic, type_name) in types {
            topic_type_register::register_topic_type(
                endpoints.service_frontend.as_deref(),
                &endpoints.host,
                &endpoints.transport,
                &topic,
                &type_name,
            );
        }
        for guard in guards {
            guard.refresh();
        }
    }

    fn ensure_keep_alive(self: &Arc<Self>) {
        {
            let Ok(mut inner) = self.inner.lock() else {
                return;
            };
            if inner.keep_alive_started {
                return;
            }
            inner.keep_alive_started = true;
        }
        let weak = Arc::downgrade(self);
        thread::Builder::new()
            .name("rbus-meta-refresh".into())
            .spawn(move || loop {
                thread::sleep(KEEP_ALIVE);
                let Some(this) = weak.upgrade() else {
                    break;
                };
                if this.stop.load(Ordering::Relaxed) {
                    break;
                }
                this.restore_now();
            })
            .ok();
    }
}

impl Drop for ControlPlaneLedger {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
