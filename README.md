# sun

A [Rust](https://www.rust-lang.org) port of the JS library [suncalc](https://github.com/mourner/suncalc/).

[![Crates.io](https://img.shields.io/crates/v/sun.svg)](https://crates.io/crates/sun)
[![Docs.rs](https://docs.rs/sun/badge.svg)](https://docs.rs/sun/)

## Install

Add the following to your `Cargo.toml`

    [dependencies]
    sun = "0.3"

## Usage

```rust
pub fn main() {
  let unixtime = 1_362_441_600_000.0;
  let lat = 48.0;
  let lon = 9.0;
  let pos = sun::pos(unixtime,lat,lon);
  let az  = pos.azimuth.to_degrees();
  let alt = pos.altitude.to_degrees();
  println!("The position of the sun is {az}/{alt}");

  let time_ms = sun::time_at_phase(unixtime, sun::SunPhase::Sunrise, lat, lon, 0.0);
  println!("Sunrise is at {time_ms}");
}
```
