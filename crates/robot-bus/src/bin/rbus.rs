//! `rbus` — discover a broker and inspect its monitoring data over HTTP.
//!
//! Default API URL is `http://127.0.0.1:15560` (override with `--url` or
//! `ROBOT_BUS_BROKER_URL`).

use std::io;
use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use robot_bus::discovery::normalize_api_base;
use robot_bus::{DEFAULT_API_DISCOVER_PATH, DiscoverResponse};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

const DEFAULT_URL: &str = "http://127.0.0.1:15560";
const ENV_BROKER_URL: &str = "ROBOT_BUS_BROKER_URL";
const DEFAULT_TIMEOUT_MS: u64 = 3000;
const TOPICS_PATH: &str = "/api/v1/topics";

#[derive(Parser, Debug)]
#[command(
    name = "rbus",
    version,
    about = "Robot Bus discovery and monitoring CLI",
    long_about = None
)]
struct Cli {
    /// Broker API base URL (default: ROBOT_BUS_BROKER_URL or http://127.0.0.1:15560)
    #[arg(long, global = true)]
    url: Option<String>,

    /// Print the broker's JSON response
    #[arg(long, global = true)]
    json: bool,

    /// HTTP request timeout in milliseconds
    #[arg(long, global = true, default_value_t = DEFAULT_TIMEOUT_MS, value_parser = clap::value_parser!(u64).range(1..))]
    timeout_ms: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show the broker's advertised TCP endpoints and federation capabilities
    Discover,
    /// Topic introspection
    Topic {
        #[command(subcommand)]
        command: TopicCmd,
    },
    /// Service introspection
    Service {
        #[command(subcommand)]
        command: ServiceCmd,
    },
    /// Action introspection
    Action {
        #[command(subcommand)]
        command: ActionCmd,
    },
    /// Broker / console status summary
    Status,
}

#[derive(Subcommand, Debug)]
enum TopicCmd {
    /// List topics (registered types and/or recent traffic)
    List,
    /// Show details for one topic (type + metrics)
    Info {
        /// Topic name (e.g. `/robot1/imu`)
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum ServiceCmd {
    /// List known services (after worker READY)
    List,
}

#[derive(Subcommand, Debug)]
enum ActionCmd {
    /// List known actions (after worker READY)
    List,
}

#[derive(Deserialize)]
struct TopicList {
    topics: Vec<TopicRow>,
}

#[derive(Deserialize)]
struct ServiceList {
    services: Vec<Named>,
}

#[derive(Deserialize)]
struct ActionList {
    actions: Vec<Named>,
}

#[derive(Deserialize)]
struct Named {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TopicRow {
    name: String,
    #[serde(default)]
    type_name: Option<String>,
    #[serde(default)]
    msg_per_sec: f64,
    #[serde(default)]
    bytes_per_sec: u64,
    #[serde(default)]
    last_seen: u64,
    #[serde(default)]
    total_msgs: u64,
    #[serde(default)]
    subscribers: u64,
    #[serde(default)]
    publishers: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse {
    status: String,
    version: String,
    uptime: u64,
    pid: u32,
    grpc_addr: String,
    web_addr: String,
    msg_bus_x_sub: String,
    msg_bus_x_pub: String,
    #[serde(rename = "svcFE")]
    svc_fe: String,
    #[serde(rename = "svcBE")]
    svc_be: String,
    #[serde(rename = "actFE")]
    act_fe: String,
    #[serde(rename = "actBE")]
    act_be: String,
    msg_per_sec: f64,
    bytes_per_sec: u64,
    svc_calls_per_sec: f64,
    act_runs_per_sec: f64,
    total_messages: u64,
    total_errors: u64,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let base = resolve_base_url(cli.url.as_deref());
    let timeout = Duration::from_millis(cli.timeout_ms);
    let api = BrokerApi {
        base: &base,
        agent: ureq::AgentBuilder::new()
            .timeout(timeout)
            .timeout_connect(timeout)
            .build(),
        json: cli.json,
    };

    match cli.command {
        Commands::Discover => api.discover(),
        Commands::Topic {
            command: TopicCmd::List,
        } => api.show(TOPICS_PATH, |body: TopicList| {
            for t in body.topics {
                let ty = t.type_name.as_deref().unwrap_or("-");
                println!("{}\t{}", t.name, ty);
            }
        }),
        Commands::Topic {
            command: TopicCmd::Info { name },
        } => {
            let path = format!("{TOPICS_PATH}/{}", encode_topic_path(&name));
            api.show(&path, |t: TopicRow| {
                println!("Topic: {}", t.name);
                println!("Type: {}", t.type_name.as_deref().unwrap_or("-"));
                println!("Publisher count: {}", t.publishers);
                println!("Subscription count: {}", t.subscribers);
                println!("Msg/s: {}", t.msg_per_sec);
                println!("Bytes/s: {}", t.bytes_per_sec);
                println!("Total msgs: {}", t.total_msgs);
                println!("Last seen: {}", t.last_seen);
            })
        }
        Commands::Service {
            command: ServiceCmd::List,
        } => api.show("/api/v1/services", |body: ServiceList| {
            for s in body.services {
                println!("{}", s.name);
            }
        }),
        Commands::Action {
            command: ActionCmd::List,
        } => api.show("/api/v1/actions", |body: ActionList| {
            for a in body.actions {
                println!("{}", a.name);
            }
        }),
        Commands::Status => api.show("/api/v1/status", |s: StatusResponse| {
            println!("status: {}", s.status);
            println!("version: {}", s.version);
            println!("uptime_secs: {}", s.uptime);
            println!("pid: {}", s.pid);
            println!("ws: {}", s.grpc_addr);
            println!("console: {}", s.web_addr);
            println!("msg_xsub: {}", s.msg_bus_x_sub);
            println!("msg_xpub: {}", s.msg_bus_x_pub);
            println!("svc_fe: {}", s.svc_fe);
            println!("svc_be: {}", s.svc_be);
            println!("act_fe: {}", s.act_fe);
            println!("act_be: {}", s.act_be);
            println!("msg_per_sec: {}", s.msg_per_sec);
            println!("bytes_per_sec: {}", s.bytes_per_sec);
            println!("svc_calls_per_sec: {}", s.svc_calls_per_sec);
            println!("act_runs_per_sec: {}", s.act_runs_per_sec);
            println!("total_messages: {}", s.total_messages);
            println!("total_errors: {}", s.total_errors);
        }),
    }
}

fn resolve_base_url(cli_url: Option<&str>) -> String {
    if let Some(u) = cli_url {
        return normalize_api_base(u);
    }
    if let Ok(u) = std::env::var(ENV_BROKER_URL) {
        if !u.trim().is_empty() {
            return normalize_api_base(&u);
        }
    }
    DEFAULT_URL.to_string()
}

/// Percent-encode a topic path so `/` survives as a single Axum `{*name}` segment.
fn encode_topic_path(topic: &str) -> String {
    topic
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

struct BrokerApi<'a> {
    base: &'a str,
    agent: ureq::Agent,
    json: bool,
}

impl BrokerApi<'_> {
    fn discover(&self) -> Result<()> {
        let (discovery, body) = self.get::<DiscoverResponse>(DEFAULT_API_DISCOVER_PATH)?;
        let announcement = discovery.to_announcement(self.base).with_context(|| {
            format!(
                "invalid discovery response from {}{DEFAULT_API_DISCOVER_PATH}",
                self.base
            )
        })?;
        if self.json {
            println!("{body}");
            return Ok(());
        }

        println!("Broker ID: {}", discovery.broker_id);
        println!("Domain ID: {}", discovery.domain_id);
        let host = announcement.advertise_host;
        println!("TCP host: {host}");
        for (label, port) in [
            ("Topic XSUB", discovery.message_xsub_port),
            ("Topic XPUB", discovery.message_xpub_port),
            ("Service frontend", discovery.service_frontend_port),
            ("Service backend", discovery.service_backend_port),
            ("Action frontend", discovery.action_frontend_port),
            ("Action backend", discovery.action_backend_port),
        ] {
            println!("{label}: tcp://{host}:{port}");
        }
        if discovery.message_fed_port != 0 {
            println!(
                "Topic federation: tcp://{host}:{}",
                discovery.message_fed_port
            );
        }
        if !discovery.federation.is_empty() {
            println!(
                "Federation capabilities: {}",
                discovery.federation.join(", ")
            );
        }
        Ok(())
    }

    fn show<T: DeserializeOwned>(&self, path: &str, print_text: impl FnOnce(T)) -> Result<()> {
        let (value, body) = self.get(path)?;
        if self.json {
            println!("{body}");
        } else {
            print_text(value);
        }
        Ok(())
    }

    fn get<T: DeserializeOwned>(&self, path: &str) -> Result<(T, Value)> {
        let url = format!("{}{path}", self.base);
        let response = match self.agent.get(&url).call() {
            Ok(response) if (200..300).contains(&response.status()) => response,
            Ok(response) => bail!("GET {url} returned HTTP {}", response.status()),
            Err(ureq::Error::Status(404, _)) if path == DEFAULT_API_DISCOVER_PATH => {
                bail!(
                    "discovery endpoint unavailable (HTTP 404): GET {url}. Check the broker API address and build profile."
                );
            }
            Err(ureq::Error::Status(404, response)) => {
                if path
                    .strip_prefix(TOPICS_PATH)
                    .is_some_and(|suffix| suffix.starts_with('/'))
                {
                    let body = response
                        .into_string()
                        .with_context(|| format!("read HTTP 404 response from {url}"))?;
                    if body.trim() == "topic not found" {
                        bail!("topic not found (HTTP 404): GET {url}");
                    }
                }
                bail!(
                    "monitoring API unavailable (HTTP 404): GET {url}. Try `rbus discover` against the same API address."
                );
            }
            Err(ureq::Error::Status(status, _)) => bail!("GET {url} returned HTTP {status}"),
            Err(error) => {
                let error = anyhow::Error::new(error);
                if error.chain().any(|cause| {
                    cause
                        .downcast_ref::<io::Error>()
                        .is_some_and(|e| e.kind() == io::ErrorKind::TimedOut)
                }) {
                    return Err(error.context(format!("GET {url} timed out")));
                }
                return Err(error.context(format!(
                    "could not reach broker API at {}. Check the broker's API listen address and --url",
                    self.base
                )));
            }
        };
        let body: Value = response.into_json().map_err(|error| {
            let context = if error.kind() == io::ErrorKind::TimedOut {
                format!("GET {url} timed out reading the response")
            } else {
                format!("invalid JSON response from {url}")
            };
            anyhow::Error::new(error).context(context)
        })?;
        let value =
            T::deserialize(&body).with_context(|| format!("invalid JSON response from {url}"))?;
        Ok((value, body))
    }
}
