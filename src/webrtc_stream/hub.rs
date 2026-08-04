//! Fan-out of encoded media and data payloads to WHEP peer sessions.

use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;

/// One Annex-B access unit (or Opus packet) ready for `write_sample`.
#[derive(Debug, Clone)]
pub struct MediaSample {
    pub data: Bytes,
    pub duration: Duration,
}

/// Raw bus payload for a named DataChannel.
#[derive(Debug, Clone)]
pub struct DataPayload {
    pub topic: String,
    pub data: Bytes,
}

pub struct PeerSession {
    pub pc: Arc<RTCPeerConnection>,
    pub video: Option<Arc<TrackLocalStaticSample>>,
    pub audio: Option<Arc<TrackLocalStaticSample>>,
    /// DataChannel label → channel (label == topic name).
    pub data_channels: HashMap<String, Arc<RTCDataChannel>>,
}

/// Shared state between the robot-bus node thread and the Tokio WHEP server.
#[derive(Clone)]
pub struct MediaHub {
    pub video_tx: broadcast::Sender<MediaSample>,
    pub audio_tx: broadcast::Sender<MediaSample>,
    pub data_tx: broadcast::Sender<DataPayload>,
    pub sessions: Arc<RwLock<HashMap<String, PeerSession>>>,
    pub enable_video: bool,
    pub enable_audio: bool,
    pub data_topics: Vec<String>,
    pub fps: u32,
}

impl MediaHub {
    pub fn new(
        enable_video: bool,
        enable_audio: bool,
        data_topics: Vec<String>,
        fps: u32,
    ) -> Self {
        let (video_tx, _) = broadcast::channel(64);
        let (audio_tx, _) = broadcast::channel(64);
        let (data_tx, _) = broadcast::channel(128);
        Self {
            video_tx,
            audio_tx,
            data_tx,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            enable_video,
            enable_audio,
            data_topics,
            fps,
        }
    }

    pub fn publish_video(&self, data: Vec<u8>, duration: Duration) {
        let _ = self.video_tx.send(MediaSample {
            data: Bytes::from(data),
            duration,
        });
    }

    pub fn publish_audio(&self, data: Vec<u8>, duration: Duration) {
        let _ = self.audio_tx.send(MediaSample {
            data: Bytes::from(data),
            duration,
        });
    }

    pub fn publish_data(&self, topic: &str, data: Vec<u8>) {
        let _ = self.data_tx.send(DataPayload {
            topic: topic.to_string(),
            data: Bytes::from(data),
        });
    }
}
