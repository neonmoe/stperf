#[macro_use]
extern crate stprof;
use std::thread;
use std::time::Duration;

fn main() {
    let process = || thread::sleep(Duration::from_millis(1));

    for _ in 0..2 {
        prof_measure!("main");
        for _ in 0..2 {
            prof_measure!("inner thing");
            for _ in 0..2 {
                prof_measure!("innerer thing");
                process();
            }
            for _ in 0..2 {
                prof_measure!("another innerer things");
                process();
                for _ in 0..2 {
                    prof_measure!("another innerer zthing");
                    process();
                    for _ in 0..2 {
                        for _ in 0..2 {
                            prof_measure!("anotherc innerer thing");
                            process();
                        }
                        prof_measure!("another inunerer thing");
                        process();
                    }
                    for _ in 0..2 {
                        prof_measure!("another innerere thing");
                        process();
                        for _ in 0..2 {
                            prof_measure!("another innerer thing");
                            process();
                        }
                        for _ in 0..2 {
                            prof_measure!("another innerer thinsg");
                            process();
                        }
                    }
                }
                for _ in 0..2 {
                    prof_measure!("another innerer thding");
                    process();
                    for _ in 0..2 {
                        prof_measure!("another innerer thing");
                        process();
                    }
                    for _ in 0..2 {
                        prof_measure!("anothera innerer thing");
                        process();
                    }
                }
            }
        }
        for _ in 0..2 {
            prof_measure!("inner thing B");
            process();
            for _ in 0..2 {
                prof_measure!("another innerer thingc");
                process();
                for _ in 0..2 {
                    prof_measure!("another innerer thinbg");
                    process();
                    for _ in 0..2 {
                        prof_measure!("another innerer thinga");
                        process();
                    }
                }
                for _ in 0..2 {
                    prof_measure!("another innerer thing2");
                    process();
                }
                for _ in 0..2 {
                    prof_measure!("another innerer thing3");
                    process();
                }
                for _ in 0..2 {
                    prof_measure!("another innerer thing4");
                    process();
                }
            }
            for _ in 0..2 {
                prof_measure!("another innerer thing6");
                process();
            }
        }
    }

    stprof::print();
}
