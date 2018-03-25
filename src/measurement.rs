//! The backend for the measurements.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use measurement_tracker::MeasurementTracker;

lazy_static! {
    pub(crate) static ref MEASUREMENT_STACK: Mutex<Vec<Arc<Mutex<Measurement>>>> =
        Mutex::new(vec![Measurement::new("root".to_string(), 0, None)]);
}

/// Starts a measurement in the current scope. **Don't use this, use
/// the [`perf_measure!`](macro.perf_measure.html) macro.**
pub fn measure<T: Into<String>>(now: Instant, measurement_name: T) -> MeasurementTracker {
    let name = measurement_name.into();
    let mut stack = MEASUREMENT_STACK.lock().unwrap();
    let depth = stack.len();

    let parent = stack.get(depth - 1).unwrap().clone();
    let measurement = Measurement::new(name.clone(), depth, parent.clone());
    let mut parent = parent.lock().unwrap();
    if let Some(measurement) = parent.get_child(&name) {
        stack.push(measurement.clone());
    } else {
        stack.push(measurement.clone());
        parent.children.push(measurement);
        parent.children_names.push(name);
    }

    MeasurementTracker {
        start_time: now,
        overhead: Instant::now() - now,
    }
}

impl Drop for MeasurementTracker {
    fn drop(&mut self) {
        let latter_overhead_start = Instant::now();
        let mut stack = MEASUREMENT_STACK.lock().unwrap();
        let measurement = stack.pop().unwrap();
        let mut measurement = measurement.lock().unwrap();
        measurement.overhead += self.overhead;
        measurement.durations.push(Instant::now() - self.start_time);
        measurement.overhead += Instant::now() - latter_overhead_start;
    }
}

/// Resets the measurement data.
///
/// Warning: This will wipe all measurements from the memory!
pub fn reset() {
    let mut stack = MEASUREMENT_STACK.lock().unwrap();
    stack.split_off(1);
    let root = stack.get(0).unwrap();
    let mut root = root.lock().unwrap();
    root.children.clear();
    root.children_names.clear();
}

/// Returns a `Vec` of all the
/// [`Measurement`](struct.Measurement.html)s taken so far.
///
/// **Warning**: This function is pretty heavy, especially as the
/// amount of samples rises, as it clones every one of them.
pub(crate) fn get_measures() -> Vec<Measurement> {
    let stack = MEASUREMENT_STACK.lock().unwrap();
    let root = stack.get(0).unwrap().lock().unwrap();
    root.collect_all_children()
}

/// Represents a scope's running time.
#[derive(Clone)]
pub(crate) struct Measurement {
    pub(crate) name: String,
    pub(crate) depth: usize,
    pub(crate) overhead: Duration,
    pub(crate) durations: Vec<Duration>,
    pub(crate) parent: Option<Arc<Mutex<Measurement>>>,
    children: Vec<Arc<Mutex<Measurement>>>,
    children_names: Vec<String>,
}

impl Measurement {
    fn new<T: Into<Option<Arc<Mutex<Measurement>>>>>(
        name: String,
        depth: usize,
        parent: T,
    ) -> Arc<Mutex<Measurement>> {
        Arc::new(Mutex::new(Measurement {
            name,
            depth,
            overhead: Duration::new(0, 0),
            durations: Vec::new(),
            parent: parent.into(),
            children: Vec::new(),
            children_names: Vec::new(),
        }))
    }

    pub(crate) fn get_ancestor(&self, generation: u32) -> Option<Arc<Mutex<Measurement>>> {
        if generation == 0 {
            if let Some(ref parent) = self.parent {
                Some(parent.clone())
            } else {
                None
            }
        } else {
            if let Some(ref parent) = self.parent {
                let parent = parent.lock().unwrap();
                parent.get_ancestor(generation - 1)
            } else {
                None
            }
        }
    }

    pub(crate) fn has_children(&self) -> bool {
        self.children.len() > 0
    }

    /// Is `name` the last child of `self`?
    pub(crate) fn is_last_child_name(&self, name: &str, leaf: bool) -> bool {
        if let Some(last_leaf_name) = self.last_child_name(leaf) {
            last_leaf_name == name
        } else {
            false
        }
    }

    fn last_child_name(&self, leaf: bool) -> Option<String> {
        if leaf && self.children.len() == 0 {
            None
        } else {
            Some(self.children_names[self.children.len() - 1].clone())
        }
    }

    pub(crate) fn collect_all_children(&self) -> Vec<Measurement> {
        let mut collection = Vec::new();
        collection.push(self.clone());
        for child in &self.children {
            collection.append(&mut child.lock().unwrap().collect_all_children());
        }
        collection
    }

    pub(crate) fn get_duration_ns(&self) -> Option<u64> {
        let count = self.durations.len();
        if count == 0 {
            None
        } else {
            let mut total: u64 = 0;
            for duration in &self.durations {
                total += duration.subsec_nanos() as u64;
            }
            if total < self.get_overhead_ns() {
                // This should never happen, but technically it's possible.
                Some(0)
            } else {
                Some(total - self.get_overhead_ns())
            }
        }
    }

    pub(crate) fn get_overhead_ns(&self) -> u64 {
        let mut overhead =
            self.overhead.as_secs() * 1_000_000_000 + self.overhead.subsec_nanos() as u64;
        for child in &self.children {
            if let Ok(child) = child.try_lock() {
                overhead += child.get_overhead_ns();
            }
        }
        overhead
    }

    fn get_child(&mut self, name: &str) -> Option<Arc<Mutex<Measurement>>> {
        for child in &self.children {
            let mut child_lock = child.lock().unwrap();
            let child_name = child_lock.name.clone();
            if child_name == name {
                drop(child_lock);
                return Some(child.clone());
            }
        }
        None
    }
}
