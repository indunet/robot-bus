//! Best-effort HTTP registration of topic → type with the broker console.

use serde_json::json;

const DEFAULT_CONSOLE_URL: &str = "http://127.0.0.1:15771";
const ENV_BROKER_URL: &str = "ROBOT_BUS_BROKER_URL";

/// Resolve console base URL: explicit option, then env, then localhost default.
pub fn resolve_console_url(explicit: Option<&str>) -> String {
    if let Some(u) = explicit {
        let t = u.trim();
        if !t.is_empty() {
            return trim_trailing_slash(t);
        }
    }
    if let Ok(u) = std::env::var(ENV_BROKER_URL) {
        let t = u.trim();
        if !t.is_empty() {
            return trim_trailing_slash(t);
        }
    }
    DEFAULT_CONSOLE_URL.to_string()
}

fn trim_trailing_slash(s: &str) -> String {
    s.trim_end_matches('/').to_string()
}

/// POST `/api/v1/topics/register`. Failures are logged only (never block publish).
pub fn register_topic_type(console_url: Option<&str>, topic: &str, type_name: &str) {
    let base = resolve_console_url(console_url);
    let url = format!("{base}/api/v1/topics/register");
    let body = json!({
        "topic": topic,
        "typeName": type_name,
    });
    match ureq::post(&url).send_json(body) {
        Ok(resp) => {
            let status = resp.status();
            if !(200..300).contains(&status) {
                log::warn!("topic type register {topic} -> {type_name}: HTTP {status} from {url}");
            }
        }
        Err(err) => {
            log::warn!("topic type register {topic} -> {type_name} failed ({url}): {err}");
        }
    }
}
