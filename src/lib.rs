// Copyright (c) 2014 Markus Kohlhase <mail@markus-kohlhase.de>

// This is a port of the JavaScript library suncalc:
// https://github.com/mourner/suncalc

use std::f64::consts::PI;

// date/time constants and conversions

const MILLISECONDS_PER_DAY  : u32 = 1000 * 60 * 60 * 24;
const J1970                 : u32 = 2440588;
const J2000                 : u32 = 2451545;
const TO_RAD                : f64 = PI / 180.0;
const OBLIQUITY_OF_EARTH    : f64 = 23.4397  * TO_RAD;
const PERIHELION_OF_EARTH   : f64 = 102.9372 * TO_RAD;

#[derive(Debug)]
pub struct Position {
  pub azimuth   : f64,
  pub altitude  : f64
}

fn to_julian(unixtime: i64) -> f64 {
  unixtime as f64 /
  (MILLISECONDS_PER_DAY as f64) - 0.5 + J1970 as f64
}

#[test]
fn test_to_julian(){
  // 1. Jan. 2015
  assert_eq!(2457054.5, to_julian(1422748800000));
}

fn to_days(unixtime: i64) -> f64 {
  to_julian(unixtime) - J2000 as f64
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
  (280.16 + 360.9856235 * d) * TO_RAD - lw
}

// general sun calculations

fn solar_mean_anomaly(d:f64) -> f64 {
  (357.5291 + 0.98560028 * d) * TO_RAD
}

fn equation_of_center(m:f64) -> f64 {
  (1.9148 * (1.0 * m).sin() +
   0.02   * (2.0 * m).sin() +
   0.0003 * (3.0 * m).sin()
  ) * TO_RAD
}

fn ecliptic_longitude(m:f64) -> f64 {
  m + equation_of_center(m) + PERIHELION_OF_EARTH + PI
}

/// calculates the sun position for a given date and latitude/longitude
pub fn get_pos(unixtime: i64, lat: f64, lon: f64) -> Position {

  let lw  = -lon * TO_RAD;
  let phi = lat * TO_RAD;
  let d   = to_days(unixtime);
  let m   = solar_mean_anomaly(d);
  let l   = ecliptic_longitude(m);
  let dec = declination(l, 0.0);
  let ra  = right_ascension(l, 0.0);
  let h   = sidereal_time(d, lw) - ra;

  Position {
    azimuth  :  azimuth(h, phi, dec),
    altitude : altitude(h, phi, dec)
  }
}

#[test]
fn test_get_pos(){
  // 2013-03-05 UTC
  let date = 1362441600000;
  let pos = get_pos(date, 50.5, 30.5);
  assert_eq!(0.6412750628729547, pos.azimuth);
  assert_eq!(-0.7000406838781611, pos.altitude);
}
