//! Best-effort readiness probes via broker console HTTP metrics.
//!
//! `service_is_ready` / `wait_for_action_server` poll `GET {console}/api/v1/services|actions`
//! and treat `workers > 0` as ready. This is not DDS graph discovery.

use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::discovery::normalize_api_base;

/// Default console base when [`crate::NodeOptions::console_url`] is unset.
pub const DEFAULT_CONSOLE_URL: &str = "http://127.0.0.1:15570";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadyKind {
    Service,
    Action,
}

impl ReadyKind {
    fn path(self) -> &'static str {
        match self {
            Self::Service => "/api/v1/services",
            Self::Action => "/api/v1/actions",
        }
    }

    fn list_key(self) -> &'static str {
        match self {
            Self::Service => "services",
            Self::Action => "actions",
        }
    }
}

#[derive(Debug, Deserialize)]
struct NamedWorkers {
    name: String,
    #[serde(default)]
    workers: u64,
}

#[derive(Debug, Deserialize)]
struct ServicesEnvelope {
    #[serde(default)]
    services: Vec<NamedWorkers>,
}

#[derive(Debug, Deserialize)]
struct ActionsEnvelope {
    #[serde(default)]
    actions: Vec<NamedWorkers>,
}

/// Resolve console base URL (trim trailing `/`).
pub fn resolve_console_url(console_url: Option<&str>) -> String {
    let raw = console_url
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_CONSOLE_URL);
    normalize_api_base(raw)
}

fn names_match(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let a = a.strip_prefix('/').unwrap_or(a);
    let b = b.strip_prefix('/').unwrap_or(b);
    a == b
}

fn fetch_workers(console_url: &str, kind: ReadyKind, name: &str) -> Option<u64> {
    let base = resolve_console_url(Some(console_url));
    let url = format!("{}{}", base.trim_end_matches('/'), kind.path());
    let body = ureq::get(&url).call().ok()?.into_string().ok()?;
    let entries = match kind {
        ReadyKind::Service => {
            let env: ServicesEnvelope = serde_json::from_str(&body).ok()?;
            env.services
        }
        ReadyKind::Action => {
            let env: ActionsEnvelope = serde_json::from_str(&body).ok()?;
            env.actions
        }
    };
    let _ = kind.list_key();
    entries
        .into_iter()
        .find(|e| names_match(&e.name, name))
        .map(|e| e.workers)
}

/// True when console reports `workers > 0` for `name`.
pub fn is_ready(console_url: Option<&str>, kind: ReadyKind, name: &str) -> bool {
    let url = resolve_console_url(console_url);
    fetch_workers(&url, kind, name).unwrap_or(0) > 0
}

/// Poll until ready or `timeout` elapses. `None` waits indefinitely.
pub fn wait_until_ready(
    console_url: Option<&str>,
    kind: ReadyKind,
    name: &str,
    timeout: Option<Duration>,
) -> bool {
    let url = resolve_console_url(console_url);
    let deadline = timeout.map(|d| Instant::now() + d);
    let poll = Duration::from_millis(50);
    loop {
        if is_ready(Some(&url), kind, name) {
            return true;
        }
        if let Some(deadline) = deadline {
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            let remaining = deadline.saturating_duration_since(now);
            thread::sleep(poll.min(remaining));
        } else {
            thread::sleep(poll);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_falls_back_to_default() {
        assert_eq!(
            resolve_console_url(None),
            normalize_api_base(DEFAULT_CONSOLE_URL)
        );
        assert_eq!(
            resolve_console_url(Some("  ")),
            normalize_api_base(DEFAULT_CONSOLE_URL)
        );
    }

    #[test]
    fn names_match_strips_slash() {
        assert!(names_match("/echo", "echo"));
        assert!(names_match("echo", "/echo"));
        assert!(!names_match("a", "b"));
    }
}
