# sun

A rust port of the JS library [suncalc](https://github.com/mourner/suncalc/).

[![](http://meritbadge.herokuapp.com/sun)](https://crates.io/crates/sun)
[![Build Status](https://travis-ci.org/flosse/rust-sun.svg?branch=master)](https://travis-ci.org/flosse/rust-sun)
[![Clippy Linting Result](https://clippy.bashy.io/github/flosse/rust-sun/master/badge.svg)](https://clippy.bashy.io/github/flosse/rust-sun/master/log)

## Install

Add the following to your `Cargo.toml`

    [dependencies]
    sun = "0.2"

## Usage

```rust
extern crate sun;

pub fn main() {
  let unixtime = 1362441600000;
  let lat = 48.0;
  let lon = 9.0;
  let pos = sun::pos(unixtime,lat,lon);
  let az  = pos.azimuth.to_degrees();
  let alt = pos.altitude.to_degrees();
  println!("The position of the sun is {}/{}", az, alt);
}
```
