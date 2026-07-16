//! Test helpers: inline message proxy and subprocess broker binaries.

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use zmq::{Context, Socket, SocketType};

static PROXY_ID: AtomicU64 = AtomicU64::new(0);

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

pub struct MessageProxy {
    pub xsub_endpoint: String,
    pub xpub_endpoint: String,
    handle: Option<JoinHandle<()>>,
    control_client: Socket,
    context: Context,
}

impl MessageProxy {
    pub fn spawn() -> Self {
        let context = Context::new();
        let mut xsub = context.socket(SocketType::XSUB).expect("xsub");
        let mut xpub = context.socket(SocketType::XPUB).expect("xpub");
        let mut control = context.socket(SocketType::PAIR).expect("control");
        let control_client = context.socket(SocketType::PAIR).expect("control client");
        let xsub_port = free_port();
        let xpub_port = free_port();
        let xsub_endpoint = format!("tcp://127.0.0.1:{xsub_port}");
        let xpub_endpoint = format!("tcp://127.0.0.1:{xpub_port}");
        xsub.bind(&xsub_endpoint).expect("bind xsub");
        xpub.bind(&xpub_endpoint).expect("bind xpub");
        xsub.set_linger(0).expect("linger");
        xpub.set_linger(0).expect("linger");

        let id = PROXY_ID.fetch_add(1, Ordering::Relaxed);
        let control_name = format!("inproc://message-proxy-ctl-{id}");
        control.bind(&control_name).expect("bind control");
        control_client.connect(&control_name).expect("connect control");

        let handle = thread::spawn(move || {
            let _ = zmq::proxy_steerable(&mut xsub, &mut xpub, &mut control);
        });
        thread::sleep(Duration::from_millis(50));
        Self {
            xsub_endpoint,
            xpub_endpoint,
            handle: Some(handle),
            control_client,
            context,
        }
    }
}

impl Drop for MessageProxy {
    fn drop(&mut self) {
        let _ = self.control_client.send(b"TERMINATE" as &[u8], 0);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

pub struct BrokerProcess {
    pub frontend_endpoint: String,
    pub backend_endpoint: String,
    child: Child,
}

impl BrokerProcess {
    pub fn spawn_service() -> Self {
        ensure_broker_built();
        let frontend_port = free_port();
        let backend_port = free_port();
        let frontend = format!("tcp://127.0.0.1:{frontend_port}");
        let backend = format!("tcp://127.0.0.1:{backend_port}");
        let child = Command::new(broker_bin("service_bus_broker"))
            .args([
                "--frontend-bind",
                &frontend,
                "--backend-bind",
                &backend,
                "--heartbeat-interval-ms",
                "200",
                "--heartbeat-timeout-ms",
                "600",
            ])
            .current_dir(repo_root())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn service_bus_broker");
        thread::sleep(Duration::from_millis(150));
        assert!(child.id() > 0, "service_bus_broker failed to start");
        Self {
            frontend_endpoint: frontend,
            backend_endpoint: backend,
            child,
        }
    }

    pub fn spawn_action() -> Self {
        ensure_broker_built();
        let frontend_port = free_port();
        let backend_port = free_port();
        let frontend = format!("tcp://127.0.0.1:{frontend_port}");
        let backend = format!("tcp://127.0.0.1:{backend_port}");
        let child = Command::new(broker_bin("action_bus_broker"))
            .args([
                "--frontend-bind",
                &frontend,
                "--backend-bind",
                &backend,
                "--heartbeat-interval-ms",
                "100",
                "--heartbeat-timeout-ms",
                "600",
                "--pending-timeout-ms",
                "200",
                "--tcp-only",
            ])
            .current_dir(repo_root())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn action_bus_broker");
        thread::sleep(Duration::from_millis(150));
        Self {
            frontend_endpoint: frontend,
            backend_endpoint: backend,
            child,
        }
    }
}

impl Drop for BrokerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn broker_bin(name: &str) -> PathBuf {
    repo_root().join("target").join("debug").join(name)
}

fn ensure_broker_built() {
    for name in ["service_bus_broker", "action_bus_broker"] {
        let path = broker_bin(name);
        if !path.is_file() {
            let status = Command::new("cargo")
                .args(["build", "--quiet", "--bin", name])
                .current_dir(repo_root())
                .status()
                .expect("cargo build broker");
            assert!(status.success(), "failed to build {name}");
        }
    }
}
