//! Axum WHEP server: POST /whep, DELETE /whep/:id, GET / demo page.

use anyhow::{Context, Result};
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::Router;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

use super::hub::{MediaHub, PeerSession};

const DEMO_HTML: &str = include_str!("demo.html");

#[derive(Clone)]
struct AppState {
    hub: MediaHub,
}

/// Spawn a Tokio runtime thread that serves WHEP until the process exits.
pub fn spawn_whep_server(hub: MediaHub, listen: SocketAddr) -> Result<std::thread::JoinHandle<()>> {
    let handle = std::thread::Builder::new()
        .name("rbus-webrtc-whep".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("rbus-webrtc")
                .build()
                .expect("tokio runtime for webrtc");
            rt.block_on(async move {
                if let Err(e) = serve(hub, listen).await {
                    log::error!("WHEP server stopped: {e:#}");
                }
            });
        })
        .context("spawn WHEP server thread")?;
    Ok(handle)
}

async fn serve(hub: MediaHub, listen: SocketAddr) -> Result<()> {
    tokio::spawn(video_fanout(hub.clone()));
    tokio::spawn(audio_fanout(hub.clone()));
    tokio::spawn(data_fanout(hub.clone()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([header::LOCATION]);

    let state = AppState { hub };
    let app = Router::new()
        .route("/", get(demo_page))
        .route("/whep", post(whep_create))
        .route("/whep/{id}", delete(whep_delete))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(listen)
        .await
        .with_context(|| format!("bind WHEP on {listen}"))?;
    log::info!("WHEP listening on http://{listen}/whep (demo http://{listen}/)");

    axum::serve(listener, app)
        .await
        .context("WHEP HTTP server")?;
    Ok(())
}

async fn demo_page() -> Html<&'static str> {
    Html(DEMO_HTML)
}

async fn whep_create(State(state): State<AppState>, body: String) -> Response {
    match create_session(&state.hub, &body).await {
        Ok((id, answer_sdp)) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/sdp"),
            );
            if let Ok(loc) = HeaderValue::from_str(&format!("/whep/{id}")) {
                headers.insert(header::LOCATION, loc);
            }
            (StatusCode::CREATED, headers, answer_sdp).into_response()
        }
        Err(e) => {
            log::warn!("WHEP create failed: {e:#}");
            (
                StatusCode::BAD_REQUEST,
                format!("WHEP create failed: {e:#}"),
            )
                .into_response()
        }
    }
}

async fn whep_delete(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    let session = {
        let mut sessions = state.hub.sessions.write().await;
        sessions.remove(&id)
    };
    if let Some(session) = session {
        let _ = session.pc.close().await;
        log::info!("WHEP session {id} closed");
        StatusCode::NO_CONTENT.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn create_session(hub: &MediaHub, offer_sdp: &str) -> Result<(String, String)> {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine)?;
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".into()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let pc = Arc::new(api.new_peer_connection(config).await?);
    let id = uuid::Uuid::new_v4().to_string();
    let sessions = Arc::clone(&hub.sessions);

    {
        let id_cb = id.clone();
        pc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            let id_cb = id_cb.clone();
            let sessions = Arc::clone(&sessions);
            Box::pin(async move {
                log::info!("WHEP session {id_cb} state: {s}");
                if matches!(
                    s,
                    RTCPeerConnectionState::Failed
                        | RTCPeerConnectionState::Closed
                        | RTCPeerConnectionState::Disconnected
                ) {
                    let mut map = sessions.write().await;
                    if let Some(sess) = map.remove(&id_cb) {
                        let _ = sess.pc.close().await;
                    }
                }
            })
        }));
    }

    let video = if hub.enable_video {
        let track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f"
                        .to_owned(),
                rtcp_feedback: vec![],
            },
            "video".to_owned(),
            "robot-bus".to_owned(),
        ));
        pc.add_track(Arc::clone(&track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;
        Some(track)
    } else {
        None
    };

    let audio = if hub.enable_audio {
        let track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                clock_rate: 48000,
                channels: 2,
                sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
                rtcp_feedback: vec![],
            },
            "audio".to_owned(),
            "robot-bus".to_owned(),
        ));
        pc.add_track(Arc::clone(&track) as Arc<dyn TrackLocal + Send + Sync>)
            .await?;
        Some(track)
    } else {
        None
    };

    let mut data_channels = HashMap::new();
    for topic in &hub.data_topics {
        let dc = pc
            .create_data_channel(
                topic,
                Some(RTCDataChannelInit {
                    ordered: Some(true),
                    ..Default::default()
                }),
            )
            .await?;
        data_channels.insert(topic.clone(), dc);
    }

    let offer = RTCSessionDescription::offer(offer_sdp.to_string())?;
    pc.set_remote_description(offer).await?;
    let answer = pc.create_answer(None).await?;
    let mut gather_complete = pc.gathering_complete_promise().await;
    pc.set_local_description(answer).await?;
    let _ = gather_complete.recv().await;

    let local = pc
        .local_description()
        .await
        .context("missing local description")?;
    let answer_sdp = local.sdp;

    {
        let mut map = hub.sessions.write().await;
        map.insert(
            id.clone(),
            PeerSession {
                pc: Arc::clone(&pc),
                video,
                audio,
                data_channels,
            },
        );
    }

    log::info!("WHEP session {id} created");
    Ok((id, answer_sdp))
}

async fn video_fanout(hub: MediaHub) {
    let mut rx = hub.video_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(sample) => {
                let sessions = hub.sessions.read().await;
                for (id, sess) in sessions.iter() {
                    if let Some(track) = &sess.video {
                        if let Err(e) = track
                            .write_sample(&Sample {
                                data: sample.data.clone(),
                                duration: sample.duration,
                                ..Default::default()
                            })
                            .await
                        {
                            log::debug!("video write to {id}: {e}");
                        }
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                log::warn!("video broadcast lagged by {n}");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

async fn audio_fanout(hub: MediaHub) {
    let mut rx = hub.audio_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(sample) => {
                let sessions = hub.sessions.read().await;
                for (id, sess) in sessions.iter() {
                    if let Some(track) = &sess.audio {
                        if let Err(e) = track
                            .write_sample(&Sample {
                                data: sample.data.clone(),
                                duration: sample.duration,
                                ..Default::default()
                            })
                            .await
                        {
                            log::debug!("audio write to {id}: {e}");
                        }
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                log::warn!("audio broadcast lagged by {n}");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

async fn data_fanout(hub: MediaHub) {
    let mut rx = hub.data_tx.subscribe();
    loop {
        match rx.recv().await {
            Ok(payload) => {
                let sessions = hub.sessions.read().await;
                for (id, sess) in sessions.iter() {
                    if let Some(dc) = sess.data_channels.get(&payload.topic) {
                        if let Err(e) = dc.send(&payload.data).await {
                            log::debug!("datachannel {} to {id}: {e}", payload.topic);
                        }
                    }
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                log::warn!("data broadcast lagged by {n}");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}
