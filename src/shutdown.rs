//! Shared SIGINT/SIGTERM handling for multi-bus processes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Install a handler that sets `flag` on SIGINT/SIGTERM (Unix only).
pub fn install(shutdown: Arc<AtomicBool>) {
    #[cfg(unix)]
    ctrlc_set_flag(shutdown);
    #[cfg(not(unix))]
    let _ = shutdown;
}

#[cfg(unix)]
fn ctrlc_set_flag(flag: Arc<AtomicBool>) {
    use std::os::raw::c_int;
    extern "C" fn handle(_sig: c_int) {
        SHUTDOWN.store(true, Ordering::Release);
    }
    static SHUTDOWN: AtomicBool = AtomicBool::new(false);
    SHUTDOWN.store(false, Ordering::Release);
    std::thread::spawn(move || loop {
        if SHUTDOWN.load(Ordering::Acquire) {
            flag.store(true, Ordering::Release);
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    });
    const SIGINT: c_int = 2;
    const SIGTERM: c_int = 15;
    unsafe {
        libc_signal(SIGINT, handle as *const () as usize);
        libc_signal(SIGTERM, handle as *const () as usize);
    }
}

#[cfg(unix)]
unsafe extern "C" {
    #[link_name = "signal"]
    fn libc_signal(signum: std::os::raw::c_int, handler: usize) -> usize;
}
