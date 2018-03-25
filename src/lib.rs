//! # stpref
//!
//! stpref (**s**ingle-**t**hreaded **prof**iler) is a very simple
//! profiling utility for single-threaded applications, inspired by
//! [`hprof`](https://crates.io/crates/hprof). Mostly intended for
//! games. Before using this crate, be sure to check the
//! [Warning](#warning) section.
//!
//! # Usage
//!
//! Using the crate consists of two parts: measuring and
//! analyzing.
//!
//! The measurement part consists of calling the `perf_measure!` macro
//! at the start of every code block we want to measure, and giving it
//! an identifiable name.
//!
//! ```no_run
//! # #[macro_use] extern crate stperf; fn main() {
//! let process = |n| std::thread::sleep(std::time::Duration::from_millis(n));
//!
//! perf_measure!("top level processing");
//! {
//!     perf_measure!("light initial processing");
//!     process(100);
//! }
//! for _ in 0..5 {
//!     perf_measure!("heavy processing");
//!     process(100); // "heavy processing"
//! }
//! # }
//! ```
//!
//! The analysis part starts with printing out the information in any
//! of the following ways.
//!
//! ```
//! # #[macro_use] extern crate stperf; fn main() {
//! // Simply print out the data with some sensible defaults for configuration.
//! stperf::print();
//!
//! // Print out the data, but configure the output. In this case, we
//! // use a different format, and specify that we want to see the
//! // timings with 3 decimals.
//! stperf::print_with_format(stperf::format::COMPATIBLE, 3);
//!
//! // Just get the formatted string like with print_with_format,
//! // except it's a String so you can print it out somewhere else than
//! // stdout(). (A GUI, for example.)
//! let s = stperf::get_formatted_string(stperf::format::STREAMLINED, 2);
//! # }
//! ```
//!
//! And then you get to ponder the deeper meaning of a graph like this:
//!
//! ```text
//! ╶──┬╼ top level processing         - 100.0%, 600 ms/loop, 1 samples
//!    ├───╼ light initial processing  -  16.7%, 100 ms/loop, 1 samples
//!    └───╼ heavy processing          -  83.3%, 500 ms/loop, 5 samples
//! ```
//!
//! ## How to read the graph
//!
//! The graph will show scopes inside scopes in a tree-like
//! structure, with each indentation implying a deeper scope, and the
//! branches illustrating who is whose child and sibling.
//!
//! * The percents represent the fraction of the time the process took
//!   from its parent's processing time.
//!
//! * The ms/loop represents the total time the process took to finish
//!   inside the shallowest scope ("root scope"); the graph above shows
//!   500ms for "heavy processing" even though a single process took
//!   100ms, since it was ran 5 times inside "top level processing" (the
//!   shallowest scope, or "root scope").
//!
//! * The samples show how many perf_measure!'s were ran for this row
//!   of data.
//!
//! # Overhead
//!
//! The crate has a bit of performance overhead. Here's a few specific
//! causes, with performance measured on the machine this crate was
//! developed on (i7-6700k @ 4.1GHz):
//!
//! * The print/string formatting functions are quite heavy, as they go
//!   through all the measurement data. One call measures at about 3.0ms
//!   for 100k samples.
//!
//! * The [`perf_measure!`](macro.perf_measure.html) macro is pretty
//!   light, but when used in large quantities, may be noticeable. One
//!   call measures at about 200ns with a --release flag, 1µs
//!   without. However, stprof can track its own overhead to a degree,
//!   so the reported overhead is only about 50ns with --release, and
//!   360ns without.
//!
//! All this said, it's important to note: the most useful information
//! this profiler gives you is the percents, not the absolute timing
//! value.
//!
//! # Warning
//!
//! The crate accumulates a pretty large amount of data in a small
//! amount of time, if you're using it in a realtime application. A
//! recommended way of displaying and handling the measurement data is
//! as follows:
//! 1. Set an update interval, eg. 1 second.
//! 2. Every interval, print out the data (eg. `stperf::print()`), and
//!    cleanup (`stperf::reset()`).
//! This way, you'll always have quite a few samples (1 second is a
//! long amount of time to gather data), and they'll be fresh. And
//! you'll avoid filling up your ram.
//!
//! # Examples
//! ```
//! # #[macro_use] extern crate stperf; fn main() {
//! use std::thread;
//! use std::time::Duration;
//!
//! let process = || {
//!     perf_measure!("processing");
//!     thread::sleep(Duration::from_millis(100));
//! };
//!
//! for _ in 0..2 {
//!     perf_measure!("main");
//!     for _ in 0..2 {
//!         perf_measure!("inner operations");
//!         process();
//!     }
//!     process();
//! }
//!
//! stperf::print();
//! // Prints out:
//! // ╶──┬╼ main                 - 100.0%, 300 ms/loop, 2 samples
//! //    ├──┬╼ inner operations  -  66.7%, 200 ms/loop, 4 samples
//! //    │  └───╼ processing     - 100.0%, 200 ms/loop, 4 samples
//! //    └───╼ processing        -  33.3%, 100 ms/loop, 2 samples
//! # }
//! ```

#![deny(missing_docs)]

#[cfg(not(feature = "disabled"))]
#[macro_use]
extern crate lazy_static;

pub mod format;
#[allow(dead_code, unused_variables)]
mod measurement_tracker;
pub use measurement_tracker::MeasurementTracker;

#[cfg(not(feature = "disabled"))]
mod measurement;
#[cfg(not(feature = "disabled"))]
pub use measurement::{measure, reset};
#[cfg(not(feature = "disabled"))]
mod formatter;
#[cfg(not(feature = "disabled"))]
pub use formatter::{get_formatted_string, print, print_with_format};

#[cfg(feature = "disabled")]
#[allow(dead_code, unused_variables)]
mod disabled;
#[cfg(feature = "disabled")]
pub use disabled::*;

/// Logs the time between this call and the end of the current scope.
#[cfg(not(feature = "disabled"))]
#[macro_export]
macro_rules! perf_measure {
    ($s: expr) => {
        use std::time::Instant;
        #[allow(unused_variables)]
        let measurement = stperf::measure(Instant::now(), $s);
    };
}

/// Logs the time between this call and the end of the current scope.
#[cfg(feature = "disabled")]
#[macro_export]
macro_rules! perf_measure {
    ($s: expr) => {};
}
