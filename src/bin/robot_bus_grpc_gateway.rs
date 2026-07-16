//! Standalone gRPC / gRPC-Web gateway over message / service / action buses.

use anyhow::{bail, Context, Result};
use robot_bus::grpc::{serve, GatewayConfig};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let mut config = GatewayConfig::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--listen" => {
                i += 1;
                let value = args
                    .get(i)
                    .context("--listen requires host:port")?;
                config.listen = value
                    .parse::<SocketAddr>()
                    .with_context(|| format!("invalid --listen {value}"))?;
            }
            "--message-xpub" => {
                i += 1;
                config.message_xpub = args
                    .get(i)
                    .context("--message-xpub requires endpoint")?
                    .clone();
            }
            "--service-frontend" => {
                i += 1;
                config.service_frontend = args
                    .get(i)
                    .context("--service-frontend requires endpoint")?
                    .clone();
            }
            "--action-frontend" => {
                i += 1;
                config.action_frontend = args
                    .get(i)
                    .context("--action-frontend requires endpoint")?
                    .clone();
            }
            "--cors-origin" => {
                i += 1;
                config.cors_origins.push(
                    args.get(i)
                        .context("--cors-origin requires a value")?
                        .clone(),
                );
            }
            other => bail!("unknown argument: {other} (try --help)"),
        }
        i += 1;
    }

    serve(config).await
}

fn print_help() {
    println!(
        "robot_bus_grpc_gateway — gRPC / gRPC-Web over message / service / action buses\n\n\
Usage:\n  robot_bus_grpc_gateway [options]\n\n\
Options:\n  \
--listen HOST:PORT            Listen address (default 0.0.0.0:15770)\n  \
--message-xpub ENDPOINT       Message bus XPUB connect endpoint\n  \
                              (default tcp://127.0.0.1:15561)\n  \
--service-frontend ENDPOINT   Service bus frontend connect endpoint\n  \
                              (default tcp://127.0.0.1:15662)\n  \
--action-frontend ENDPOINT    Action bus frontend connect endpoint\n  \
                              (default tcp://127.0.0.1:15664)\n  \
--cors-origin ORIGIN          Allowed browser origin (repeatable).\n  \
                              Default: allow any origin (local development).\n  \
--help, -h                    Show this help\n\n\
RPCs: MessageGateway.Subscribe, ServiceGateway.Call,\n  \
      ActionGateway.Run (bidi stream).\n\
Native gRPC and gRPC-Web share the same port (HTTP/2 + HTTP/1.1).\n\
Start robot_bus_broker (or the relevant single-bus brokers) first.\n"
    );
}
