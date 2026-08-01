use anyhow::{Context, Result};
use clap::Parser;
use robot_bus::NodeOptions;
use robot_bus_image_encoder::run;

#[derive(Debug, Parser)]
#[command(
    name = "robot-bus-image-encoder",
    about = "Subscribe to sensor_msgs/Image and publish foxglove CompressedVideo (H.264/H.265 via FFmpeg)"
)]
struct Args {
    /// Node name on the bus.
    #[arg(long, default_value = "image_encoder")]
    name: String,

    /// YAML parameter file (ros__parameters or flat map).
    #[arg(long)]
    params: Option<String>,

    /// Transport: tcp | ipc (default tcp).
    #[arg(long, default_value = "tcp")]
    transport: String,

    /// Broker host for tcp transport.
    #[arg(long, default_value = "localhost")]
    host: String,

    /// IPC directory when transport=ipc (must match broker).
    #[arg(long, default_value = "/tmp/robot_bus")]
    ipc_dir: String,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    let options = match args.transport.as_str() {
        "tcp" => NodeOptions::tcp_at(&args.host),
        "ipc" => NodeOptions::ipc_at(&args.ipc_dir),
        other => anyhow::bail!("unsupported transport {other:?}; use tcp or ipc"),
    };

    run(
        &args.name,
        options,
        args.params.as_deref(),
    )
    .with_context(|| format!("run image encoder node {}", args.name))?;
    Ok(())
}
