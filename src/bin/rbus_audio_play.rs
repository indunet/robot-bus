//! CLI: RawAudio → speaker (feature `audio-play`).

use anyhow::{Context, Result};
use clap::Parser;
use robot_bus::audio_play::{list_output_devices, run, EXAMPLE_CONFIG};
use robot_bus::NodeOptions;

#[derive(Debug, Parser)]
#[command(
    name = "rbus_audio_play",
    about = "Subscribe to foxglove RawAudio (pcm-s16) and play on a speaker"
)]
struct Args {
    /// Node name on the bus.
    #[arg(long, default_value = "audio_play")]
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

    /// List output devices and exit.
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
        return list_output_devices();
    }

    let options = match args.transport.as_str() {
        "tcp" => NodeOptions::tcp_at(&args.host),
        "ipc" => NodeOptions::ipc_at(&args.ipc_dir),
        other => anyhow::bail!("unsupported transport {other:?}; use tcp or ipc"),
    };

    run(&args.name, options, args.params.as_deref())
        .with_context(|| format!("run audio play node {}", args.name))?;
    Ok(())
}
