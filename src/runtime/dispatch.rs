//! Frame parsing and inline callback dispatch for [`super::Executor`].

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use crate::action_bus::{ActionKind, ActionMessage};
use crate::runtime::callback_group::SubscriptionCallback;
use crate::runtime::queues::{ActionReply, OutboundCommand, ReplyMessage, ServiceReply};
use crate::runtime::registrations::{
    ActionClientRegistration, ActionGoalContext, ActionRegistration, Registration,
    RegistrationKind, ServiceRegistration, SubRegistration,
};
use crate::runtime::topic_callbacks::for_each_matching_callback;
use crate::runtime::worker_pool::WorkerPool;

pub fn dispatch_registration(
    reg: &mut Registration,
    topic_callbacks: &HashMap<String, Vec<SubscriptionCallback>>,
    reply_tx: &Sender<ReplyMessage>,
    worker_pool: Option<&WorkerPool>,
) {
    match reg.kind() {
        RegistrationKind::Sub => {
            if let Registration::Sub(sub) = reg {
                dispatch_sub_message(sub, topic_callbacks, worker_pool);
            }
        }
        RegistrationKind::Service => {
            if let Registration::Service(service) = reg {
                dispatch_service_request(service, reply_tx, worker_pool);
            }
        }
        RegistrationKind::Action => {
            if let Registration::Action(action) = reg {
                dispatch_action_message(action, reply_tx, worker_pool);
            }
        }
        RegistrationKind::ActionClient => {
            if let Registration::ActionClient(client) = reg {
                dispatch_action_client_message(client);
            }
        }
    }
}

pub fn dispatch_sub_message(
    reg: &SubRegistration,
    topic_callbacks: &HashMap<String, Vec<SubscriptionCallback>>,
    worker_pool: Option<&WorkerPool>,
) {
    let mut frames = match reg.socket.recv_multipart(0) {
        Ok(frames) => frames,
        Err(err) => {
            log::warn!("sub recv failed on {}: {err}", reg.endpoint);
            return;
        }
    };
    if frames.len() < 2 {
        log::warn!(
            "ignored sub frame with count {} on {}",
            frames.len(),
            reg.endpoint
        );
        return;
    }
    let payload_bytes = std::mem::take(&mut frames[1]);
    let topic = String::from_utf8_lossy(&frames[0]);
    let payload: Arc<[u8]> = payload_bytes.into();
    for_each_matching_callback(&topic, topic_callbacks, |entry| {
        let callback = Arc::clone(&entry.callback);
        let group = entry.group.clone();
        let payload = Arc::clone(&payload);
        group.run(worker_pool, move || callback(&payload));
    });
}

pub fn dispatch_service_request(
    reg: &ServiceRegistration,
    reply_tx: &Sender<ReplyMessage>,
    worker_pool: Option<&WorkerPool>,
) {
    let frames = match reg.socket.recv_multipart(0) {
        Ok(frames) => frames,
        Err(err) => {
            log::warn!("service recv failed for {}: {err}", reg.service_name);
            return;
        }
    };
    let Ok([client_id, svc, req_id, body]) = <[Vec<u8>; 4]>::try_from(frames) else {
        log::warn!("ignored service frame with unexpected count");
        return;
    };
    if String::from_utf8_lossy(&svc) != reg.service_name {
        log::warn!(
            "ignored request for service {:?}",
            String::from_utf8_lossy(&svc)
        );
        return;
    }

    let handler = Arc::clone(&reg.handler);
    let service_name = reg.service_name.clone();
    let reply_tx = reply_tx.clone();
    let group = reg.callback_group.clone();
    group.run(worker_pool, move || {
        let reply_body = handler(&body);
        let _ = reply_tx.send(ReplyMessage::Service {
            service_name,
            reply: ServiceReply {
                client_id,
                service: svc,
                request_id: req_id,
                body: reply_body,
            },
        });
    });
}

pub fn dispatch_action_message(
    reg: &ActionRegistration,
    reply_tx: &Sender<ReplyMessage>,
    worker_pool: Option<&WorkerPool>,
) {
    let frames = match reg.socket.recv_multipart(0) {
        Ok(frames) => frames,
        Err(err) => {
            log::warn!("action recv failed for {}: {err}", reg.action_name);
            return;
        }
    };
    let Ok([client_id, action_bytes, goal_id, kind, body]) = <[Vec<u8>; 5]>::try_from(frames)
    else {
        log::warn!("ignored action frame with unexpected count");
        return;
    };
    let action = String::from_utf8_lossy(&action_bytes);
    if action != reg.action_name {
        log::warn!("ignored message for action {action:?}");
        return;
    }
    let kind_str = String::from_utf8_lossy(&kind);
    let goal_id_str = String::from_utf8_lossy(&goal_id).into_owned();
    if kind_str == "CANCEL" {
        if let Ok(map) = reg.inflight.lock() {
            if let Some(flag) = map.get(&goal_id_str) {
                flag.store(true, Ordering::SeqCst);
            }
        }
        return;
    }
    if kind_str != "GOAL" {
        log::warn!("ignored action kind {kind_str:?}");
        return;
    }

    let cancel = Arc::new(AtomicBool::new(false));
    if let Ok(mut map) = reg.inflight.lock() {
        map.insert(goal_id_str.clone(), Arc::clone(&cancel));
    }

    let handler = Arc::clone(&reg.handler);
    let inflight = Arc::clone(&reg.inflight);
    let action_name = reg.action_name.clone();
    let reply_tx = reply_tx.clone();
    let group = reg.callback_group.clone();

    let client_id_fb = client_id.clone();
    let goal_id_fb = goal_id.clone();
    let action_name_fb = action_name.clone();
    let reply_tx_fb = reply_tx.clone();
    let ctx = ActionGoalContext::new(
        goal_id_str.clone(),
        cancel,
        Arc::new(move |chunk: &[u8]| {
            let _ = reply_tx_fb.send(ReplyMessage::Action {
                action_name: action_name_fb.clone(),
                reply: ActionReply {
                    client_id: client_id_fb.clone(),
                    goal_id: goal_id_fb.clone(),
                    kind: b"FEEDBACK".to_vec(),
                    body: chunk.to_vec(),
                },
            });
        }),
    );

    let job = move || {
        let result = handler(&body, &ctx);
        let _ = reply_tx.send(ReplyMessage::Action {
            action_name,
            reply: ActionReply {
                client_id,
                goal_id,
                kind: b"RESULT".to_vec(),
                body: result,
            },
        });
        if let Ok(mut map) = inflight.lock() {
            map.remove(&goal_id_str);
        }
    };

    // Live FEEDBACK/CANCEL need the poll thread free. Offload even without a pool.
    if worker_pool.is_some() {
        group.run(worker_pool, job);
    } else {
        thread::spawn(job);
    }
}

pub fn dispatch_action_client_message(reg: &mut ActionClientRegistration) {
    let frames = match reg.socket.recv_multipart(0) {
        Ok(frames) => frames,
        Err(err) => {
            log::warn!("action client recv failed on {}: {err}", reg.endpoint);
            return;
        }
    };
    let Ok([action_bytes, goal_bytes, kind_bytes, body]) = <[Vec<u8>; 4]>::try_from(frames) else {
        log::warn!("ignored action client frame with unexpected count");
        return;
    };
    let action_name = String::from_utf8_lossy(&action_bytes).into_owned();
    let goal_id = String::from_utf8_lossy(&goal_bytes).into_owned();
    let kind = match ActionKind::from_wire(&String::from_utf8_lossy(&kind_bytes)) {
        Ok(kind) => kind,
        Err(err) => {
            log::warn!("ignored action client kind: {err}");
            return;
        }
    };
    let message = ActionMessage {
        action_name,
        goal_id: goal_id.clone(),
        kind,
        body,
    };
    if let Some(callback) = reg.goal_callbacks.get(&goal_id) {
        callback(&message);
    } else {
        log::warn!("no callback registered for goal {goal_id:?}");
    }
    if message.kind == ActionKind::Result {
        reg.goal_callbacks.remove(&goal_id);
    }
}

pub fn flush_reply_queue(
    registrations: &mut [Registration],
    reply_rx: &std::sync::mpsc::Receiver<ReplyMessage>,
) {
    while let Ok(item) = reply_rx.try_recv() {
        match item {
            ReplyMessage::Service {
                service_name,
                reply,
            } => {
                if let Some(reg) = registrations.iter().find_map(|reg| {
                    if let Registration::Service(s) = reg {
                        if s.service_name == service_name {
                            Some(s)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }) {
                    let _ = send_service_reply(
                        &reg.socket,
                        &reply.client_id,
                        &reply.service,
                        &reply.request_id,
                        &reply.body,
                    );
                }
            }
            ReplyMessage::Action { action_name, reply } => {
                if let Some(reg) = registrations.iter().find_map(|reg| {
                    if let Registration::Action(a) = reg {
                        if a.action_name == action_name {
                            Some(a)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }) {
                    let _ = send_action_reply(&reg.socket, &reg.action_name, &reply);
                }
            }
        }
    }
}

pub fn tick_heartbeats(workers: &mut [Registration], now: std::time::Instant) {
    for reg in workers.iter_mut() {
        match reg {
            Registration::Service(worker) => {
                if now.duration_since(worker.last_heartbeat) >= worker.heartbeat_interval {
                    if let Err(err) = worker.send_heartbeat() {
                        log::error!("heartbeat failed: {err}");
                    }
                    worker.last_heartbeat = now;
                }
            }
            Registration::Action(worker) => {
                if now.duration_since(worker.last_heartbeat) >= worker.heartbeat_interval {
                    if let Err(err) = worker.send_heartbeat() {
                        log::error!("heartbeat failed: {err}");
                    }
                    worker.last_heartbeat = now;
                }
            }
            _ => {}
        }
    }
}

pub fn flush_outbound(
    reg: &mut ActionClientRegistration,
    outbound_rx: &std::sync::mpsc::Receiver<OutboundCommand>,
) {
    while let Ok(cmd) = outbound_rx.try_recv() {
        match cmd {
            OutboundCommand::SendGoal {
                action_name,
                goal_id,
                body,
                callback,
            } => {
                reg.goal_callbacks.insert(goal_id.clone(), callback);
                if let Err(err) = reg.socket.send_multipart(
                    [
                        action_name.as_bytes(),
                        goal_id.as_bytes(),
                        b"GOAL",
                        body.as_slice(),
                    ],
                    0,
                ) {
                    log::warn!("action goal send failed: {err}");
                    reg.goal_callbacks.remove(&goal_id);
                }
            }
            OutboundCommand::CancelGoal {
                action_name,
                goal_id,
                body,
            } => {
                if let Err(err) = reg.socket.send_multipart(
                    [
                        action_name.as_bytes(),
                        goal_id.as_bytes(),
                        b"CANCEL",
                        body.as_slice(),
                    ],
                    0,
                ) {
                    log::warn!("action cancel send failed: {err}");
                }
            }
        }
    }
}

fn send_service_reply(
    socket: &zmq::Socket,
    client_id: &[u8],
    svc: &[u8],
    req_id: &[u8],
    body: &[u8],
) -> Result<(), zmq::Error> {
    socket.send_multipart([client_id, svc, req_id, body], 0)
}

fn send_action_reply(
    socket: &zmq::Socket,
    action_name: &str,
    reply: &ActionReply,
) -> Result<(), zmq::Error> {
    socket.send_multipart(
        [
            reply.client_id.as_slice(),
            action_name.as_bytes(),
            reply.goal_id.as_slice(),
            reply.kind.as_slice(),
            reply.body.as_slice(),
        ],
        0,
    )
}
