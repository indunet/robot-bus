//! Background worker runner for tests and simple deployments.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use crate::action_bus::{ActionGoalHandler, ActionWorker};
use crate::errors::Result;
use crate::service_bus::{ServiceHandler, ServiceWorker};

pub struct WorkerThread {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl WorkerThread {
    pub fn spawn_service(
        service_name: impl Into<String>,
        handler: ServiceHandler,
        endpoint: impl Into<String>,
    ) -> Result<Self> {
        let service_name = service_name.into();
        let endpoint = endpoint.into();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let handle = thread::spawn(move || {
            let mut worker =
                match ServiceWorker::new(service_name, handler, Some(&endpoint), None, 2500) {
                    Ok(worker) => worker,
                    Err(err) => {
                        log::error!("service worker startup failed: {err}");
                        return;
                    }
                };
            while !stop_flag.load(Ordering::Relaxed) {
                let _ = worker.serve_once(500);
            }
            worker.close();
        });
        Ok(Self { stop, handle })
    }

    pub fn spawn_action(
        action_name: impl Into<String>,
        handler: ActionGoalHandler,
        endpoint: impl Into<String>,
    ) -> Result<Self> {
        let action_name = action_name.into();
        let endpoint = endpoint.into();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();
        let handle = thread::spawn(move || {
            let mut worker =
                match ActionWorker::new(action_name, handler, Some(&endpoint), None, 2500) {
                    Ok(worker) => worker,
                    Err(err) => {
                        log::error!("action worker startup failed: {err}");
                        return;
                    }
                };
            while !stop_flag.load(Ordering::Relaxed) {
                let _ = worker.serve_once(500);
            }
            worker.close();
        });
        Ok(Self { stop, handle })
    }

    pub fn stop(self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.handle.join();
    }
}
