//! `rbus` — ROS 2–style introspection CLI over the broker console HTTP API.
//!
//! Phase 1: `topic|service|action list` and `status`. Default console URL is
//! `http://127.0.0.1:15771` (override with `--url` or `ROBOT_BUS_BROKER_URL`).

use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde::Deserialize;

const DEFAULT_URL: &str = "http://127.0.0.1:15771";
const ENV_BROKER_URL: &str = "ROBOT_BUS_BROKER_URL";

#[derive(Parser, Debug)]
#[command(
    name = "rbus",
    version,
    about = "robot-bus introspection CLI (console HTTP API)",
    long_about = None
)]
struct Cli {
    /// Console base URL (default: ROBOT_BUS_BROKER_URL or http://127.0.0.1:15771)
    #[arg(long, global = true)]
    url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
    /// List topics with recent forwarded traffic
    List,
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
struct NamedList {
    #[serde(default)]
    topics: Vec<Named>,
    #[serde(default)]
    services: Vec<Named>,
    #[serde(default)]
    actions: Vec<Named>,
}

#[derive(Deserialize)]
struct Named {
    name: String,
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
    msg_per_sec: u64,
    bytes_per_sec: u64,
    svc_calls_per_sec: u64,
    act_runs_per_sec: u64,
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

    match cli.command {
        Commands::Topic {
            command: TopicCmd::List,
        } => {
            let body: NamedList = get_json(&base, "/api/v1/topics")?;
            for t in body.topics {
                println!("{}", t.name);
            }
        }
        Commands::Service {
            command: ServiceCmd::List,
        } => {
            let body: NamedList = get_json(&base, "/api/v1/services")?;
            for s in body.services {
                println!("{}", s.name);
            }
        }
        Commands::Action {
            command: ActionCmd::List,
        } => {
            let body: NamedList = get_json(&base, "/api/v1/actions")?;
            for a in body.actions {
                println!("{}", a.name);
            }
        }
        Commands::Status => {
            let s: StatusResponse = get_json(&base, "/api/v1/status")?;
            println!("status: {}", s.status);
            println!("version: {}", s.version);
            println!("uptime_secs: {}", s.uptime);
            println!("pid: {}", s.pid);
            println!("grpc: {}", s.grpc_addr);
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
        }
    }
    Ok(())
}

fn resolve_base_url(cli_url: Option<&str>) -> String {
    if let Some(u) = cli_url {
        return trim_trailing_slash(u);
    }
    if let Ok(u) = std::env::var(ENV_BROKER_URL) {
        if !u.trim().is_empty() {
            return trim_trailing_slash(u.trim());
        }
    }
    DEFAULT_URL.to_string()
}

fn trim_trailing_slash(s: &str) -> String {
    s.trim_end_matches('/').to_string()
}

fn get_json<T: for<'de> Deserialize<'de>>(base: &str, path: &str) -> Result<T> {
    let url = format!("{base}{path}");
    let response = ureq::get(&url).call().map_err(|e| {
        anyhow::anyhow!(
            "failed to GET {url}: {e}\n\
             Is robot_bus_broker running with the console enabled?\n\
             Default URL is {DEFAULT_URL} (override with --url or {ENV_BROKER_URL})."
        )
    })?;

    let status = response.status();
    if !(200..300).contains(&status) {
        bail!("GET {url} returned HTTP {status}");
    }

    response
        .into_json::<T>()
        .with_context(|| format!("decode JSON from {url}"))
}
