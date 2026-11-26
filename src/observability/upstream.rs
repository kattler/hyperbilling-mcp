use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

const HISTORY: usize = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Check {
    /// Unix time in milliseconds.
    pub at: u64,
    pub reachable: bool,
    /// Absent when no response arrived.
    pub status: Option<u16>,
    pub latency_ms: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct Snapshot {
    pub ready_since: Option<u64>,
    pub checks: Vec<Check>,
}

#[derive(Default)]
struct State {
    ready_since: Option<u64>,
    checks: VecDeque<Check>,
}

/// The recent reachability checks of the HyperBilling API, for the status board.
#[derive(Default)]
pub struct UpstreamHealth(Mutex<State>);

impl UpstreamHealth {
    pub fn record(&self, reachable: bool, status: Option<u16>, latency: Duration) {
        self.record_at(now_ms(), reachable, status, latency);
    }

    fn record_at(&self, at: u64, reachable: bool, status: Option<u16>, latency: Duration) {
        let mut state = self.state();
        if reachable && state.ready_since.is_none() {
            state.ready_since = Some(at);
        }
        if state.checks.len() == HISTORY {
            state.checks.pop_front();
        }
        state.checks.push_back(Check {
            at,
            reachable,
            status,
            latency_ms: millis(latency),
        });
    }

    pub fn snapshot(&self) -> Snapshot {
        let state = self.state();
        Snapshot {
            ready_since: state.ready_since,
            checks: state.checks.iter().copied().collect(),
        }
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_first_ready_time_and_the_latest_checks() {
        let health = UpstreamHealth::default();
        assert!(health.snapshot().ready_since.is_none());

        health.record_at(10, false, None, Duration::from_millis(5000));
        assert!(health.snapshot().ready_since.is_none());
        health.record_at(20, true, Some(404), Duration::from_millis(120));
        health.record_at(30, false, Some(502), Duration::from_millis(80));
        let snapshot = health.snapshot();
        assert_eq!(snapshot.ready_since, Some(20));
        assert_eq!(
            snapshot.checks[1],
            Check {
                at: 20,
                reachable: true,
                status: Some(404),
                latency_ms: 120
            }
        );
        assert_eq!(snapshot.checks[2].status, Some(502));

        for at in 0..100 {
            health.record_at(at, true, Some(404), Duration::ZERO);
        }
        let snapshot = health.snapshot();
        assert_eq!(snapshot.checks.len(), HISTORY);
        assert_eq!(snapshot.checks.last().unwrap().at, 99);
        assert_eq!(snapshot.ready_since, Some(20));
    }

    #[test]
    fn serializes_for_the_status_board() {
        let health = UpstreamHealth::default();
        health.record_at(20, false, None, Duration::from_millis(7));
        assert_eq!(
            serde_json::to_value(health.snapshot()).unwrap(),
            serde_json::json!({
                "ready_since": null,
                "checks": [{"at": 20, "reachable": false, "status": null, "latency_ms": 7}]
            })
        );
    }
}
