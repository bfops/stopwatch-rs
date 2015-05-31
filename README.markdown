A small, simple named timer library.

Example usage:

    extern crate stopwatch;

    fn main() {
      let ts = stopwatch::TimerSet::new();
      ts.time("hello", || {
        for _ in 0..100 {
          ts.time("world", || {
            std::thread::sleep_ms(1);
          });
        }
      });

      ts.print();
    }
