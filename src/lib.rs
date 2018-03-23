//! # stprof
//! stprof (**s**ingle-**t**hreaded **prof**iler) is a very simple
//! profiling utility for single-threaded applications. Mostly
//! intended for games.
//!
//! # Examples
//! ```
//! # #[macro_use]
//! # extern crate stprof;
//! # fn main() {
//! use std::thread;
//! use std::time::Duration;
//!
//! // An arbitrary "do something" function
//! let process = || thread::sleep(Duration::from_millis(100));
//! for _ in 0..1 {
//!     prof_measure!("main");
//!     for _ in 0..1 {
//!         prof_measure!("physics simulation");
//!         for _ in 0..1 {
//!             prof_measure!("moving things");
//!             process();
//!         }
//!         for _ in 0..1 {
//!             prof_measure!("resolving collisions");
//!             process();
//!         }
//!     }
//!     for _ in 0..1 {
//!         prof_measure!("rendering");
//!         process();
//!     }
//! }
//! stprof::print();
//!
//! // Prints out:
//! // ╶──┬╼ main                      - 100.0%, 300 ms/loop
//! //    ├──┬╼ physics simulation     -  66.7%, 200 ms/loop
//! //    │  ├─╼ moving things         -  50.0%, 100 ms/loop
//! //    │  └─╼ resolving collisions  -  50.0%, 100 ms/loop
//! //    └─╼ rendering                -  33.3%, 100 ms/loop
//! # }
//! ```

#![deny(missing_docs)]

#[macro_use]
extern crate lazy_static;

pub mod measurement;
mod formatter;
pub use formatter::*;

/// Logs the time between this call and the end of the current scope.
#[macro_export]
macro_rules! prof_measure {
    ($s: expr) => {
        #[allow(unused_variables)]
        let measurement = stprof::measurement::measure($s);
    };
}
