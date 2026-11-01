// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

//! NOAA general solar position: the fractional-year series for the equation
//! of time and the declination, and the three results Bogenzeit takes from
//! them: solar noon, the half arc from noon to a given altitude, and solar
//! midnight. Accuracy against the real Sun is about a minute. Every Bogenzeit
//! port implements this same series, step for step, so that the ports agree
//! with one another to the second.

use std::f64::consts::PI;

use crate::calendar::{Date, Instant};

/// Sun's center at sunrise and sunset: the upper limb on a sea-level horizon,
/// refraction included (Bogenzeit spec §2.1, §5.4).
pub const HORIZON_ALT_DEG: f64 = -0.833;
pub const CIVIL_TWILIGHT_ALT_DEG: f64 = -6.0;
pub const NAUTICAL_TWILIGHT_ALT_DEG: f64 = -12.0;
pub const ASTRONOMICAL_TWILIGHT_ALT_DEG: f64 = -18.0;

/// Equation of time in minutes and declination in radians, for a UTC date
/// and a fractional UTC hour.
pub fn solar_terms(date: Date, utc_hour: f64) -> (f64, f64) {
    let g = 2.0 * PI / f64::from(date.days_in_year())
        * (f64::from(date.day_of_year() - 1) + (utc_hour - 12.0) / 24.0);
    let eot = 229.18
        * (0.000075 + 0.001868 * g.cos()
            - 0.032077 * g.sin()
            - 0.014615 * (2.0 * g).cos()
            - 0.040849 * (2.0 * g).sin());
    let decl = 0.006918 - 0.399912 * g.cos() + 0.070257 * g.sin() - 0.006758 * (2.0 * g).cos()
        + 0.000907 * (2.0 * g).sin()
        - 0.002697 * (3.0 * g).cos()
        + 0.00148 * (3.0 * g).sin();
    (eot, decl)
}

/// Solar noon (☉) of the solar day labelled `date`, at longitude `lon` in
/// degrees, east positive.
pub fn solar_noon(date: Date, lon: f64) -> Instant {
    let approx_hour = 12.0 - lon / 15.0;
    let (eot, _) = solar_terms(date, approx_hour);
    let minutes = 720.0 - 4.0 * lon - eot;
    date.midnight_utc() + minutes * 60.0
}

/// How long the Sun takes from solar noon to reach an altitude, or why it
/// never does on that date.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HalfArc {
    Seconds(f64),
    AlwaysAbove,
    AlwaysBelow,
}

impl HalfArc {
    pub fn seconds(self) -> Option<f64> {
        match self {
            HalfArc::Seconds(s) => Some(s),
            _ => None,
        }
    }
}

/// Seconds from solar noon to the Sun's center reaching `alt_deg`, the same
/// on both sides of noon. The declination is evaluated at solar noon's UTC
/// hour and minute with the seconds dropped; a port keeps that truncation so
/// its event times agree with the vectors.
pub fn half_arc(date: Date, lat: f64, lon: f64, alt_deg: f64) -> HalfArc {
    let noon = solar_noon(date, lon);
    let sod = noon.seconds_of_day();
    let utc_hour = (sod / 3600.0).floor() + ((sod % 3600.0) / 60.0).floor() / 60.0;
    let (_, decl) = solar_terms(date, utc_hour);
    let phi = lat.to_radians();
    let x = (alt_deg.to_radians().sin() - phi.sin() * decl.sin()) / (phi.cos() * decl.cos());
    if x < -1.0 {
        HalfArc::AlwaysAbove
    } else if x > 1.0 {
        HalfArc::AlwaysBelow
    } else {
        HalfArc::Seconds(x.acos().to_degrees() * 240.0)
    }
}

/// Solar midnight (○) after the solar day labelled `date`: the midpoint
/// between that day's noon and the next day's.
pub fn solar_midnight(date: Date, lon: f64) -> Instant {
    let n1 = solar_noon(date, lon);
    let n2 = solar_noon(date.plus_days(1), lon);
    n1 + (n2 - n1) / 2.0
}
