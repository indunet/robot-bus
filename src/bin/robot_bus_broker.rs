use anyhow::Result;
use robot_bus::broker::{RobotBusBroker, RobotBusConfig};
use robot_bus::shutdown;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
    let broker = RobotBusBroker::start(RobotBusConfig::default())?;

    while !shutdown.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(50));
    }

    broker.stop()?;
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
To run a single bus only, use message_bus_broker, service_bus_broker, or action_bus_broker.\n\
To embed in application code, use robot_bus::RobotBusBroker::start(...).\n"
    );
}
