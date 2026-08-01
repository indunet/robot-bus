//! CLI: USB camera → sensor_msgs/Image rgb8 (feature `usb-camera`).

use anyhow::{Context, Result};
use clap::Parser;
use robot_bus::usb_camera::{list_cameras, run, EXAMPLE_CONFIG};
use robot_bus::NodeOptions;

#[derive(Debug, Parser)]
#[command(
    name = "rbus_usb_camera",
    about = "Capture USB / webcam frames and publish sensor_msgs/Image (rgb8)"
)]
struct Args {
    /// Node name on the bus.
    #[arg(long, default_value = "usb_camera")]
    name: String,

    /// YAML parameter file (ros__parameters or flat map).
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

    /// List cameras and exit.
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
        return list_cameras();
    }

    let options = match args.transport.as_str() {
        "tcp" => NodeOptions::tcp_at(&args.host),
        "ipc" => NodeOptions::ipc_at(&args.ipc_dir),
        other => anyhow::bail!("unsupported transport {other:?}; use tcp or ipc"),
    };
    run(&args.name, options, args.params.as_deref())
        .with_context(|| format!("run usb camera node {}", args.name))?;
    Ok(())
}
