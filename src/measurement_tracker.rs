use std::time::{Duration, Instant};

/// Represents a started measurement. When dropped, it will log the
/// duration into memory.
pub struct MeasurementTracker {
    pub(crate) start_time: Instant,
    pub(crate) overhead: Duration,
}
