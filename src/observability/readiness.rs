use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A latch: not ready until the first upstream check passes, then ready for the life of the process.
#[derive(Clone, Default)]
pub struct Readiness(Arc<AtomicBool>);

impl Readiness {
    pub fn is_ready(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// Returns whether this call made the transition.
    pub fn mark_ready(&self) -> bool {
        !self.0.swap(true, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_not_ready_and_latches() {
        let readiness = Readiness::default();
        let shared = readiness.clone();
        assert!(!readiness.is_ready());
        assert!(shared.mark_ready());
        assert!(!readiness.mark_ready());
        assert!(readiness.is_ready());
    }
}
