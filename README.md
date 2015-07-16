# sun

A rust port of the JS library [suncalc](https://github.com/mourner/suncalc/).

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
