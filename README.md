# stperf
[![Crates.io](https://img.shields.io/crates/d/stperf.svg)](https://crates.io/crates/stperf)
[![Documentation](https://docs.rs/stperf/badge.svg)](https://docs.rs/stperf)
[![CI](https://img.shields.io/travis/neonmoe/stperf/0.1.3.svg)](https://travis-ci.org/neonmoe/stperf)

stperf (**s**ingle-**t**hreaded **perf**ormance profiler) is a very
simple profiling utility for single-threaded applications, inspired by
[`hprof`](https://crates.io/crates/hprof).

## Usage
Check out the [docs](https://docs.rs/stperf).

```rust
#[macro_use]
extern crate stperf;

fn main() {
    use std::thread;
    use std::time::Duration;

    let process = || {
        perf_measure!("processing");
        thread::sleep(Duration::from_millis(100));
    };

    for _ in 0..2 {
        perf_measure!("main");
        for _ in 0..2 {
            perf_measure!("inner operations");
            process();
        }
        process();
    }

    stperf::print();
}
```

Will print out:

```
╶──┬╼ main                 - 100.0%, 300 ms/loop, 2 samples
   ├──┬╼ inner operations  -  66.7%, 200 ms/loop, 4 samples
   │  └───╼ processing     - 100.0%, 200 ms/loop, 4 samples
   └───╼ processing        -  33.3%, 100 ms/loop, 2 samples
```

## License
This crate is distributed under the terms of the [ISC license](COPYING.md).
