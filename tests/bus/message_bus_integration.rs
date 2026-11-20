//! Integration tests for the message bus XSUB/XPUB transparent proxy.
//!
//! Publishers (PUB) connect to the XSUB side; subscribers (SUB) connect to the
//! XPUB side. The proxy forwards multipart frames without parsing them.

use robot_bus::transports::{BindAllOpts, bind_all, inproc_endpoint};
use std::thread;
use std::time::Duration;
use zmq::{Context as ZmqContext, SocketType};

fn bind_ephemeral(ctx: &ZmqContext, ty: SocketType) -> (zmq::Socket, String) {
    let sock = ctx.socket(ty).expect("create socket");
    sock.bind("tcp://127.0.0.1:0").expect("bind ephemeral");
    let endpoint = match sock.get_last_endpoint().expect("last_endpoint") {
        Ok(s) => s,
        Err(_) => panic!("endpoint not utf8"),
    };
    (sock, endpoint)
}

/// Spawn an in-process XSUB/XPUB proxy on ephemeral TCP ports.
struct ProxyHandle {
    control: zmq::Socket,
    handle: Option<thread::JoinHandle<()>>,
    xsub_ep: String,
    xpub_ep: String,
}

impl Drop for ProxyHandle {
    fn drop(&mut self) {
        let _ = self.control.send(b"TERMINATE".as_ref(), 0);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn spawn_proxy() -> ProxyHandle {
    let ctx = ZmqContext::new();
    let (mut xsub, xsub_ep) = bind_ephemeral(&ctx, SocketType::XSUB);
    let (mut xpub, xpub_ep) = bind_ephemeral(&ctx, SocketType::XPUB);
    let mut control = ctx.socket(SocketType::PAIR).expect("create control PAIR");
    control
        .bind("inproc://message-bus-proxy-ctl")
        .expect("bind control");
    let control_client = ctx.socket(SocketType::PAIR).expect("create control client");
    control_client
        .connect("inproc://message-bus-proxy-ctl")
        .expect("connect control");

    for sock in [&xsub, &xpub] {
        sock.set_linger(0).expect("set linger");
    }

    let handle = thread::spawn(move || {
        let _ = zmq::proxy_steerable(&mut xsub, &mut xpub, &mut control);
    });
    thread::sleep(Duration::from_millis(50));

    ProxyHandle {
        control: control_client,
        handle: Some(handle),
        xsub_ep,
        xpub_ep,
    }
}

fn make_pub(xsub_ep: &str) -> zmq::Socket {
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::PUB).expect("create PUB");
    sock.connect(xsub_ep).expect("connect XSUB");
    sock
}

fn make_sub(xpub_ep: &str, topic: &str) -> zmq::Socket {
    let ctx = ZmqContext::new();
    let sock = ctx.socket(SocketType::SUB).expect("create SUB");
    sock.connect(xpub_ep).expect("connect XPUB");
    sock.set_subscribe(topic.as_bytes()).expect("set subscribe");
    sock
}

fn wait_for_subscription() {
    // XPUB/XSUB must propagate the SUB subscription before filtered delivery works.
    thread::sleep(Duration::from_millis(150));
}

#[test]
fn e2e_forward_multipart() {
    let proxy = spawn_proxy();
    let pub_sock = make_pub(&proxy.xsub_ep);
    let sub_sock = make_sub(&proxy.xpub_ep, "wireless.imu");
    wait_for_subscription();

    pub_sock
        .send_multipart([b"wireless.imu".as_ref(), b"imu-bytes".as_ref()], 0)
        .expect("pub send");

    sub_sock.set_rcvtimeo(2000).expect("set rcvtimeo");
    let frames = sub_sock.recv_multipart(0).expect("sub recv");
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0], b"wireless.imu");
    assert_eq!(frames[1], b"imu-bytes");
}

#[test]
fn topic_filter() {
    let proxy = spawn_proxy();
    let pub_sock = make_pub(&proxy.xsub_ep);
    let sub_sock = make_sub(&proxy.xpub_ep, "wireless.imu");
    wait_for_subscription();

    pub_sock
        .send_multipart([b"wireless.camera.h264".as_ref(), b"video".as_ref()], 0)
        .expect("pub video");
    pub_sock
        .send_multipart([b"wireless.imu".as_ref(), b"imu".as_ref()], 0)
        .expect("pub imu");

    sub_sock.set_rcvtimeo(2000).expect("set rcvtimeo");
    let frames = sub_sock.recv_multipart(0).expect("sub recv");
    assert_eq!(frames[0], b"wireless.imu");
    assert_eq!(frames[1], b"imu");
}

#[test]
fn multiple_messages_in_order() {
    let proxy = spawn_proxy();
    let pub_sock = make_pub(&proxy.xsub_ep);
    let sub_sock = make_sub(&proxy.xpub_ep, "topic.v1");
    wait_for_subscription();

    for i in 0..3 {
        pub_sock
            .send_multipart([b"topic.v1".as_ref(), format!("payload-{i}").as_bytes()], 0)
            .expect("pub send");
    }

    sub_sock.set_rcvtimeo(2000).expect("set rcvtimeo");
    for i in 0..3 {
        let frames = sub_sock.recv_multipart(0).expect("sub recv");
        assert_eq!(frames[0], b"topic.v1");
        assert_eq!(frames[1], format!("payload-{i}").into_bytes());
    }
}

#[test]
fn inproc_forward_multipart() {
    const XSUB_CH: &str = "test/message_bus/xsub";
    const XPUB_CH: &str = "test/message_bus/xpub";

    // inproc requires all sockets to share the same ZmqContext.
    let ctx = ZmqContext::new();
    let xsub = ctx.socket(SocketType::XSUB).expect("create XSUB");
    let xpub = ctx.socket(SocketType::XPUB).expect("create XPUB");
    bind_all(&xsub, "tcp://127.0.0.1:0", XSUB_CH, &BindAllOpts::default()).expect("bind xsub");
    bind_all(&xpub, "tcp://127.0.0.1:0", XPUB_CH, &BindAllOpts::default()).expect("bind xpub");

    let control = ctx.socket(SocketType::PAIR).expect("create control");
    control
        .bind("inproc://message-bus-inproc-test-ctl")
        .expect("bind control");
    let control_client = ctx.socket(SocketType::PAIR).expect("create control client");
    control_client
        .connect("inproc://message-bus-inproc-test-ctl")
        .expect("connect control");

    for sock in [&xsub, &xpub] {
        sock.set_linger(0).expect("set linger");
    }

    let mut xsub = xsub;
    let mut xpub = xpub;
    let mut control = control;
    let proxy = thread::spawn(move || {
        let _ = zmq::proxy_steerable(&mut xsub, &mut xpub, &mut control);
    });
    thread::sleep(Duration::from_millis(50));

    let pub_sock = ctx.socket(SocketType::PUB).expect("create PUB");
    pub_sock
        .connect(&inproc_endpoint(XSUB_CH))
        .expect("connect XSUB inproc");
    let sub_sock = ctx.socket(SocketType::SUB).expect("create SUB");
    sub_sock
        .connect(&inproc_endpoint(XPUB_CH))
        .expect("connect XPUB inproc");
    sub_sock
        .set_subscribe(b"wireless.imu")
        .expect("set subscribe");
    wait_for_subscription();

    pub_sock
        .send_multipart([b"wireless.imu".as_ref(), b"via-inproc".as_ref()], 0)
        .expect("pub send");
    sub_sock.set_rcvtimeo(2000).expect("set rcvtimeo");
    let frames = sub_sock.recv_multipart(0).expect("sub recv");
    assert_eq!(frames[1], b"via-inproc");

    let _ = control_client.send(b"TERMINATE".as_ref(), 0);
    let _ = proxy.join();
}
