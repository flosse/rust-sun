# sun

A rust port of the JS library [suncalc](https://github.com/mourner/suncalc/).

[![Build Status](https://travis-ci.org/flosse/rust-sun.svg?branch=master)](https://travis-ci.org/flosse/rust-sun)

## Install

Add the following to your `Cargo.toml`

    [dependencies]
    sun = "0.1"

## Usage

```rust
extern crate sun;

pub fn main() {
  let unixtime = 1362441600000;
  let lat = 48.0;
  let lon = 9.0;
  let pos = sun::pos(unixtime,lat,lon);
  println!(pos);
}
```
