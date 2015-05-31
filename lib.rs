//! Closure-timing data structure.

#[macro_use]
extern crate log;
extern crate time;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::convert::AsRef;
use std::sync::Mutex;

#[derive(Debug, Copy, Clone)]
/// A simple stopwatch that can time events and print stats about them.
pub struct Stopwatch {
  /// The total amount of time clocked.
  pub total_time: u64,
  /// The number of time windows we've clocked.
  pub number_of_windows: u64,
}

impl Stopwatch {
  #[inline]
  /// Creates a new stopwatch.
  pub fn new() -> Stopwatch {
    Stopwatch {
      total_time: 0,
      number_of_windows: 0,
    }
  }

  #[inline]
  /// Times a function, updating stats as necessary.
  pub fn timed<T, F: FnOnce() -> T>(&mut self, event: F) -> T {
    let then = time::precise_time_ns();
    let ret = event();
    self.total_time += time::precise_time_ns() - then;
    self.number_of_windows += 1;
    ret
  }

  /// Prints out timing statistics of this stopwatch.
  fn print(&self, name: &str) {
    if self.number_of_windows == 0 {
      info!("{} never ran", name);
    } else {
      info!(
        "{}: {}ms over {} samples (avg {}us)",
        name,
        self.total_time / 1000000,
        self.number_of_windows,
        (self.total_time / self.number_of_windows / 1000),
      );
    }
  }

}

unsafe impl Send for Stopwatch {}
unsafe impl Sync for Stopwatch {}

/// A set of stopwatches for multiple, named events.
pub struct TimerSet {
  timers: Mutex<HashMap<String, Stopwatch>>,
}

impl TimerSet {
  /// Creates a new set of timers.
  pub fn new() -> TimerSet {
    TimerSet { timers: Mutex::new(HashMap::new()) }
  }

  /// Times the execution of a function, and logs it under a timer with
  /// the given name.
  ///
  /// This function is not marked `mut` because borrow checking is done
  /// dynamically.
  pub fn time<T, F: FnOnce() -> T>(&self, name: &str, f: F) -> T {
    let then = time::precise_time_ns();
    let ret = f();
    let total_time = time::precise_time_ns() - then;

    let mut timers = self.timers.lock().unwrap();
    match timers.entry(name.to_string()) {
      Entry::Occupied(mut entry) => {
        entry.get_mut().number_of_windows += 1;
        entry.get_mut().total_time += total_time;
      },
      Entry::Vacant(entry) => {
        entry.insert(Stopwatch {
          total_time: total_time,
          number_of_windows: 1,
        });
      },
    }

    ret
  }

  /// Prints all the timer statistics to stdout, each tagged with their name.
  pub fn print(&self) {
    let timers = self.timers.lock().unwrap();
    let mut timer_vec : Vec<(&str, &Stopwatch)> =
      timers
        .iter()
        .map(|(name, sw)| (name.as_ref(), sw))
        .collect();

    timer_vec.sort_by(|&(k1, _), &(k2, _)| k1.cmp(k2));

    for &(name, ref timer) in timer_vec.iter() {
      timer.print(name);
    }
  }
}

unsafe impl Send for TimerSet {}
unsafe impl Sync for TimerSet {}

#[test]
fn test_simple() {
  let ts = TimerSet::new();
  ts.time("hello", || {});
}

#[test]
fn test_nested() {
  let ts = TimerSet::new();
  ts.time("hello", || {
    ts.time("world", || {});
  });
}
