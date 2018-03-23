use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

lazy_static! {
    static ref MEASUREMENT_STACK: Mutex<Vec<Arc<Mutex<Measurement>>>> = Mutex::new(vec![
        Measurement::new("root".to_string(), 0)
    ]);
}

/// Starts a measurement in the current scope.
pub fn measure<T: Into<String>>(measurement_name: T) -> MeasurementTracker {
    let name = measurement_name.into();
    let mut stack = MEASUREMENT_STACK.lock().unwrap();
    let depth = stack.len();
    let measurement = Measurement::new(name.clone(), depth);
    if depth == 0 {
        stack.push(measurement);
    } else {
        let parent = stack.get(depth - 1).unwrap().clone();
        let mut parent = parent.lock().unwrap();
        if let Some(measurement) = parent.get_child(name.clone()) {
            stack.push(measurement.clone());
        } else {
            stack.push(measurement.clone());
            parent.children.push(measurement);
        }
    }

    MeasurementTracker {
        start_time: Instant::now(),
    }
}

/// Prints out the data gathered by the profiler.
pub fn print() {
    println!("{}", get_formatted_string());
}

/// Returns what [`print`](fn.print.html) prints, if you want to put it somewhere else
/// than stdout.
pub fn get_formatted_string() -> String {
    let mut result = String::new();
    let stack = MEASUREMENT_STACK.lock().unwrap();
    let root = stack.get(0).unwrap().lock().unwrap();
    let children = root.collect_all_children();
    let mut index = 0;
    let mut main_duration = 1.0;
    let mut main_count = 1;
    for measurement in children {
        if index == 0 {
            // Skip "root"
            index = 1;
            continue;
        }

        if let Some(duration) = measurement.get_avg_duration_ms() {
            let count = measurement.durations.len();
            let total_duration = duration * count as f64;
            if index == 1 {
                main_duration = total_duration;
                main_count = count;
            }
            let line = &format!(
                "{:width$}{}─{:.1}%, {:.6} ms {} times, {} samples",
                "",
                if index > 1 { "└" } else { "╾" },
                100.0 * total_duration / main_duration,
                duration,
                count / main_count,
                count,
                width = (measurement.depth - 1) * 2
            );
            result += &format!("{:width$}- {}\n", line, measurement.name, width = 55);
        } else {
            result += &format!(" {}\n", measurement.name);
        }
        index += 1;
    }
    result
}

/// Represents a measurement in a single scope.
#[derive(Clone)]
pub struct Measurement {
    name: String,
    depth: usize,
    durations: Vec<Duration>,
    children: Vec<Arc<Mutex<Measurement>>>,
}

impl Measurement {
    fn new(name: String, depth: usize) -> Arc<Mutex<Measurement>> {
        Arc::new(Mutex::new(Measurement {
            name,
            depth,
            durations: Vec::new(),
            children: Vec::new(),
        }))
    }

    fn collect_all_children(&self) -> Vec<Measurement> {
        let mut collection = Vec::new();
        collection.push(self.clone());
        for child in &self.children {
            collection.append(&mut child.lock().unwrap().collect_all_children());
        }
        collection
    }

    fn get_avg_duration_ms(&self) -> Option<f64> {
        let count = self.durations.len();
        if count == 0 {
            None
        } else {
            let mut total = 0.0;
            for duration in &self.durations {
                total += duration.subsec_nanos() as f64 / 1_000_000.0;
            }
            Some(total / count as f64)
        }
    }

    fn get_child(&mut self, name: String) -> Option<Arc<Mutex<Measurement>>> {
        for child in &self.children {
            let child_lock = child.lock().unwrap();
            let child_name = child_lock.name.clone();
            drop(child_lock);
            if child_name == name {
                return Some(child.clone());
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
        let now = Instant::now();
        let mut stack = MEASUREMENT_STACK.lock().unwrap();
        {
            let measurement = stack.pop().unwrap();
            let mut measurement = measurement.lock().unwrap();
            measurement.durations.push(now - self.start_time);
        }
    }
}
