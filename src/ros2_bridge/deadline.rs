//! A single monotonic deadline for every phase of a bridge RPC.
use std::sync::{Mutex, MutexGuard, TryLockError};
use std::time::{Duration, Instant};
use crate::errors::{BusError, Result};

#[derive(Clone, Copy)]
pub(crate) struct Deadline(Instant);
impl Deadline {
    pub(crate) fn new(timeout: Duration) -> Self { Self(Instant::now() + timeout) }
    pub(crate) fn remaining(self) -> Result<Duration> {
        self.0.checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| BusError::Timeout("ROS bridge RPC deadline exceeded".into()))
    }
    pub(crate) fn lock<T>(self, mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
        loop {
            self.remaining()?;
            match mutex.try_lock() {
                Ok(guard) => return Ok(guard),
                Err(TryLockError::Poisoned(e)) => return Err(BusError::Protocol(e.to_string())),
                Err(TryLockError::WouldBlock) => std::thread::sleep(self.remaining()?.min(Duration::from_millis(2))),
            }
        }
    }
}
