use anyhow::Result;
use robot_bus::broker::{
    parse_robot_bus_config, robot_bus_broker_help, RobotBusBroker, RobotBusConfig,
};
use robot_bus::shutdown;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match parse_robot_bus_config(&args)? {
        None => {
            print!("{}", robot_bus_broker_help());
            return Ok(());
        }
        Some(config) => config,
    };

    run(config)
}

fn run(config: RobotBusConfig) -> Result<()> {
    let shutdown = Arc::new(AtomicBool::new(false));
    shutdown::install(shutdown.clone());

    println!("robot_bus_broker starting message + service + action buses + gRPC…");
    let broker = RobotBusBroker::start(config)?;

    #[cfg(feature = "grpc")]
    println!(
        "gRPC / gRPC-Web listening on http://{}",
        broker.grpc_listen()
    );

    while !shutdown.load(Ordering::Acquire) {
        thread::sleep(Duration::from_millis(50));
    }

    broker.stop()?;
    println!("robot_bus_broker stopped");
    Ok(())
}
