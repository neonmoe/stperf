//! The backend for the measurements.

use std::sync::{Arc, Mutex, MutexGuard, TryLockResult};
use std::time::{Duration, Instant};

use measurement_tracker::MeasurementTracker;

lazy_static! {
    pub(crate) static ref MEASUREMENT_STACK: Mutex<Vec<MeasurementRef>> =
        Mutex::new(vec![Measurement::new("root".to_string(), 0, None)]);
}

/// Starts a measurement in the current scope. **Don't use this, use
/// the [`perf_measure!`](macro.perf_measure.html) macro.**
pub fn measure<T: Into<String>>(now: Instant, measurement_name: T) -> MeasurementTracker {
    let name = measurement_name.into();
    let mut stack = MEASUREMENT_STACK.lock().unwrap();
    let depth = stack.len();

    let parent = stack.get(depth - 1).unwrap().clone();
    let measurement = Measurement::new(
        name.clone(),
        depth,
        Some(MeasurementRef::from(parent.clone())),
    );

    let mut parent = parent.get_mut();
    if let Some(existing_measurement) = parent.get_child(&name) {
        {
            let mut measurement = existing_measurement.get_mut();
            measurement.measuring_currently = true;
        }
        stack.push(existing_measurement.clone());
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
        let latest_measurement = stack.pop().unwrap();
        let mut measurement = latest_measurement.get_mut();
        measurement.measuring_currently = false;
        measurement.overhead += self.overhead;
        measurement.durations.push(Instant::now() - self.start_time);
        measurement.overhead += Instant::now() - latter_overhead_start;
    }
}

/// Resets the measurement data.
///
/// **Warning**: This will wipe all measurements from the memory!
pub fn reset() {
    let stack = MEASUREMENT_STACK.lock().unwrap();
    let root = stack.get(0).unwrap().get_mut();
    let children = root.collect_all_children_arc();

    for i in 0..children.len() {
        let mut child = children[i].get_mut();
        if !child.measuring_currently {
            child.remove_while_locked();
        } else {
            child.clear_durations();
        }
    }
}

/// Returns a `Vec` of all the
/// [`Measurement`](struct.Measurement.html)s taken so far.
///
/// **Warning**: This function is pretty heavy, especially as the
/// amount of samples rises, as it clones every one of them.
pub(crate) fn get_measures() -> Vec<Measurement> {
    let stack = MEASUREMENT_STACK.lock().unwrap();
    let root = stack.get(0).unwrap().get_mut();
    root.collect_all_children()
}

#[derive(Clone, Debug)]
pub(crate) struct MeasurementRef {
    reference: Arc<Mutex<Measurement>>,
}

impl MeasurementRef {
    pub(crate) fn get_mut(&self) -> MutexGuard<Measurement> {
        match self.reference.try_lock() {
            Ok(measurement) => measurement,
            Err(err) => panic!("Failed to lock measurement! {}", err),
        }
    }

    fn try_get_mut(&self) -> TryLockResult<MutexGuard<Measurement>> {
        self.reference.try_lock()
    }
}

impl From<Arc<Mutex<Measurement>>> for MeasurementRef {
    fn from(t: Arc<Mutex<Measurement>>) -> Self {
        MeasurementRef { reference: t }
    }
}

/// Represents a scope's running time.
#[derive(Clone, Debug)]
pub(crate) struct Measurement {
    pub(crate) name: String,
    pub(crate) depth: usize,
    pub(crate) overhead: Duration,
    pub(crate) durations: Vec<Duration>,
    pub(crate) parent: Option<MeasurementRef>,
    children: Vec<MeasurementRef>,
    children_names: Vec<String>,
    measuring_currently: bool,
}

impl Measurement {
    fn new(name: String, depth: usize, parent: Option<MeasurementRef>) -> MeasurementRef {
        MeasurementRef::from(Arc::new(Mutex::new(Measurement {
            name,
            depth,
            overhead: Duration::new(0, 0),
            durations: Vec::new(),
            parent: parent.into(),
            children: Vec::new(),
            children_names: Vec::new(),
            measuring_currently: true,
        })))
    }

    pub(crate) fn get_ancestor(&self, generation: u32) -> Option<MeasurementRef> {
        if generation == 0 {
            if let Some(ref parent) = self.parent {
                Some(MeasurementRef::from(parent.clone()))
            } else {
                None
            }
        } else {
            if let Some(ref parent) = self.parent {
                let parent = parent.get_mut();
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
            collection.append(&mut child.get_mut().collect_all_children());
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
            if let Ok(child) = child.try_get_mut() {
                overhead += child.get_overhead_ns();
            }
        }
        overhead
    }

    fn collect_all_children_arc(&self) -> Vec<MeasurementRef> {
        let mut collection = Vec::new();
        for child in &self.children {
            collection.push(child.clone());
            let child = child.get_mut();
            collection.append(&mut child.collect_all_children_arc());
        }
        collection
    }

    fn get_child(&mut self, name: &str) -> Option<MeasurementRef> {
        for child in &self.children {
            let mut child_lock = child.get_mut();
            let child_name = child_lock.name.clone();
            if child_name == name {
                return Some(child.clone());
            }
        }
        None
    }

    fn remove_locked_child(&mut self) {
        let children = &mut self.children;
        let mut remove_index = None;
        for i in 0..children.len() {
            if let Err(_) = children[i].try_get_mut() {
                remove_index = Some(i);
                break;
            }
        }
        if let Some(i) = remove_index {
            children.remove(i);
        }
    }

    fn remove_while_locked(&self) {
        if let Some(ref parent) = self.parent {
            let mut parent = parent.get_mut();
            parent.remove_locked_child();
        }
    }

    fn clear_durations(&mut self) {
        self.durations.clear();
    }
}
