//! The `sun` crate is a library for calculating the position of the sun and sun phases
//! (like sunrise, sunset).
//! It is a port of the `JavaScript` library
//! [suncalc](https://github.com/mourner/suncalc).
//!
//! # Example
//!
//! ```rust
//! let unixtime = 1362441600000;
//! let lat = 48.0;
//! let lon = 9.0;
//! let pos = sun::pos(unixtime,lat,lon);
//! let az  = pos.azimuth.to_degrees();
//! let alt = pos.altitude.to_degrees();
//! println!("The position of the sun is {}/{}", az, alt);
//!
//! // calculate time of sunrise
//! let time_ms = sun::time_at_phase(unixtime, sun::SunPhase::Sunrise, lat, lon, 0.0);
//! assert_eq!(time_ms, 1362463116241);
//! ```

use std::f64::consts::PI;

// date/time constants and conversions

const MILLISECONDS_PER_DAY: u32 = 1000 * 60 * 60 * 24;
const J0: f64 = 0.0009;
const J1970: u32 = 2_440_588;
const J2000: u32 = 2_451_545;
const TO_RAD: f64 = PI / 180.0;
const OBLIQUITY_OF_EARTH: f64 = 23.4397 * TO_RAD;
const PERIHELION_OF_EARTH: f64 = 102.9372 * TO_RAD;

/// Holds the [azimuth](https://en.wikipedia.org/wiki/Azimuth)
/// and [altitude](https://en.wikipedia.org/wiki/Horizontal_coordinate_system)
/// angles of the sun position.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub azimuth: f64,
    pub altitude: f64,
}

const fn to_julian(unixtime_in_ms: i64) -> f64 {
    unixtime_in_ms as f64 / (MILLISECONDS_PER_DAY as f64) - 0.5 + J1970 as f64
}

fn from_julian(j: f64) -> i64 {
    ((j + 0.5 - J1970 as f64) * MILLISECONDS_PER_DAY as f64).round() as i64
}

const fn to_days(unixtime_in_ms: i64) -> f64 {
    to_julian(unixtime_in_ms) - J2000 as f64
}

// general calculations for position

fn right_ascension(l: f64, b: f64) -> f64 {
    (l.sin() * OBLIQUITY_OF_EARTH.cos() - b.tan() * OBLIQUITY_OF_EARTH.sin()).atan2(l.cos())
}

fn declination(l: f64, b: f64) -> f64 {
    (b.sin() * OBLIQUITY_OF_EARTH.cos() + b.cos() * OBLIQUITY_OF_EARTH.sin() * l.sin()).asin()
}

fn azimuth(h: f64, phi: f64, dec: f64) -> f64 {
    h.sin().atan2(h.cos() * phi.sin() - dec.tan() * phi.cos()) + PI
}

fn altitude(h: f64, phi: f64, dec: f64) -> f64 {
    (phi.sin() * dec.sin() + phi.cos() * dec.cos() * h.cos()).asin()
}

fn sidereal_time(d: f64, lw: f64) -> f64 {
    (280.16 + 360.985_623_5 * d).to_radians() - lw
}

// general sun calculations

fn solar_mean_anomaly(d: f64) -> f64 {
    (357.5291 + 0.985_600_28 * d).to_radians()
}

fn equation_of_center(m: f64) -> f64 {
    (1.9148 * (1.0 * m).sin() + 0.02 * (2.0 * m).sin() + 0.0003 * (3.0 * m).sin()).to_radians()
}

fn ecliptic_longitude(m: f64) -> f64 {
    m + equation_of_center(m) + PERIHELION_OF_EARTH + PI
}

/// Calculates the sun position for a given date and latitude/longitude.
/// The angles are calculated as [radians](https://en.wikipedia.org/wiki/Radian).
///
/// * `unixtime`  - [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
/// * `lat`       - [latitude](https://en.wikipedia.org/wiki/Latitude) in degrees.
/// * `lon`       - [longitude](https://en.wikipedia.org/wiki/Longitude) in degrees.
///
/// calculates the sun position for a given date and latitude/longitude
pub fn pos(unixtime_in_ms: i64, lat: f64, lon: f64) -> Position {
    let lw = -lon.to_radians();
    let phi = lat.to_radians();
    let d = to_days(unixtime_in_ms);
    let m = solar_mean_anomaly(d);
    let l = ecliptic_longitude(m);
    let dec = declination(l, 0.0);
    let ra = right_ascension(l, 0.0);
    let h = sidereal_time(d, lw) - ra;

    Position {
        azimuth: azimuth(h, phi, dec),
        altitude: altitude(h, phi, dec),
    }
}

fn julian_cycle(d: f64, lw: f64) -> f64 {
    (d - J0 - lw / (2.0 * PI)).round()
}

const fn approx_transit(ht: f64, lw: f64, n: f64) -> f64 {
    J0 + (ht + lw) / (2.0 * PI) + n
}

fn solar_transit_j(ds: f64, m: f64, l: f64) -> f64 {
    J2000 as f64 + ds + 0.0053 * m.sin() - 0.0069 * (2.0 * l).sin()
}

fn hour_angle(h: f64, phi: f64, d: f64) -> f64 {
    ((h.sin() - phi.sin() * d.sin()) / (phi.cos() * d.cos())).acos()
}

fn observer_angle(height: f64) -> f64 {
    -2.076 * height.sqrt() / 60.0
}

/// returns set time for the given sun altitude
fn get_set_j(h: f64, lw: f64, phi: f64, dec: f64, n: f64, m: f64, l: f64) -> f64 {
    let w = hour_angle(h, phi, dec);
    let a = approx_transit(w, lw, n);

    solar_transit_j(a, m, l)
}

/// Calculates the time for the given [`SunPhase`] at a given date, height and Latitude/Longitude.
/// The returned time is the [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
///
/// # Arguments
///
/// * `date`      - [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
/// * `sun_phase` - [`SunPhase`] to calcuate time for
/// * `lat`       - [latitude](https://en.wikipedia.org/wiki/Latitude) in degrees.
/// * `lon`       - [longitude](https://en.wikipedia.org/wiki/Longitude) in degrees.
/// * `height`    - Observer height in meters above the horizon
///
/// # Examples
///
/// ```rust
/// // calculate time of sunrise
/// let unixtime = 1362441600000;
/// let lat = 48.0;
/// let lon = 9.0;
/// let time_ms = sun::time_at_phase(unixtime, sun::SunPhase::Sunrise, lat, lon, 0.0);
/// assert_eq!(time_ms, 1362463116241);
/// ```

pub fn time_at_phase(date: i64, sun_phase: SunPhase, lat: f64, lon: f64, height: f64) -> i64 {
    let lw = -lon.to_radians();
    let phi = lat.to_radians();

    let dh = observer_angle(height);

    let d = to_days(date);
    let n = julian_cycle(d, lw);
    let ds = approx_transit(0.0, lw, n);

    let m = solar_mean_anomaly(ds);
    let l = ecliptic_longitude(m);
    let dec = declination(l, 0.0);

    let j_noon = solar_transit_j(ds, m, l);

    let h0 = (sun_phase.angle_deg() + dh).to_radians();
    let j_set = get_set_j(h0, lw, phi, dec, n, m, l);

    if sun_phase.is_rise() {
        let j_rise = j_noon - (j_set - j_noon);
        from_julian(j_rise)
    } else {
        from_julian(j_set)
    }
}

/// Sun phases for use with [`time_at_phase`].
#[derive(Clone, Copy, Debug)]
pub enum SunPhase {
    Sunrise,
    Sunset,
    SunriseEnd,
    SunsetStart,
    Dawn,
    Dusk,
    NauticalDawn,
    NauticalDusk,
    NightEnd,
    Night,
    GoldenHourEnd,
    GoldenHour,
    Custom(f64, bool),
}

impl SunPhase {
    /// Create a custom sun phase
    ///
    /// # Arguments
    /// * `angle_deg` - Angle in degrees of the sun above the horizon. Use negative
    ///                 numbers for angles below the horizon.
    /// * `rise`      - `true` when this sun phase applies to the sun rising, `false`
    ///                 if it's setting.
    pub const fn custom(angle_deg: f64, rise: bool) -> Self {
        SunPhase::Custom(angle_deg, rise)
    }

    const fn angle_deg(&self) -> f64 {
        match self {
            SunPhase::Sunrise | SunPhase::Sunset => -0.833,
            SunPhase::SunriseEnd | SunPhase::SunsetStart => -0.5,
            SunPhase::Dawn | SunPhase::Dusk => -6.0,
            SunPhase::NauticalDawn | SunPhase::NauticalDusk => -12.0,
            SunPhase::NightEnd | SunPhase::Night => -18.0,
            SunPhase::GoldenHourEnd | SunPhase::GoldenHour => 6.0,
            SunPhase::Custom(angle, _) => *angle,
        }
    }

    const fn is_rise(&self) -> bool {
        match self {
            SunPhase::Sunrise
            | SunPhase::SunriseEnd
            | SunPhase::Dawn
            | SunPhase::NauticalDawn
            | SunPhase::NightEnd
            | SunPhase::GoldenHourEnd => true,
            SunPhase::Sunset
            | SunPhase::SunsetStart
            | SunPhase::Dusk
            | SunPhase::NauticalDusk
            | SunPhase::Night
            | SunPhase::GoldenHour => false,
            SunPhase::Custom(_, rise) => *rise,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_pos() {
        // 2013-03-05 UTC
        let date = 1362441600000;
        let pos = pos(date, 50.5, 30.5);
        assert_eq!(0.6412750628729547, pos.azimuth);
        assert_eq!(-0.7000406838781611, pos.altitude);
    }

    #[test]
    fn test_time_at_angle() {
        // 2013-03-05 UTC
        let date = 1362441600000;

        assert_eq!(
            time_at_phase(date, SunPhase::Sunrise, 50.5, 30.5, 0.0),
            1362458096440
        );
        assert_eq!(
            time_at_phase(date, SunPhase::Sunset, 50.5, 30.5, 0.0),
            1362498417875
        );

        // equal to Dusk
        assert_eq!(
            time_at_phase(date, SunPhase::custom(-6.0, false), 50.5, 30.5, 0.0),
            1362500376781
        );
        // equal to Dawn
        assert_eq!(
            time_at_phase(date, SunPhase::custom(-6.0, true), 50.5, 30.5, 0.0),
            1362456137534
        );
    }

    #[test]
    fn test_to_julian() {
        // 1. Jan. 2015
        assert_eq!(2457054.5, to_julian(1422748800000));
    }

    #[test]
    fn test_from_julian() {
        // 1. Jan. 2015
        assert_eq!(from_julian(2457054.5), 1422748800000);
    }

    #[test]
    fn test_to_days() {
        // 1. Jan. 2015
        assert_eq!(5509.5, to_days(1422748800000));
    }
}
