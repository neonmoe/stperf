//! The backend for the measurements.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

lazy_static! {
    pub(crate) static ref MEASUREMENT_STACK: Mutex<Vec<Arc<Mutex<Measurement>>>> =
        Mutex::new(vec![Measurement::new("root".to_string(), 0, None)]);
}

/// Starts a measurement in the current scope.
pub fn measure<T: Into<String>>(measurement_name: T) -> MeasurementTracker {
    let now = Instant::now();
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

    MeasurementTracker { start_time: now }
}

/// Returns a `Vec` of all the
/// [`Measurement`](struct.Measurement.html)s taken so far.
///
/// **Warning**: This function is pretty heavy, especially as the
/// amount of samples rises, as it clones every one of them.
pub fn get_measures() -> Vec<Measurement> {
    let stack = MEASUREMENT_STACK.lock().unwrap();
    let root = stack.get(0).unwrap().lock().unwrap();
    root.collect_all_children()
}

/// Represents a scope's running time.
#[derive(Clone)]
pub struct Measurement {
    pub(crate) name: String,
    pub(crate) depth: usize,
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
            durations: Vec::new(),
            parent: parent.into(),
            children: Vec::new(),
            children_names: Vec::new(),
        }))
    }

    /// Returns the name of the measurement (the string passed to the
    /// `measure!` macro).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the depth of the measurement. The first `measure!`
    /// call will have a depth of 1, and each `measure!` inside
    /// another `measure!` will increment the depth by 1.  Consider
    /// the following:
    /// ```
    /// #[macro_use]
    /// extern crate stprof;
    ///
    /// fn main() {
    ///     prof_measure!("main");                     /* Depth == 1 */
    ///     { prof_measure!("inner"); }                /* Depth == 2 */
    ///     { prof_measure!("another inner"); }        /* Depth == 2 */
    ///     {
    ///         prof_measure!("third inner");          /* Depth == 2 */
    ///         { prof_measure!("even more inner"); }  /* Depth == 3 */
    ///     }
    ///     { prof_measure!("last inner"); }           /* Depth == 2 */
    /// }
    /// ```
    pub fn depth(&self) -> usize {
        self.depth
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

    pub(crate) fn get_avg_duration_ns(&self) -> Option<u32> {
        let count = self.durations.len();
        if count == 0 {
            None
        } else {
            let mut total = 0;
            for duration in &self.durations {
                total += duration.subsec_nanos();
            }
            Some(total / count as u32)
        }
    }

    fn get_child(&mut self, name: &str) -> Option<Arc<Mutex<Measurement>>> {
        for child in &self.children {
            let mut child_lock = child.lock().unwrap();
            let child_name = child_lock.name.clone();
            if child_name == name {
                drop(child_lock);
                return Some(child.clone());
            } else if let Some(result) = child_lock.get_child(name) {
                drop(child_lock);
                return Some(result);
            }
        }
        None
    }
}

/// Represents a started measurement. When dropped, it will log the
/// duration into memory.
pub struct MeasurementTracker {
    start_time: Instant,
}

impl Drop for MeasurementTracker {
    fn drop(&mut self) {
        let mut stack = MEASUREMENT_STACK.lock().unwrap();
        let measurement = stack.pop().unwrap();
        let mut measurement = measurement.lock().unwrap();
        measurement.durations.push(Instant::now() - self.start_time);
    }
}
