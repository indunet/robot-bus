//! Camera enumeration and resolution via nokhwa.

use anyhow::{bail, Context, Result};
use nokhwa::utils::{ApiBackend, CameraIndex};
use nokhwa::{native_api_backend, query};

/// Ensure camera permission on macOS (no-op elsewhere).
pub fn ensure_camera_permission() -> Result<()> {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        use std::sync::mpsc;
        use std::time::Duration;

        if nokhwa::nokhwa_check() {
            return Ok(());
        }

        let (tx, rx) = mpsc::channel();
        nokhwa::nokhwa_initialize(move |ok| {
            let _ = tx.send(ok);
        });
        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(true) => Ok(()),
            Ok(false) => bail!("camera permission denied"),
            Err(_) => {
                if nokhwa::nokhwa_check() {
                    Ok(())
                } else {
                    bail!("camera permission timed out (grant access in System Settings)")
                }
            }
        }
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        Ok(())
    }
}

fn backend() -> Result<ApiBackend> {
    native_api_backend().context("no native camera backend for this platform")
}

/// Print available cameras to stdout (index + name).
pub fn list_cameras() -> Result<()> {
    ensure_camera_permission()?;
    let api = backend()?;
    println!("nokhwa backend: {api:?}");
    println!("cameras:");
    let devices = query(api).context("enumerate cameras")?;
    if devices.is_empty() {
        println!("  (none)");
        return Ok(());
    }
    for info in devices {
        let idx = info.index();
        let name = info.human_name();
        let misc = info.misc();
        if misc.is_empty() {
            println!("  - [{idx}] {name}");
        } else {
            println!("  - [{idx}] {name} ({misc})");
        }
    }
    Ok(())
}

/// Resolve a camera index: empty → 0; numeric string → that index; else match human name.
pub fn resolve_camera_index(name: &str) -> Result<CameraIndex> {
    ensure_camera_permission()?;
    if name.is_empty() {
        return Ok(CameraIndex::Index(0));
    }
    if let Ok(i) = name.parse::<u32>() {
        return Ok(CameraIndex::Index(i));
    }

    let api = backend()?;
    let devices = query(api).context("enumerate cameras")?;
    for info in &devices {
        if info.human_name() == name {
            return Ok(info.index().clone());
        }
    }

    let available: Vec<String> = devices
        .iter()
        .map(|d| format!("[{}] {}", d.index(), d.human_name()))
        .collect();
    bail!(
        "camera not found: {name:?}; available: {} (use --list-devices)",
        if available.is_empty() {
            "(none)".into()
        } else {
            available.join(", ")
        }
    );
}

/// Human-readable label for logging after open.
pub fn describe_index(index: &CameraIndex) -> String {
    let api = match backend() {
        Ok(b) => b,
        Err(_) => return index.to_string(),
    };
    let Ok(devices) = query(api) else {
        return index.to_string();
    };
    for info in devices {
        if info.index() == index {
            return format!("[{index}] {}", info.human_name());
        }
    }
    index.to_string()
}
