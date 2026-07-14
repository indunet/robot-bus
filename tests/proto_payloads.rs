//! Integration tests: ROS 2 protobuf payloads over in-process brokers.
//!
//! Body frames remain opaque to the bus; these tests encode/decode with
//! `robot_bus::msgs` to verify end-to-end typed message round-trips.

use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use prost::Message;
use robot_bus::action_bus::{ActionClient, ActionKind};
use robot_bus::broker::action_bus::ActionBusConfig;
use robot_bus::broker::message_bus::BusConfig;
use robot_bus::broker::service_bus::ServiceBusConfig;
use robot_bus::message_bus::{Publisher, Subscriber};
use robot_bus::msgs::geometry_msgs::v1::{
    Pose, Pose2D, PoseStamped, PoseWithCovariance, PoseWithCovarianceStamped, Twist, Vector3,
};
use robot_bus::msgs::nav_msgs::v1::{
    GetMapRequest, GetMapResponse, GetPlanRequest, GetPlanResponse, MapMetaData, OccupancyGrid,
    Odometry, Path, SetMapRequest, SetMapResponse,
};
use robot_bus::msgs::std_msgs::v1::{Header, Int32, String as ProtoString};
use robot_bus::msgs::std_srvs::v1::{
    SetBoolRequest, SetBoolResponse, TriggerRequest, TriggerResponse,
};
use robot_bus::service_bus::ServiceClient;
use robot_bus::worker_thread::WorkerThread;
use robot_bus::{ActionBusBroker, MessageBusBroker, ServiceBusBroker};

/// `bind_all` uses fixed inproc/ipc names — only one broker of each kind at a time.
static BROKER_LOCK: Mutex<()> = Mutex::new(());

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

fn lock_brokers() -> std::sync::MutexGuard<'static, ()> {
    BROKER_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn message_bus_twist_pubsub() {
    let _guard = lock_brokers();
    let broker = MessageBusBroker::start(BusConfig {
        xsub_bind: format!("tcp://127.0.0.1:{}", free_port()),
        xpub_bind: format!("tcp://127.0.0.1:{}", free_port()),
        ..BusConfig::default()
    })
    .expect("start message bus");

    let publisher = Publisher::new(Some(&broker.xsub_bind)).expect("publisher");
    thread::sleep(Duration::from_millis(50));
    let subscriber = Subscriber::new(Some(&broker.xpub_bind)).expect("subscriber");
    subscriber.subscribe("cmd_vel").expect("subscribe");
    thread::sleep(Duration::from_millis(150));

    let twist = Twist {
        linear: Some(Vector3 {
            x: 1.5,
            y: 0.0,
            z: 0.0,
        }),
        angular: Some(Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.25,
        }),
    };
    let payload = twist.encode_to_vec();
    publisher.publish("cmd_vel", &payload).expect("publish");

    let (topic, bytes) = subscriber
        .receive(Some(Duration::from_secs(2)))
        .expect("receive");
    assert_eq!(topic, "cmd_vel");
    let decoded = Twist::decode(bytes.as_slice()).expect("decode Twist");
    assert_eq!(decoded.linear.as_ref().unwrap().x, 1.5);
    assert_eq!(decoded.angular.as_ref().unwrap().z, 0.25);

    broker.stop().expect("stop");
}

#[test]
fn message_bus_odometry_pubsub() {
    let _guard = lock_brokers();
    let broker = MessageBusBroker::start(BusConfig {
        xsub_bind: format!("tcp://127.0.0.1:{}", free_port()),
        xpub_bind: format!("tcp://127.0.0.1:{}", free_port()),
        ..BusConfig::default()
    })
    .expect("start message bus");

    let publisher = Publisher::new(Some(&broker.xsub_bind)).expect("publisher");
    thread::sleep(Duration::from_millis(50));
    let subscriber = Subscriber::new(Some(&broker.xpub_bind)).expect("subscriber");
    subscriber.subscribe("odom").expect("subscribe");
    thread::sleep(Duration::from_millis(150));

    let odom = Odometry {
        header: Some(Header {
            stamp: Some(robot_bus::msgs::builtin_interfaces::v1::Time {
                sec: 10,
                nanosec: 500_000_000,
            }),
            frame_id: "odom".into(),
        }),
        child_frame_id: "base_link".into(),
        pose: None,
        twist: None,
    };
    publisher
        .publish("odom", &odom.encode_to_vec())
        .expect("publish");

    let (topic, bytes) = subscriber
        .receive(Some(Duration::from_secs(2)))
        .expect("receive");
    assert_eq!(topic, "odom");
    let decoded = Odometry::decode(bytes.as_slice()).expect("decode Odometry");
    assert_eq!(decoded.child_frame_id, "base_link");
    assert_eq!(decoded.header.as_ref().unwrap().frame_id, "odom");
    assert_eq!(decoded.header.as_ref().unwrap().stamp.as_ref().unwrap().sec, 10);

    broker.stop().expect("stop");
}

#[test]
fn service_bus_string_echo_proto() {
    let _guard = lock_brokers();
    let broker = ServiceBusBroker::start(ServiceBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ServiceBusConfig::default()
    })
    .expect("start service bus");

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| {
            let req = ProtoString::decode(body).expect("decode request");
            ProtoString {
                data: format!("echo:{}", req.data),
            }
            .encode_to_vec()
        });
    let worker = WorkerThread::spawn_service("svc.string_echo", handler, &broker.backend_bind)
        .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let client = ServiceClient::new(Some(&broker.frontend_bind)).expect("client");
    let req = ProtoString {
        data: "hello-proto".into(),
    };
    let reply_bytes = client
        .call(
            "svc.string_echo",
            &req.encode_to_vec(),
            None,
            Some(Duration::from_secs(10)),
        )
        .expect("call");
    let reply = ProtoString::decode(reply_bytes.as_slice()).expect("decode reply");
    assert_eq!(reply.data, "echo:hello-proto");

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn service_bus_int32_add_proto() {
    let _guard = lock_brokers();
    let broker = ServiceBusBroker::start(ServiceBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ServiceBusConfig::default()
    })
    .expect("start service bus");

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| {
            let req = Int32::decode(body).expect("decode Int32");
            Int32 {
                data: req.data.saturating_add(1),
            }
            .encode_to_vec()
        });
    let worker = WorkerThread::spawn_service("svc.int_inc", handler, &broker.backend_bind)
        .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let client = ServiceClient::new(Some(&broker.frontend_bind)).expect("client");
    let reply_bytes = client
        .call(
            "svc.int_inc",
            &Int32 { data: 41 }.encode_to_vec(),
            None,
            Some(Duration::from_secs(10)),
        )
        .expect("call");
    let reply = Int32::decode(reply_bytes.as_slice()).expect("decode reply");
    assert_eq!(reply.data, 42);

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn service_bus_std_srvs_trigger() {
    let _guard = lock_brokers();
    let broker = ServiceBusBroker::start(ServiceBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ServiceBusConfig::default()
    })
    .expect("start service bus");

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| {
            let _req = TriggerRequest::decode(body).expect("decode TriggerRequest");
            TriggerResponse {
                success: true,
                message: "triggered".into(),
            }
            .encode_to_vec()
        });
    let worker = WorkerThread::spawn_service("svc.trigger", handler, &broker.backend_bind)
        .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let client = ServiceClient::new(Some(&broker.frontend_bind)).expect("client");
    let reply_bytes = client
        .call(
            "svc.trigger",
            &TriggerRequest {}.encode_to_vec(),
            None,
            Some(Duration::from_secs(10)),
        )
        .expect("call");
    let reply = TriggerResponse::decode(reply_bytes.as_slice()).expect("decode reply");
    assert!(reply.success);
    assert_eq!(reply.message, "triggered");

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn service_bus_std_srvs_set_bool() {
    let _guard = lock_brokers();
    let broker = ServiceBusBroker::start(ServiceBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ServiceBusConfig::default()
    })
    .expect("start service bus");

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| {
            let req = SetBoolRequest::decode(body).expect("decode SetBoolRequest");
            SetBoolResponse {
                success: true,
                message: format!("set:{}", req.data),
            }
            .encode_to_vec()
        });
    let worker = WorkerThread::spawn_service("svc.set_bool", handler, &broker.backend_bind)
        .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let client = ServiceClient::new(Some(&broker.frontend_bind)).expect("client");
    let reply_bytes = client
        .call(
            "svc.set_bool",
            &SetBoolRequest { data: true }.encode_to_vec(),
            None,
            Some(Duration::from_secs(10)),
        )
        .expect("call");
    let reply = SetBoolResponse::decode(reply_bytes.as_slice()).expect("decode reply");
    assert!(reply.success);
    assert_eq!(reply.message, "set:true");

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn service_bus_nav_msgs_get_map() {
    let _guard = lock_brokers();
    let broker = ServiceBusBroker::start(ServiceBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ServiceBusConfig::default()
    })
    .expect("start service bus");

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| {
            let _req = GetMapRequest::decode(body).expect("decode GetMapRequest");
            GetMapResponse {
                map: Some(OccupancyGrid {
                    header: Some(Header {
                        stamp: None,
                        frame_id: "map".into(),
                    }),
                    info: Some(MapMetaData {
                        map_load_time: None,
                        resolution: 0.05,
                        width: 2,
                        height: 2,
                        origin: None,
                    }),
                    data: vec![0, 100, -1, 50],
                }),
            }
            .encode_to_vec()
        });
    let worker = WorkerThread::spawn_service("svc.get_map", handler, &broker.backend_bind)
        .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let client = ServiceClient::new(Some(&broker.frontend_bind)).expect("client");
    let reply_bytes = client
        .call(
            "svc.get_map",
            &GetMapRequest {}.encode_to_vec(),
            None,
            Some(Duration::from_secs(10)),
        )
        .expect("call");
    let reply = GetMapResponse::decode(reply_bytes.as_slice()).expect("decode reply");
    let map = reply.map.expect("map");
    assert_eq!(map.header.as_ref().unwrap().frame_id, "map");
    assert_eq!(map.info.as_ref().unwrap().width, 2);
    assert_eq!(map.data, vec![0, 100, -1, 50]);

    worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn service_bus_nav_msgs_set_map_and_get_plan() {
    let _guard = lock_brokers();
    let broker = ServiceBusBroker::start(ServiceBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ServiceBusConfig::default()
    })
    .expect("start service bus");

    let set_map_handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| {
            let req = SetMapRequest::decode(body).expect("decode SetMapRequest");
            assert!(req.map.is_some());
            assert!(req.initial_pose.is_some());
            SetMapResponse { success: true }.encode_to_vec()
        });
    let get_plan_handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<u8> + Send + Sync> =
        Arc::new(|_client_id, _req_id, body| {
            let req = GetPlanRequest::decode(body).expect("decode GetPlanRequest");
            let start = req.start.expect("start");
            let goal = req.goal.expect("goal");
            GetPlanResponse {
                plan: Some(Path {
                    header: Some(Header {
                        stamp: None,
                        frame_id: "map".into(),
                    }),
                    poses: vec![start, goal],
                }),
            }
            .encode_to_vec()
        });

    let set_map_worker =
        WorkerThread::spawn_service("svc.set_map", set_map_handler, &broker.backend_bind)
            .expect("set_map worker");
    let get_plan_worker =
        WorkerThread::spawn_service("svc.get_plan", get_plan_handler, &broker.backend_bind)
            .expect("get_plan worker");
    thread::sleep(Duration::from_millis(100));

    let client = ServiceClient::new(Some(&broker.frontend_bind)).expect("client");

    let set_map_reply = client
        .call(
            "svc.set_map",
            &SetMapRequest {
                map: Some(OccupancyGrid {
                    header: Some(Header {
                        stamp: None,
                        frame_id: "map".into(),
                    }),
                    info: Some(MapMetaData {
                        map_load_time: None,
                        resolution: 0.1,
                        width: 1,
                        height: 1,
                        origin: None,
                    }),
                    data: vec![0],
                }),
                initial_pose: Some(PoseWithCovarianceStamped {
                    header: Some(Header {
                        stamp: None,
                        frame_id: "map".into(),
                    }),
                    pose: Some(PoseWithCovariance {
                        pose: Some(Pose {
                            position: Some(robot_bus::msgs::geometry_msgs::v1::Point {
                                x: 0.0,
                                y: 0.0,
                                z: 0.0,
                            }),
                            orientation: None,
                        }),
                        covariance: vec![],
                    }),
                }),
            }
            .encode_to_vec(),
            None,
            Some(Duration::from_secs(10)),
        )
        .expect("set_map");
    let set_map_resp = SetMapResponse::decode(set_map_reply.as_slice()).expect("decode set_map");
    assert!(set_map_resp.success);

    let start = PoseStamped {
        header: Some(Header {
            stamp: None,
            frame_id: "map".into(),
        }),
        pose: Some(Pose {
            position: Some(robot_bus::msgs::geometry_msgs::v1::Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
            orientation: None,
        }),
    };
    let goal = PoseStamped {
        header: Some(Header {
            stamp: None,
            frame_id: "map".into(),
        }),
        pose: Some(Pose {
            position: Some(robot_bus::msgs::geometry_msgs::v1::Point {
                x: 1.0,
                y: 2.0,
                z: 0.0,
            }),
            orientation: None,
        }),
    };
    let plan_reply = client
        .call(
            "svc.get_plan",
            &GetPlanRequest {
                start: Some(start),
                goal: Some(goal),
                tolerance: 0.25,
            }
            .encode_to_vec(),
            None,
            Some(Duration::from_secs(10)),
        )
        .expect("get_plan");
    let plan_resp = GetPlanResponse::decode(plan_reply.as_slice()).expect("decode get_plan");
    let plan = plan_resp.plan.expect("plan");
    assert_eq!(plan.poses.len(), 2);
    assert_eq!(
        plan.poses[1].pose.as_ref().unwrap().position.as_ref().unwrap().x,
        1.0
    );

    set_map_worker.stop();
    get_plan_worker.stop();
    broker.stop().expect("stop");
}

#[test]
fn action_bus_pose2d_goal_proto() {
    let _guard = lock_brokers();
    let broker = ActionBusBroker::start(ActionBusConfig {
        frontend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        backend_bind: format!("tcp://127.0.0.1:{}", free_port()),
        heartbeat_interval_ms: 200,
        heartbeat_timeout_ms: 600,
        ..ActionBusConfig::default()
    })
    .expect("start action bus");

    let handler: Arc<dyn Fn(&[u8], &[u8], &[u8]) -> Vec<(String, Vec<u8>)> + Send + Sync> =
        Arc::new(|_client_id, _goal_id, body| {
            let goal = Pose2D::decode(body).expect("decode Pose2D goal");
            let progress = Int32 { data: 50 }.encode_to_vec();
            let result = Pose2D {
                x: goal.x,
                y: goal.y,
                theta: goal.theta + 0.1,
            }
            .encode_to_vec();
            vec![
                ("FEEDBACK".into(), progress),
                ("RESULT".into(), result),
            ]
        });
    let worker = WorkerThread::spawn_action("act.goto", handler, &broker.backend_bind)
        .expect("worker");
    thread::sleep(Duration::from_millis(100));

    let client = ActionClient::new(Some(&broker.frontend_bind)).expect("client");
    let goal = Pose2D {
        x: 1.0,
        y: 2.0,
        theta: 0.5,
    };
    let messages = client
        .send_goal(
            "act.goto",
            &goal.encode_to_vec(),
            None,
            Some(Duration::from_secs(30)),
        )
        .expect("goal");

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].kind, ActionKind::Feedback);
    let progress = Int32::decode(messages[0].body.as_slice()).expect("decode feedback");
    assert_eq!(progress.data, 50);

    assert_eq!(messages[1].kind, ActionKind::Result);
    let result = Pose2D::decode(messages[1].body.as_slice()).expect("decode result");
    assert_eq!(result.x, 1.0);
    assert_eq!(result.y, 2.0);
    assert!((result.theta - 0.6).abs() < 1e-9);

    worker.stop();
    broker.stop().expect("stop");
}
