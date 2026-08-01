//! Input device enumeration and resolution via cpal.

use anyhow::{bail, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device;

/// Print available input devices to stdout (name + default marker).
pub fn list_input_devices() -> Result<()> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_default();

    println!("cpal host: {:?}", host.id());
    println!("input devices:");
    let mut any = false;
    for device in host.input_devices().context("enumerate input devices")? {
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

/// Resolve an input device: empty name → default input; otherwise match by exact name.
pub fn resolve_input_device(name: &str) -> Result<Device> {
    let host = cpal::default_host();
    if name.is_empty() {
        return host
            .default_input_device()
            .context("no default input device");
    }

    for device in host.input_devices().context("enumerate input devices")? {
        if device.name().ok().as_deref() == Some(name) {
            return Ok(device);
        }
    }
    bail!("input device not found: {name:?} (use --list-devices)");
}
