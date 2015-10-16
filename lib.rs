//! Closure-timing data structure.

#[macro_use]
extern crate log;
extern crate time as std_time;

use std::borrow::Borrow;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::convert::AsRef;
use std::sync::Mutex;

#[derive(Debug, Clone)]
/// A simple stopwatch that can time events and print stats about them.
struct Stopwatch {
  /// All the clocked intervals
  pub intervals: VecDeque<u64>,
}

impl Stopwatch {
  #[inline]
  /// Creates a new stopwatch.
  pub fn new() -> Stopwatch {
    Stopwatch {
      intervals: VecDeque::new(),
    }
  }

  pub fn push(&mut self, interval: u64) {
    self.intervals.push_back(interval);
    if self.intervals.len() > (1 << 15) {
      self.intervals.pop_front();
    }
  }

  /// Prints out timing statistics of this stopwatch.
  pub fn print(&self, name: &str) {
    let mut as_string = String::new();
    as_string.push_str(format!("{} = [", name).borrow());
    let mut first = true;
    for interval in self.intervals.iter() {
      if !first {
        as_string.push_str(",");
      }
      as_string.push_str(format!("{:?}", interval).borrow());
      first = false;
    }
    as_string.push_str("];");
    info!("{}", as_string);
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
    let then = std_time::precise_time_ns();
    trace!("Start timing {:?} at {:?}", name, then);
    let ret = f();
    let now = std_time::precise_time_ns();
    let interval = now - then;
    trace!("Stop timing {:?} at {:?} ({:?}us)", name, now, interval / 1000);

    let mut timers = self.timers.lock().unwrap();
    let stopwatch =
      timers.entry(name.to_string())
      .or_insert_with(|| Stopwatch::new())
    ;
    stopwatch.push(interval);

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

thread_local!(static TIMERSET: TimerSet = TimerSet::new());

/// Time with the thread-local `TimerSet`.
pub fn time<T, F: FnOnce() -> T>(name: &str, f: F) -> T {
  TIMERSET.with(|timerset| timerset.time(name, f))
}

pub fn clone() -> TimerSet {
  TIMERSET.with(|timerset| {
    TimerSet {
      timers: Mutex::new(timerset.timers.lock().unwrap().clone()),
    }
  })
}

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
