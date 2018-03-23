#[macro_use]
extern crate stprof;

fn main() {
    for _ in 0..1000 {
        measure!("main");
        for _ in 0..2 {
            measure!("inner thing");
            for _ in 0..4 {
                measure!("innerer thing");
            }
            for _ in 0..3 {
                measure!("another innerer thing");
                for _ in 0..5 {
                    for _ in 0..10 {
                        measure!("the innest thing");
                    }
                }
            }
        }
        for _ in 0..20 {
            measure!("inner thing B");
        }
    }
    stprof::print();
    // Prints out:
    // ╾─100.0%, 0.543573 ms, 1000 samples          main
    //   └─92.8%, 0.252216 ms, 2000 samples         inner thing
    //     └─0.5%, 0.000342 ms, 8000 samples        innerer thing
    //     └─88.7%, 0.080343 ms, 6000 samples       another innerer thing
    //       └─18.8%, 0.000340 ms, 300000 samples   the innest thing
    //   └─1.3%, 0.000342 ms, 20000 samples         inner thing
}
