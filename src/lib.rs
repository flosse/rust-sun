// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

//! The `sun` crate is a library for calculating the position of the sun.
//! It is a port of the `JavaScript` library
//! [suncalc](https://github.com/mourner/suncalc).
//!
//! # Example
//!
//! ```
//! extern crate sun;
//!
//! pub fn main() {
//!   let unixtime = 1362441600000;
//!   let lat = 48.0;
//!   let lon = 9.0;
//!   let pos = sun::pos(unixtime,lat,lon);
//!   let az  = pos.azimuth.to_degrees();
//!   let alt = pos.altitude.to_degrees();
//!   println!("The position of the sun is {}/{}", az, alt);
//! }
//! ```

use std::f64::consts::PI;

// date/time constants and conversions

const MILLISECONDS_PER_DAY  : u32 = 1000 * 60 * 60 * 24;
const J1970                 : u32 = 2440588;
const J2000                 : u32 = 2451545;
const TO_RAD                : f64 = PI / 180.0;
const OBLIQUITY_OF_EARTH    : f64 = 23.4397  * TO_RAD;
const PERIHELION_OF_EARTH   : f64 = 102.9372 * TO_RAD;

/// Holds the [azimuth](https://en.wikipedia.org/wiki/Azimuth)
/// and [altitude](https://en.wikipedia.org/wiki/Horizontal_coordinate_system)
/// angles of the sun position.
#[derive(Debug, Clone)]
pub struct Position {
  pub azimuth   : f64,
  pub altitude  : f64
}

#[derive(Debug)]
struct Coords {
    pub right_ascension: f64,
    pub declination: f64,
}

fn to_julian(unixtime_in_ms: i64) -> f64 {
  unixtime_in_ms as f64 /
  (MILLISECONDS_PER_DAY as f64) - 0.5 + J1970 as f64
}

#[test]
fn test_to_julian(){
  // 1. Jan. 2015
  assert_eq!(2457054.5, to_julian(1422748800000));
}

fn to_days(unixtime_in_ms: i64) -> f64 {
  to_julian(unixtime_in_ms) - J2000 as f64
}

#[test]
fn test_to_days(){
  // 1. Jan. 2015
  assert_eq!(5509.5, to_days(1422748800000));
}

// general calculations for position

fn right_ascension(l:f64, b:f64) -> f64 {
  (
    l.sin() * OBLIQUITY_OF_EARTH.cos() -
    b.tan() * OBLIQUITY_OF_EARTH.sin()
  )
  .atan2(l.cos())
}

fn declination(l:f64, b:f64) -> f64 {
  (
    b.sin() * OBLIQUITY_OF_EARTH.cos() +
    b.cos() * OBLIQUITY_OF_EARTH.sin() * l.sin()
  )
  .asin()
}

fn azimuth(h:f64, phi:f64, dec:f64) -> f64  {
  h.sin()
  .atan2(
    h.cos()   * phi.sin() -
    dec.tan() * phi.cos()
  ) + PI
}

fn altitude(h:f64, phi:f64, dec:f64) -> f64 {
  (
    phi.sin() * dec.sin() +
    phi.cos() * dec.cos() * h.cos()
  )
  .asin()
}

fn sidereal_time(d:f64, lw:f64) -> f64 {
  (280.16 + 360.9856235 * d).to_radians() - lw
}

// general sun calculations

fn solar_mean_anomaly(d:f64) -> f64 {
  (357.5291 + 0.98560028 * d).to_radians()
}

fn equation_of_center(m:f64) -> f64 {
  (1.9148 * (1.0 * m).sin() +
   0.02   * (2.0 * m).sin() +
   0.0003 * (3.0 * m).sin()
  ).to_radians()
}

fn ecliptic_longitude(m:f64) -> f64 {
  m + equation_of_center(m) + PERIHELION_OF_EARTH + PI
}

fn sun_coords (d: f64) -> Coords {
    let m   = solar_mean_anomaly(d);
    let l   = ecliptic_longitude(m);

    Coords {
        right_ascension: right_ascension(l, 0.0),
        declination: declination(l, 0.0),
    }
}

/// Calculates the sun position for a given date and latitude/longitude.
/// The angles are calculated as [radians](https://en.wikipedia.org/wiki/Radian).
///
/// * `unixtime`  - [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
/// * `lat`       - [latitude](https://en.wikipedia.org/wiki/Latitude) in degrees.
/// * `lon`       - [longitude](https://en.wikipedia.org/wiki/Longitude) in degrees.
/// calculates the sun position for a given date and latitude/longitude
pub fn pos(unixtime_in_ms: i64, lat: f64, lon: f64) -> Position {

  let lw  = -lon.to_radians();
  let phi = lat.to_radians();
  let d   = to_days(unixtime_in_ms);
  let coords = sun_coords(d);
  let h   = sidereal_time(d, lw) - coords.right_ascension;

  Position {
    azimuth  :  azimuth(h, phi, coords.declination),
    altitude : altitude(h, phi, coords.declination)
  }
}

#[test]
fn test_pos(){
  // 2013-03-05 UTC
  let date = 1362441600000;
  let pos = pos(date, 50.5, 30.5);
  assert_eq!(0.6412750628729547, pos.azimuth);
  assert_eq!(-0.7000406838781611, pos.altitude);
}

// Moon

#[derive(Debug)]
pub struct Illumination {
    pub fraction: f64,
    pub phase: f64,
    pub angle: f64,
}

// general moon calculations

fn astro_refraction(h: f64) -> f64 {
    let hh = if h < 0.0 {
        0.0
    } else {
        h
    };

    0.0002967 / (hh + 0.00312536 / (hh + 0.08901179)).tan()
}

fn lunar_mean_anomaly(d: f64) -> f64 {
    (134.963 + 13.064993 * d).to_radians()
}

fn lunar_ecliptic_longitude(d: f64) -> f64 {
    (218.316 + 13.176396 * d).to_radians()
}

fn lunar_mean_distance(d: f64) -> f64 {
    (93.272 + 13.229350 * d).to_radians()
}

fn moon_coords(d: f64) -> Coords {
    let l = lunar_ecliptic_longitude(d);
    let m = lunar_mean_anomaly(d);
    let f = lunar_mean_distance(d);

    let lng = l + TO_RAD * 6.289 * m.sin();
    let lat = TO_RAD * 5.128 * f.sin();

    Coords {
        right_ascension: right_ascension(lng, lat),
        declination: declination(lng, lat),
    }
}

/// calculates the moon position for a given date and latitude/longitude
pub fn moon_pos(unixtime_in_ms: i64, lat: f64, lon: f64) -> Position {
    let lw = TO_RAD * -lon;
    let phi = TO_RAD * lat;
    let d = to_days(unixtime_in_ms);

    let c = moon_coords(d);

    let h = sidereal_time(d, lw) - c.right_ascension;
    let mut alt = altitude(h, phi, c.declination);
    alt = alt + astro_refraction(alt);

    Position {
        azimuth: azimuth(h, phi, c.declination),
        altitude: alt
    }
}

#[test]
fn test_moon_pos() {
    let date = 1362441600000;
    let pos = moon_pos(date, 50.5, 30.5);

    assert_eq!(-0.9783999522438225, pos.azimuth - PI);
    assert_eq!(0.014551482243892251, pos.altitude);
}

/// calculates the moon illumination, phase, and angle for a given date
pub fn moon_illumination(unixtime_in_ms: i64) -> Illumination {
    let d = to_days(unixtime_in_ms);
    let s = sun_coords(d);
    let m = moon_coords(d);
    let a = lunar_mean_anomaly(d);

    let distance = 385001.0 - 20905.0 * a.cos();  // distance to the moon in km

    let sdist = 149598000 as f64;

    let phi = (s.declination.sin() * m.declination.sin() + s.declination.cos() * m.declination.cos() * (s.right_ascension - m.right_ascension).cos()).acos();

    let inc = (sdist * phi.sin()).atan2(distance - sdist * phi.cos());
    let angle = (s.declination.cos() * (s.right_ascension - m.right_ascension).sin()).atan2(s.declination.sin() * m.declination.cos() - s.declination.cos() * (m.declination).sin() * (s.right_ascension - m.right_ascension).cos());


    let sign = if angle < 0.0 {
        -1.0
    } else {
        1.0
    };

    Illumination {
        fraction: (1.0 + inc.cos()) / 2.0,
        phase: 0.5 + 0.5 * inc * sign / PI,
        angle: angle,
    }
}

#[test]
fn test_moon_illumination() {
    let date = 1362441600000;
    let moon_illum = moon_illumination(date);

    assert_eq!(moon_illum.fraction, 0.4848068202456373);
    assert_eq!(moon_illum.phase, 0.7548368838538762);
    assert_eq!(moon_illum.angle, 1.6732942678578346);
}
