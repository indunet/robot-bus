use anyhow::{Context, Result};
use robot_bus::broker::message_bus::{
    run, BusConfig, DEFAULT_RCV_HWM, DEFAULT_SND_HWM, XPUB_CHANNEL, XSUB_CHANNEL,
};
use robot_bus::transports::{inproc_endpoint, ipc_endpoint};
use std::env;

fn main() -> Result<()> {
    run(parse_config()?)
}

fn parse_config() -> Result<BusConfig> {
    let mut config = BusConfig::default();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--xsub-bind" | "--pub-bind" => {
                let addr = args.next().context("--xsub-bind requires an address")?;
                config.xsub_bind = normalize_tcp_bind(&addr);
            }
            "--xpub-bind" | "--sub-bind" => {
                let addr = args.next().context("--xpub-bind requires an address")?;
                config.xpub_bind = normalize_tcp_bind(&addr);
            }
            "--snd-hwm" => {
                config.snd_hwm = args
                    .next()
                    .context("--snd-hwm requires a number")?
                    .parse()
                    .context("invalid --snd-hwm")?;
            }
            "--rcv-hwm" => {
                config.rcv_hwm = args
                    .next()
                    .context("--rcv-hwm requires a number")?
                    .parse()
                    .context("invalid --rcv-hwm")?;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => anyhow::bail!("unknown argument: {arg}"),
        }
    }

    Ok(config)
}

fn normalize_tcp_bind(addr: &str) -> String {
    if addr.contains("://") {
        addr.to_string()
    } else {
        format!("tcp://{addr}")
    }
}

fn print_help() {
    println!(
        "message_bus_broker — ZeroMQ XSUB/XPUB transparent proxy\n\n\
Usage:\n  message_bus_broker [--xsub-bind tcp://0.0.0.0:15560] [--xpub-bind tcp://0.0.0.0:15561]\n\n\
Ports (aligned with dji_app PhoneStreamTopics):\n  \
XSUB 15560 — publishers (JeroMQ PUB)\n  \
XPUB 15561 — subscribers (JeroMQ SUB)\n\n\
Transports (each socket binds all three; prefer inproc/ipc on same machine):\n  \
publishers: tcp://host:15560 | {} | {}\n  \
subscribers: tcp://host:15561 | {} | {}\n\n\
Wire format:\n  \
Opaque ZMTP frames, typically multipart [topic UTF-8][payload bytes].\n  \
Protobuf/JSON/binary encoding is entirely up to publishers and subscribers.\n\n\
Options:\n  \
--snd-hwm N   send high-water mark (default {DEFAULT_SND_HWM})\n  \
--rcv-hwm N   receive high-water mark (default {DEFAULT_RCV_HWM})\n",
        inproc_endpoint(XSUB_CHANNEL),
        ipc_endpoint(XSUB_CHANNEL),
        inproc_endpoint(XPUB_CHANNEL),
        ipc_endpoint(XPUB_CHANNEL),
    );
}
