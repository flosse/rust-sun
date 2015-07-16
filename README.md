# sun

A rust port of the JS library [suncalc](https://github.com/mourner/suncalc/).

[![Build Status](https://travis-ci.org/flosse/rust-sun.svg?branch=master)](https://travis-ci.org/flosse/rust-sun)

## Unstall

Add the following to your `Cargo.toml`

    [dependencies]
    sun = "0.1.0"

## Usage

```rust
extern crate sun;

pub fn main() {
  let pos = sun::get_pos();
  println!(pos);
}
```
