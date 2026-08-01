//! Discover / select connected joys via gilrs.

use anyhow::{bail, Result};
use gilrs::{GamepadId, Gilrs};

/// Print connected joys and exit-style listing for the CLI.
pub fn list_joys() -> Result<()> {
    let gilrs = Gilrs::new().map_err(|e| anyhow::anyhow!("init gilrs: {e}"))?;
    let mut n = 0usize;
    for (i, (id, gp)) in gilrs.gamepads().enumerate() {
        println!(
            "[{i}] id={id} name={:?} connected={} ff_supported={}",
            gp.name(),
            gp.is_connected(),
            gp.is_ff_supported()
        );
        n += 1;
    }
    if n == 0 {
        println!("(no joys connected)");
    }
    Ok(())
}

/// Resolve `device` selector to a connected [`GamepadId`].
///
/// - empty → first connected joy
/// - decimal index → N-th entry in `gilrs.gamepads()` order
/// - otherwise → case-insensitive substring match on the device name
pub fn resolve_joy_id(gilrs: &Gilrs, device: &str) -> Result<GamepadId> {
    let connected: Vec<(GamepadId, String)> = gilrs
        .gamepads()
        .filter(|(_, gp)| gp.is_connected())
        .map(|(id, gp)| (id, gp.name().to_string()))
        .collect();

    if connected.is_empty() {
        bail!("no connected joy");
    }

    let sel = device.trim();
    if sel.is_empty() {
        return Ok(connected[0].0);
    }

    if let Ok(index) = sel.parse::<usize>() {
        return connected
            .get(index)
            .map(|(id, _)| *id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "joy index {index} out of range (0..{})",
                    connected.len().saturating_sub(1)
                )
            });
    }

    let needle = sel.to_ascii_lowercase();
    let matches: Vec<_> = connected
        .iter()
        .filter(|(_, name)| name.to_ascii_lowercase().contains(&needle))
        .collect();
    match matches.as_slice() {
        [(id, _)] => Ok(*id),
        [] => bail!("no connected joy matching name substring {sel:?}"),
        many => bail!(
            "ambiguous device {sel:?}; matches: {}",
            many.iter()
                .map(|(_, n)| n.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
