//! Output device enumeration and resolution via cpal.

use anyhow::{bail, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device;

/// Print available output devices to stdout (name + default marker).
pub fn list_output_devices() -> Result<()> {
    let host = cpal::default_host();
    let default_name = host
        .default_output_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    println!("cpal host: {:?}", host.id());
    println!("output devices:");
    let mut any = false;
    for device in host.output_devices().context("enumerate output devices")? {
        let name = device.name().unwrap_or_else(|_| "<unknown>".into());
        let marker = if name == default_name { " (default)" } else { "" };
        println!("  - {name}{marker}");
        any = true;
    }
    if !any {
        println!("  (none)");
    }
    Ok(())
}

/// Resolve an output device: empty name → default output; otherwise match by exact name.
pub fn resolve_output_device(name: &str) -> Result<Device> {
    let host = cpal::default_host();
    if name.is_empty() {
        return host
            .default_output_device()
            .context("no default output device");
    }

    for device in host.output_devices().context("enumerate output devices")? {
        if device.name().ok().as_deref() == Some(name) {
            return Ok(device);
        }
    }
    bail!("output device not found: {name:?} (use --list-devices)");
}
