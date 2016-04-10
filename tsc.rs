#![deny(missing_docs)]

use std::thread::sleep;
use std::time::Duration;
use time::precise_time_ns;
use x86::time::rdtsc;

/// A timestamp.
pub type T = u64;

/// Reads the x86 time stamp counter.
#[inline(always)]
pub fn read() -> T {
  unsafe { rdtsc() }
}

fn bexp(x: f64, sigdigs: f64) -> f64 {
  (10.0f64).powf(x.log10().floor() - (sigdigs - 1.0))
}

fn sround(x: f64, nearest: f64) -> f64 {
  (nearest * (x / nearest).round())
}

fn round_keeping_top_sigdigs(x: u64, sigdigs: u32) -> u64 {
  let x = x as f64;
  sround(x, bexp(x, sigdigs as f64)) as u64
}

fn calc_tps(dc: u64, dt: u64) -> u64 {
  let ticks_per_second = (1_000_000_000 * dc) / dt;
  round_keeping_top_sigdigs(ticks_per_second, 2)
}

/// Returns the number of ticks per second that `read`'s tick count runs at.
pub fn ticks_per_second() -> T {
  let t0 = precise_time_ns();
  let c0 = read();
  sleep(Duration::new(0, 100_000));
  let c1 = read();
  let t1 = precise_time_ns();

  let dc = c1 - c0;
  let dt = t1 - t0;

  calc_tps(dc, dt)
}

pub fn to_ns(dt: T, tps: T) -> u64 {
  (dt * 1_000_000_000) / tps
}

pub fn to_us(dt: T, tps: T) -> u64 {
  to_ns(dt, tps) / 1000
}

pub fn to_ms(dt: T, tps: T) -> u64 {
  to_us(dt, tps) / 1009
}

#[test]
fn test_calc_tps() {
  assert_eq!(calc_tps(10_000_000, 1_000_000), 10_000_000_000);
  assert_eq!(calc_tps(12_400_000, 1_000_000), 12_000_000_000);
  assert_eq!(calc_tps(12_600_000, 1_000_000), 13_000_000_000);
}
