#[macro_use]
extern crate stprof;

fn main() {
    use std::thread;
    use std::time::Duration;

    let process = || thread::sleep(Duration::from_millis(100));
    for _ in 0..1 {
        prof_measure!("main");
        for _ in 0..1 {
            prof_measure!("physics simulation");
            for _ in 0..1 {
                prof_measure!("moving things");
                process();
            }
            for _ in 0..1 {
                prof_measure!("resolving collisions");
                process();
            }
        }
        for _ in 0..1 {
            prof_measure!("rendering");
            process();
        }
    }
    stprof::print();
    // Prints out:
    // ╶──┬╼ main                      - 100.0%, 300 ms/loop
    //    ├──┬╼ physics simulation     -  66.7%, 200 ms/loop
    //    │  ├─╼ moving things         -  50.0%, 100 ms/loop
    //    │  └─╼ resolving collisions  -  50.0%, 100 ms/loop
    //    └─╼ rendering                -  33.3%, 100 ms/loop
}
