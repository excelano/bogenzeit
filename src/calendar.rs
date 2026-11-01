// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

//! Dates and instants, UTC only and without a dependency. Bogenzeit needs a
//! Gregorian date, seconds since the Unix epoch, and the conversions between
//! them, and nothing else from a calendar.

use std::fmt;
use std::ops::{Add, Sub};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DAY_S: f64 = 86_400.0;

/// A Gregorian calendar date.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Date, String> {
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return Err(format!(
                "{year:04}-{month:02}-{day:02} is not a calendar date"
            ));
        }
        Ok(Date { year, month, day })
    }

    /// Parses `YYYY-MM-DD`.
    pub fn parse(s: &str) -> Result<Date, String> {
        let bad = || format!("`{s}` is not a date; write YYYY-MM-DD");
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return Err(bad());
        }
        let year = s[0..4].parse().map_err(|_| bad())?;
        let month = s[5..7].parse().map_err(|_| bad())?;
        let day = s[8..10].parse().map_err(|_| bad())?;
        Date::new(year, month, day)
    }

    pub fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    pub fn days_in_year(self) -> u32 {
        if Self::is_leap_year(self.year) {
            366
        } else {
            365
        }
    }

    /// 1 for 1 January.
    pub fn day_of_year(self) -> u32 {
        let jan1 = Date {
            year: self.year,
            month: 1,
            day: 1,
        };
        (self.days_since_epoch() - jan1.days_since_epoch() + 1) as u32
    }

    /// Days since 1970-01-01, negative before it.
    pub fn days_since_epoch(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    pub fn from_days_since_epoch(days: i64) -> Date {
        let z = days + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Date { year, month, day }
    }

    pub fn plus_days(self, n: i64) -> Date {
        Self::from_days_since_epoch(self.days_since_epoch() + n)
    }

    /// 00:00:00 UTC on this date.
    pub fn midnight_utc(self) -> Instant {
        Instant(self.days_since_epoch() as f64 * DAY_S)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if Date::is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// A UTC instant: seconds since the Unix epoch, with a fraction.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Instant(pub f64);

impl Instant {
    pub fn now() -> Instant {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is set before 1970");
        Instant(since_epoch.as_secs_f64())
    }

    /// Parses RFC 3339: `2026-11-01T17:08:00Z`, with an optional fraction of
    /// a second and either `Z` or a `±HH:MM` offset.
    pub fn parse_rfc3339(s: &str) -> Result<Instant, String> {
        let bad = || format!("`{s}` is not an RFC 3339 instant; write 2026-11-01T17:08:00Z");
        let (date_part, rest) = s.split_once(['T', 't', ' ']).ok_or_else(bad)?;
        let date = Date::parse(date_part).map_err(|_| bad())?;
        let (time_part, offset_s) = if let Some(t) = rest.strip_suffix(['Z', 'z']) {
            (t, 0.0)
        } else {
            let i = rest.rfind(['+', '-']).ok_or_else(bad)?;
            let (t, off) = rest.split_at(i);
            let sign = if off.starts_with('-') { -1.0 } else { 1.0 };
            let (oh, om) = off[1..].split_once(':').ok_or_else(bad)?;
            let oh: f64 = oh.parse().map_err(|_| bad())?;
            let om: f64 = om.parse().map_err(|_| bad())?;
            (t, sign * (oh * 3600.0 + om * 60.0))
        };
        let b = time_part.as_bytes();
        if b.len() < 8 || b[2] != b':' || b[5] != b':' {
            return Err(bad());
        }
        let hour: f64 = time_part[0..2].parse().map_err(|_| bad())?;
        let minute: f64 = time_part[3..5].parse().map_err(|_| bad())?;
        let second: f64 = time_part[6..].parse().map_err(|_| bad())?;
        if hour >= 24.0 || minute >= 60.0 || !(0.0..61.0).contains(&second) {
            return Err(bad());
        }
        let seconds = hour * 3600.0 + minute * 60.0 + second - offset_s;
        Ok(date.midnight_utc() + seconds)
    }

    pub fn utc_date(self) -> Date {
        Date::from_days_since_epoch((self.0 / DAY_S).floor() as i64)
    }

    /// Seconds since 00:00 UTC of the instant's own date.
    pub fn seconds_of_day(self) -> f64 {
        self.0.rem_euclid(DAY_S)
    }

    /// RFC 3339 to the whole second, truncated toward the past.
    pub fn rfc3339(self) -> String {
        self.format(0)
    }

    /// RFC 3339 with three decimals.
    pub fn rfc3339_millis(self) -> String {
        self.format(3)
    }

    fn format(self, decimals: usize) -> String {
        let sod = self.seconds_of_day();
        let hour = (sod / 3600.0).floor();
        let minute = ((sod - hour * 3600.0) / 60.0).floor();
        let second = sod - hour * 3600.0 - minute * 60.0;
        let second = if decimals == 0 {
            second.floor()
        } else {
            (second * 1000.0).floor() / 1000.0
        };
        let width = if decimals == 0 { 2 } else { 3 + decimals };
        format!(
            "{}T{:02}:{:02}:{:0width$.decimals$}Z",
            self.utc_date(),
            hour,
            minute,
            second,
            width = width,
            decimals = decimals
        )
    }
}

impl Add<f64> for Instant {
    type Output = Instant;
    fn add(self, seconds: f64) -> Instant {
        Instant(self.0 + seconds)
    }
}

impl Sub<f64> for Instant {
    type Output = Instant;
    fn sub(self, seconds: f64) -> Instant {
        Instant(self.0 - seconds)
    }
}

impl Sub for Instant {
    type Output = f64;
    fn sub(self, other: Instant) -> f64 {
        self.0 - other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip_through_epoch_days() {
        for days in (-200_000..200_000).step_by(37) {
            let d = Date::from_days_since_epoch(days);
            assert_eq!(d.days_since_epoch(), days, "{d}");
        }
        assert_eq!(Date::new(1970, 1, 1).unwrap().days_since_epoch(), 0);
        assert_eq!(Date::new(2000, 3, 1).unwrap().days_since_epoch(), 11_017);
    }

    #[test]
    fn day_of_year_counts_leap_days() {
        assert_eq!(Date::parse("2028-02-29").unwrap().day_of_year(), 60);
        assert_eq!(Date::parse("2028-12-31").unwrap().day_of_year(), 366);
        assert_eq!(Date::parse("2100-12-31").unwrap().day_of_year(), 365);
        assert!(Date::parse("2027-02-29").is_err());
    }

    #[test]
    fn instants_parse_and_print() {
        let t = Instant::parse_rfc3339("2026-11-01T17:08:00Z").unwrap();
        assert_eq!(t.0, 1_793_552_880.0);
        assert_eq!(t.rfc3339(), "2026-11-01T17:08:00Z");
        let with_offset = Instant::parse_rfc3339("2026-11-01T11:08:00-06:00").unwrap();
        assert_eq!(with_offset, t);
        let fractional = Instant::parse_rfc3339("2026-11-01T18:05:05.977137+00:00").unwrap();
        assert_eq!(fractional.rfc3339_millis(), "2026-11-01T18:05:05.977Z");
        assert!(Instant::parse_rfc3339("2026-11-01").is_err());
        assert!(Instant::parse_rfc3339("2026-11-01T25:00:00Z").is_err());
    }
}
