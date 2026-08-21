//! Node ↔ broker session: connection state, HTTP discover retry, liveness.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::discovery::DiscoverOpts;
use crate::runtime::node::NodeOptions;

/// Backoff after a failed discover / WS connect attempt.
pub const SESSION_BACKOFF_INITIAL: Duration = Duration::from_millis(200);
/// Cap for discover / WS reconnect backoff.
pub const SESSION_BACKOFF_MAX: Duration = Duration::from_secs(5);
/// How often a Connected node re-probes `GET /api/v1/discover`.
pub const SESSION_LIVENESS_INTERVAL: Duration = Duration::from_secs(3);
/// Timeout for a single discover HTTP call from the session thread.
pub const SESSION_DISCOVER_TIMEOUT: Duration = Duration::from_millis(800);
/// How long `create_*` waits for the session to become Connected.
pub const SESSION_CREATE_WAIT: Duration = Duration::from_secs(3);

/// Broker link state for a [`crate::Node`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Created,
    Discovering,
    Connecting,
    Connected,
    Reconnecting,
    Shutdown,
}

impl ConnectionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Discovering => "discovering",
            Self::Connecting => "connecting",
            Self::Connected => "connected",
            Self::Reconnecting => "reconnecting",
            Self::Shutdown => "shutdown",
        }
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub type ConnectionEventCallback =
    Arc<dyn Fn(ConnectionState, ConnectionState, &str) + Send + Sync>;

type ReconnectHook = Arc<dyn Fn() + Send + Sync>;

struct SessionInner {
    state: ConnectionState,
    options: NodeOptions,
    events: Vec<ConnectionEventCallback>,
    reconnect_hook: Option<ReconnectHook>,
    discover_fail_logged: bool,
}

struct SessionShared {
    inner: Mutex<SessionInner>,
    cond: Condvar,
    stop: AtomicBool,
}

/// Background broker session owned by a [`crate::Node`].
pub struct BrokerSession {
    shared: Arc<SessionShared>,
    thread: Option<JoinHandle<()>>,
}

impl BrokerSession {
    pub fn start(options: NodeOptions) -> Self {
        let shared = Arc::new(SessionShared {
            inner: Mutex::new(SessionInner {
                state: ConnectionState::Created,
                options,
                events: Vec::new(),
                reconnect_hook: None,
                discover_fail_logged: false,
            }),
            cond: Condvar::new(),
            stop: AtomicBool::new(false),
        });
        let thread_shared = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name("rbus-session".into())
            .spawn(move || session_loop(thread_shared))
            .ok();
        Self {
            shared,
            thread,
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.shared
            .inner
            .lock()
            .map(|g| g.state)
            .unwrap_or(ConnectionState::Shutdown)
    }

    pub fn options(&self) -> NodeOptions {
        self.shared
            .inner
            .lock()
            .map(|g| g.options.clone())
            .unwrap_or_else(|_| NodeOptions::tcp())
    }

    pub fn add_on_connection_event(&self, cb: ConnectionEventCallback) {
        if let Ok(mut g) = self.shared.inner.lock() {
            g.events.push(cb);
        }
    }

    pub fn set_reconnect_hook(&self, hook: ReconnectHook) {
        if let Ok(mut g) = self.shared.inner.lock() {
            g.reconnect_hook = Some(hook);
        }
    }

    /// Block until [`ConnectionState::Connected`], `Shutdown`, or `timeout`.
    ///
    /// `None` waits until Connected or Shutdown. Returns `false` on timeout or
    /// Shutdown without ever reaching Connected.
    pub fn wait_for_broker(&self, timeout: Option<Duration>) -> bool {
        let deadline = timeout.map(|d| std::time::Instant::now() + d);
        let mut guard = match self.shared.inner.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };
        loop {
            if guard.state == ConnectionState::Connected {
                return true;
            }
            if guard.state == ConnectionState::Shutdown || self.shared.stop.load(Ordering::Acquire)
            {
                return false;
            }
            if let Some(deadline) = deadline {
                let now = std::time::Instant::now();
                if now >= deadline {
                    return false;
                }
                let remaining = deadline.saturating_duration_since(now);
                let (g, result) = match self.shared.cond.wait_timeout(guard, remaining) {
                    Ok(pair) => pair,
                    Err(_) => return false,
                };
                guard = g;
                if result.timed_out() && guard.state != ConnectionState::Connected {
                    return false;
                }
            } else {
                guard = match self.shared.cond.wait(guard) {
                    Ok(g) => g,
                    Err(_) => return false,
                };
            }
        }
    }

    pub fn shutdown(&mut self) {
        set_state(&self.shared, ConnectionState::Shutdown, "shutdown");
        self.shared.stop.store(true, Ordering::Release);
        self.shared.cond.notify_all();
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for BrokerSession {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn requires_http_liveness(opts: &NodeOptions) -> bool {
    opts.is_ws() || opts.needs_endpoint_discover()
}

fn discover_api_url(opts: &NodeOptions) -> String {
    if let Some(url) = opts.console_url.as_deref().filter(|s| !s.is_empty()) {
        return url.to_string();
    }
    if let Some(url) = opts.ws_url.as_deref().filter(|s| !s.is_empty()) {
        return url.to_string();
    }
    DiscoverOpts::for_host(&opts.host).api_url
}

fn set_state(shared: &SessionShared, next: ConnectionState, reason: &str) {
    let (prev, events, hook) = {
        let Ok(mut guard) = shared.inner.lock() else {
            return;
        };
        if guard.state == ConnectionState::Shutdown && next != ConnectionState::Shutdown {
            return;
        }
        if guard.state == next {
            return;
        }
        let prev = guard.state;
        guard.state = next;
        let events = guard.events.clone();
        let hook = if prev == ConnectionState::Reconnecting && next == ConnectionState::Connected {
            guard.reconnect_hook.clone()
        } else {
            None
        };
        (prev, events, hook)
    };
    shared.cond.notify_all();
    for cb in events {
        cb(prev, next, reason);
    }
    if let Some(hook) = hook {
        hook();
    }
}

fn try_discover(shared: &SessionShared) -> Result<NodeOptions, String> {
    let options = shared
        .inner
        .lock()
        .map_err(|_| "session mutex poisoned".to_string())?
        .options
        .clone();
    let api = discover_api_url(&options);
    let mut opts = DiscoverOpts::at(&api);
    opts.timeout = SESSION_DISCOVER_TIMEOUT;
    match options.clone().discover(opts) {
        Ok(filled) => Ok(filled),
        Err(err) => Err(err.to_string()),
    }
}

fn apply_discovered(shared: &SessionShared, filled: NodeOptions) {
    if let Ok(mut guard) = shared.inner.lock() {
        guard.options = filled;
        guard.discover_fail_logged = false;
    }
}

fn wait_interruptible(shared: &SessionShared, dur: Duration) -> bool {
    if shared.stop.load(Ordering::Acquire) {
        return true;
    }
    let Ok(guard) = shared.inner.lock() else {
        return true;
    };
    if shared.stop.load(Ordering::Acquire) || guard.state == ConnectionState::Shutdown {
        return true;
    }
    let _ = shared.cond.wait_timeout(guard, dur);
    shared.stop.load(Ordering::Acquire)
}

fn session_loop(shared: Arc<SessionShared>) {
    let mut backoff = SESSION_BACKOFF_INITIAL;
    {
        let requires_http = shared
            .inner
            .lock()
            .map(|g| requires_http_liveness(&g.options))
            .unwrap_or(true);
        if requires_http {
            set_state(&shared, ConnectionState::Discovering, "start discover");
        } else {
            set_state(&shared, ConnectionState::Connecting, "endpoints known");
            set_state(&shared, ConnectionState::Connected, "local endpoints");
        }
    }

    while !shared.stop.load(Ordering::Acquire) {
        let state = shared
            .inner
            .lock()
            .map(|g| g.state)
            .unwrap_or(ConnectionState::Shutdown);
        if state == ConnectionState::Shutdown {
            break;
        }

        let requires_http = shared
            .inner
            .lock()
            .map(|g| requires_http_liveness(&g.options))
            .unwrap_or(true);

        match state {
            ConnectionState::Created | ConnectionState::Discovering | ConnectionState::Connecting
            | ConnectionState::Reconnecting => {
                if !requires_http {
                    set_state(&shared, ConnectionState::Connected, "local endpoints");
                    backoff = SESSION_BACKOFF_INITIAL;
                    continue;
                }
                if state == ConnectionState::Discovering || state == ConnectionState::Created {
                    set_state(&shared, ConnectionState::Discovering, "discover");
                } else if state == ConnectionState::Reconnecting {
                    set_state(&shared, ConnectionState::Connecting, "retry");
                }
                match try_discover(&shared) {
                    Ok(filled) => {
                        apply_discovered(&shared, filled);
                        set_state(&shared, ConnectionState::Connected, "discover ok");
                        backoff = SESSION_BACKOFF_INITIAL;
                    }
                    Err(err) => {
                        log_discover_fail(&shared, &err);
                        if state == ConnectionState::Reconnecting {
                            set_state(&shared, ConnectionState::Reconnecting, "discover failed");
                        } else {
                            set_state(&shared, ConnectionState::Discovering, "discover failed");
                        }
                        if wait_interruptible(&shared, backoff) {
                            break;
                        }
                        backoff = std::cmp::min(backoff.saturating_mul(2), SESSION_BACKOFF_MAX);
                    }
                }
            }
            ConnectionState::Connected => {
                if wait_interruptible(&shared, SESSION_LIVENESS_INTERVAL) {
                    break;
                }
                if !requires_http {
                    continue;
                }
                match try_discover(&shared) {
                    Ok(filled) => {
                        apply_discovered(&shared, filled);
                    }
                    Err(err) => {
                        log::debug!("broker liveness lost: {err}");
                        set_state(&shared, ConnectionState::Reconnecting, "liveness lost");
                        backoff = SESSION_BACKOFF_INITIAL;
                    }
                }
            }
            ConnectionState::Shutdown => break,
        }
    }
    set_state(&shared, ConnectionState::Shutdown, "session stopped");
}

fn log_discover_fail(shared: &SessionShared, err: &str) {
    let first = shared
        .inner
        .lock()
        .map(|mut g| {
            let first = !g.discover_fail_logged;
            g.discover_fail_logged = true;
            first
        })
        .unwrap_or(false);
    if first {
        log::warn!("auto-discover failed for node (start broker / check --api-listen): {err}");
    } else {
        log::debug!("auto-discover retry failed: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_state_display() {
        assert_eq!(ConnectionState::Connected.as_str(), "connected");
        assert_eq!(ConnectionState::Reconnecting.to_string(), "reconnecting");
    }

    #[test]
    fn wait_for_broker_times_out_without_broker() {
        let mut opts = NodeOptions::tcp();
        opts.console_url = Some("http://127.0.0.1:1".into());
        let mut session = BrokerSession::start(opts);
        assert!(!session.wait_for_broker(Some(Duration::from_millis(200))));
        let state = session.state();
        assert!(
            matches!(
                state,
                ConnectionState::Created
                    | ConnectionState::Discovering
                    | ConnectionState::Connecting
                    | ConnectionState::Reconnecting
            ),
            "unexpected state {state}"
        );
        session.shutdown();
        assert_eq!(session.state(), ConnectionState::Shutdown);
    }

    #[test]
    fn inproc_session_is_connected_without_http() {
        let session = BrokerSession::start(NodeOptions::inproc());
        assert!(session.wait_for_broker(Some(Duration::from_secs(1))));
        assert_eq!(session.state(), ConnectionState::Connected);
    }
}
