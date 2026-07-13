use anyhow::{Context, Result};
use robot_bus::broker::action_bus::{run_with_shutdown as run_action, ActionBusConfig};
use robot_bus::broker::message_bus::{run_with_shutdown as run_message, BusConfig};
use robot_bus::broker::service_bus::{run_with_shutdown as run_service, ServiceBusConfig};
use robot_bus::shutdown;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }
    if !args.is_empty() {
        anyhow::bail!("unknown arguments: {args:?} (try --help)");
    }

    let shutdown = Arc::new(AtomicBool::new(false));
    shutdown::install(shutdown.clone());

    println!("robot_bus_broker starting message + service + action buses…");

    let handles: Vec<_> = vec![
        thread::spawn({
            let shutdown = shutdown.clone();
            move || run_message(BusConfig::default(), shutdown)
        }),
        thread::spawn({
            let shutdown = shutdown.clone();
            move || run_service(ServiceBusConfig::default(), shutdown)
        }),
        thread::spawn({
            let shutdown = shutdown.clone();
            move || run_action(ActionBusConfig::default(), shutdown)
        }),
    ];

    for (name, handle) in ["message_bus", "service_bus", "action_bus"]
        .into_iter()
        .zip(handles)
    {
        let result = handle
            .join()
            .map_err(|e| anyhow::anyhow!("{name} thread panicked: {e:?}"))?;
        result.with_context(|| format!("{name} exited with error"))?;
    }

    println!("robot_bus_broker stopped");
    Ok(())
}

fn print_help() {
    println!(
        "robot_bus_broker — start all ZeroMQ buses in one process\n\n\
Usage:\n  robot_bus_broker\n\n\
Starts with default ports and tcp + inproc + ipc on each socket:\n  \
message_bus  15560 / 15561 (XSUB/XPUB proxy)\n  \
service_bus  15662 / 15663 (REQ service broker)\n  \
action_bus   15664 / 15665 (DEALER action broker)\n\n\
Press Ctrl+C to stop all buses.\n\n\
To run a single bus only, use message_bus_broker, service_bus_broker, or action_bus_broker.\n"
    );
}
