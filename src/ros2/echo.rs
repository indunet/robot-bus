//! Drop short-lived echoes when bridging the same payload both ways.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Remember recently forwarded payloads; drop matching callbacks for `ttl`.
#[derive(Debug, Default)]
pub struct EchoFilter {
    ttl: Duration,
    recent: VecDeque<(Instant, Vec<u8>)>,
}

impl EchoFilter {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            recent: VecDeque::new(),
        }
    }

    fn purge(&mut self, now: Instant) {
        while self
            .recent
            .front()
            .is_some_and(|(deadline, _)| *deadline <= now)
        {
            self.recent.pop_front();
        }
    }

    pub fn remember(&mut self, payload: impl AsRef<[u8]>) {
        let now = Instant::now();
        self.purge(now);
        self.recent
            .push_back((now + self.ttl, payload.as_ref().to_vec()));
    }

    pub fn is_echo(&mut self, payload: impl AsRef<[u8]>) -> bool {
        let now = Instant::now();
        self.purge(now);
        let payload = payload.as_ref();
        self.recent.iter().any(|(_, p)| p.as_slice() == payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remembers_and_matches() {
        let mut f = EchoFilter::new(Duration::from_secs(1));
        assert!(!f.is_echo(b"abc"));
        f.remember(b"abc");
        assert!(f.is_echo(b"abc"));
        assert!(!f.is_echo(b"other"));
    }
}
