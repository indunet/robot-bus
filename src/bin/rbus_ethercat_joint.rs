//! CLI: EtherCAT CiA402 joints → JointState + JointCommand (feature `ethercat-joint`).

use anyhow::{Context, Result};
use clap::Parser;
use robot_bus::ethercat_joint::{list_devices, run, EXAMPLE_CONFIG};
use robot_bus::NodeOptions;

#[derive(Debug, Parser)]
#[command(
    name = "rbus_ethercat_joint",
    about = "EtherCAT / CiA402 joint bridge: publish sensor_msgs/JointState, subscribe robot_bus_interface/JointCommand (CSP/CSV/CST)"
)]
struct Args {
    /// Node name on the bus.
    #[arg(long, default_value = "ethercat_joint")]
    name: String,

    /// YAML parameter file (ros__parameters; includes nested `joints:` list).
    #[arg(long)]
    params: Option<String>,

    /// Print an example parameter YAML to stdout and exit.
    #[arg(long)]
    print_example_config: bool,

    /// Transport: tcp | ipc (default tcp).
    #[arg(long, default_value = "tcp")]
    transport: String,

    /// Broker host for tcp transport.
    #[arg(long, default_value = "localhost")]
    host: String,

    /// IPC directory when transport=ipc (must match broker).
    #[arg(long, default_value = "/tmp/robot_bus")]
    ipc_dir: String,

    /// List EtherCAT subdevices (requires --params) and exit.
    #[arg(long)]
    list_devices: bool,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    if args.print_example_config {
        print!("{EXAMPLE_CONFIG}");
        return Ok(());
    }
    if args.list_devices {
        return list_devices(args.params.as_deref());
    }
    if args.params.is_none() {
        anyhow::bail!("--params is required (joint map is configuration); try --print-example-config");
    }

    let options = match args.transport.as_str() {
        "tcp" => NodeOptions::tcp_at(&args.host),
        "ipc" => NodeOptions::ipc_at(&args.ipc_dir),
        other => anyhow::bail!("unsupported transport {other:?}; use tcp or ipc"),
    };

    run(&args.name, options, args.params.as_deref())
        .with_context(|| format!("run ethercat joint node {}", args.name))?;
    Ok(())
}
