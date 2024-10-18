//! The `sun` crate is a library for calculating the position of the sun and sun phases
//! (like sunrise, sunset).
//! It is a port of the `JavaScript` library
//! [suncalc](https://github.com/mourner/suncalc).
//!
//! # Example
//!
//! ```rust
//! let unixtime = 1_362_441_600_000;
//! let lat = 48.0;
//! let lon = 9.0;
//! let pos = sun::pos(unixtime,lat,lon);
//! let az  = pos.azimuth.to_degrees();
//! let alt = pos.altitude.to_degrees();
//! println!("The position of the sun is {az}/{alt}");
//!
//! // calculate time of sunrise
//! let time_ms = sun::time_at_phase(unixtime, sun::SunPhase::Sunrise, lat, lon, 0.0);
//! assert_eq!(time_ms, 1_362_463_116_241);
//! ```

use std::f64::consts::PI;

// date/time constants and conversions

const MILLISECONDS_PER_DAY: f64 = 1_000.0 * 60.0 * 60.0 * 24.0;
const JULIAN_0: f64 = 0.000_9;
const JULIAN_1970: f64 = 2_440_588.0;
const JULIAN_2000: f64 = 2_451_545.0;
const TO_RAD: f64 = PI / 180.0;
const OBLIQUITY_OF_EARTH: f64 = 23.439_7 * TO_RAD;
const PERIHELION_OF_EARTH: f64 = 102.937_2 * TO_RAD;

/// Holds the [azimuth](https://en.wikipedia.org/wiki/Azimuth)
/// and [altitude](https://en.wikipedia.org/wiki/Horizontal_coordinate_system)
/// angles of the sun position.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub azimuth: f64,
    pub altitude: f64,
}

const fn to_julian(unixtime_in_ms: f64) -> f64 {
    unixtime_in_ms / MILLISECONDS_PER_DAY - 0.5 + JULIAN_1970
}

#[allow(clippy::cast_possible_truncation)]
fn from_julian(julian_date: f64) -> i64 {
    ((julian_date + 0.5 - JULIAN_1970) * MILLISECONDS_PER_DAY).round() as i64
}

const fn to_days(unixtime_in_ms: f64) -> f64 {
    to_julian(unixtime_in_ms) - JULIAN_2000
}

// general calculations for position

fn right_ascension(ecliptic_longitude: f64, ecliptic_latitude: f64) -> f64 {
    (ecliptic_longitude.sin() * OBLIQUITY_OF_EARTH.cos()
        - ecliptic_latitude.tan() * OBLIQUITY_OF_EARTH.sin())
    .atan2(ecliptic_longitude.cos())
}

fn declination(ecliptic_longitude: f64, ecliptic_latitude: f64) -> f64 {
    (ecliptic_latitude.sin() * OBLIQUITY_OF_EARTH.cos()
        + ecliptic_latitude.cos() * OBLIQUITY_OF_EARTH.sin() * ecliptic_longitude.sin())
    .asin()
}

fn azimuth(sidereal_time: f64, latitude_rad: f64, declination: f64) -> f64 {
    sidereal_time
        .sin()
        .atan2(sidereal_time.cos() * latitude_rad.sin() - declination.tan() * latitude_rad.cos())
        + PI
}

fn altitude(sidereal_time: f64, latitude_rad: f64, declination: f64) -> f64 {
    (latitude_rad.sin() * declination.sin()
        + latitude_rad.cos() * declination.cos() * sidereal_time.cos())
    .asin()
}

fn sidereal_time(days: f64, longitude_rad: f64) -> f64 {
    (280.16 + 360.985_623_5 * days).to_radians() - longitude_rad
}

// general sun calculations

fn solar_mean_anomaly(days: f64) -> f64 {
    (357.529_1 + 0.985_600_28 * days).to_radians()
}

fn equation_of_center(solar_mean_anomaly: f64) -> f64 {
    (1.914_8 * solar_mean_anomaly.sin()
        + 0.02 * (2.0 * solar_mean_anomaly).sin()
        + 0.000_3 * (3.0 * solar_mean_anomaly).sin())
    .to_radians()
}

fn ecliptic_longitude(solar_mean_anomaly: f64) -> f64 {
    solar_mean_anomaly + equation_of_center(solar_mean_anomaly) + PERIHELION_OF_EARTH + PI
}

/// Calculates the sun position for a given date and latitude/longitude.
/// The angles are calculated as [radians](https://en.wikipedia.org/wiki/Radian).
///
/// * `unixtime`  - [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
/// * `lat`       - [latitude](https://en.wikipedia.org/wiki/Latitude) in degrees.
/// * `lon`       - [longitude](https://en.wikipedia.org/wiki/Longitude) in degrees.
///
/// calculates the sun position for a given date and latitude/longitude
#[must_use]
pub fn pos(unixtime_in_ms: i64, lat: f64, lon: f64) -> Position {
    let longitude_rad = -lon.to_radians();
    let latitude_rad = lat.to_radians();
    #[allow(clippy::cast_precision_loss)]
    let days = to_days(unixtime_in_ms as f64);
    let mean = solar_mean_anomaly(days);
    let ecliptic_longitude = ecliptic_longitude(mean);
    let declination = declination(ecliptic_longitude, 0.0);
    let right_ascension = right_ascension(ecliptic_longitude, 0.0);
    let sidereal_time = sidereal_time(days, longitude_rad) - right_ascension;
    let azimuth = azimuth(sidereal_time, latitude_rad, declination);
    let altitude = altitude(sidereal_time, latitude_rad, declination);
    Position { azimuth, altitude }
}

fn julian_cycle(days: f64, longitude_rad: f64) -> f64 {
    (days - JULIAN_0 - longitude_rad / (2.0 * PI)).round()
}

const fn approx_transit(hour_angle: f64, longitude_rad: f64, julian_cycle: f64) -> f64 {
    JULIAN_0 + (hour_angle + longitude_rad) / (2.0 * PI) + julian_cycle
}

fn solar_transit_julian(
    approx_transit: f64,
    solar_mean_anomaly: f64,
    ecliptic_longitude: f64,
) -> f64 {
    JULIAN_2000 + approx_transit + 0.005_3 * solar_mean_anomaly.sin()
        - 0.006_9 * (2.0 * ecliptic_longitude).sin()
}

fn solar_hour_angle(altitude_angle: f64, latitude_rad: f64, declination: f64) -> f64 {
    ((altitude_angle.sin() - latitude_rad.sin() * declination.sin())
        / (latitude_rad.cos() * declination.cos()))
    .acos()
}

fn observer_angle(height: f64) -> f64 {
    -2.076 * height.sqrt() / 60.0
}

/// Returns set time for the given sun altitude.
fn sunset_julian(
    altitude_angle: f64,
    longitude_rad: f64,
    latitude_rad: f64,
    declination: f64,
    julian_cycle: f64,
    mean: f64,
    ecliptic_longitude: f64,
) -> f64 {
    let hour_angle = solar_hour_angle(altitude_angle, latitude_rad, declination);
    let approx_transit = approx_transit(hour_angle, longitude_rad, julian_cycle);
    solar_transit_julian(approx_transit, mean, ecliptic_longitude)
}

/// Calculates the time for the given [`SunPhase`] at a given date, height and Latitude/Longitude.
/// The returned time is the [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
///
/// # Arguments
///
/// * `unixtime`  - [unix time](https://en.wikipedia.org/wiki/Unix_time) in milliseconds.
/// * `sun_phase` - [`SunPhase`] to calcuate time for
/// * `lat`       - [latitude](https://en.wikipedia.org/wiki/Latitude) in degrees.
/// * `lon`       - [longitude](https://en.wikipedia.org/wiki/Longitude) in degrees.
/// * `height`    - Observer height in meters above the horizon
///
/// # Examples
///
/// ```rust
/// // calculate time of sunrise
/// let unixtime = 1_362_441_600_000;
/// let lat = 48.0;
/// let lon = 9.0;
/// let height = 0.0;
/// let time_ms = sun::time_at_phase(unixtime, sun::SunPhase::Sunrise, lat, lon, height);
/// assert_eq!(time_ms, 1_362_463_116_241);
/// ```

#[must_use]
pub fn time_at_phase(
    unixtime_in_ms: i64,
    sun_phase: SunPhase,
    lat: f64,
    lon: f64,
    height: f64,
) -> i64 {
    let longitude_rad = -lon.to_radians();
    let latitude_rad = lat.to_radians();
    let observer_angle = observer_angle(height);
    #[allow(clippy::cast_precision_loss)]
    let days = to_days(unixtime_in_ms as f64);
    let julian_cycle = julian_cycle(days, longitude_rad);
    let approx_transit = approx_transit(0.0, longitude_rad, julian_cycle);
    let solar_mean_anomaly = solar_mean_anomaly(approx_transit);
    let ecliptic_longitude = ecliptic_longitude(solar_mean_anomaly);
    let declination = declination(ecliptic_longitude, 0.0);
    let julian_noon = solar_transit_julian(approx_transit, solar_mean_anomaly, ecliptic_longitude);

    let altitude_angle = (sun_phase.angle_deg() + observer_angle).to_radians();
    let julian_set = sunset_julian(
        altitude_angle,
        longitude_rad,
        latitude_rad,
        declination,
        julian_cycle,
        solar_mean_anomaly,
        ecliptic_longitude,
    );

    if sun_phase.is_rise() {
        let julian_rise = julian_noon - (julian_set - julian_noon);
        from_julian(julian_rise)
    } else {
        from_julian(julian_set)
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
    #[must_use]
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
        assert_eq!(2457054.5, to_julian(1422748800000.0));
    }

    #[test]
    fn test_from_julian() {
        // 1. Jan. 2015
        assert_eq!(from_julian(2457054.5), 1422748800000);
    }

    #[test]
    fn test_to_days() {
        // 1. Jan. 2015
        assert_eq!(5509.5, to_days(1422748800000.0));
    }
}
