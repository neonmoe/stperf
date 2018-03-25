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
