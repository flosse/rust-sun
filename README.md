# sun

A rust port of the JS library [suncalc](https://github.com/mourner/suncalc/).

## install

Add the following to your `Cargo.toml`

    [dependencies]
    sun = "0.1.0"

## usage

```rust
extern crate sun;

pub fn main() {
  let pos = sun::get_pos();
  println!(pos);
}
```
