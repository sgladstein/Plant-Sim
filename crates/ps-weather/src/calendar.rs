//! Civil calendar arithmetic (proleptic Gregorian), with no time zones.
//!
//! The sim clock is local mean solar time: noon on the clock is when the mean sun is
//! due south (north in the southern hemisphere). See docs/decisions/0005.

use serde::{Deserialize, Serialize};

/// A calendar date and whole hour on the sim clock.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DateHour {
    /// Year, for example 2000.
    pub year: i32,
    /// Month, 1 to 12.
    pub month: u32,
    /// Day of month, 1 to 31.
    pub day: u32,
    /// Hour, 0 to 23.
    pub hour: u32,
}

/// True for Gregorian leap years.
#[must_use]
pub const fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days in the year.
#[must_use]
pub const fn days_in_year(year: i32) -> u32 {
    if is_leap(year) { 366 } else { 365 }
}

/// Days in a month.
#[must_use]
pub const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 for a civil date (H. Hinnant's algorithm).
#[must_use]
pub fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(month);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Civil date for days since 1970-01-01.
#[must_use]
pub fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + i64::from(m <= 2);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    (y as i32, m as u32, d as u32)
}

/// Day of year, 1-based.
#[must_use]
pub fn day_of_year(year: i32, month: u32, day: u32) -> u32 {
    let d = days_from_civil(year, month, day) - days_from_civil(year, 1, 1) + 1;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let d = d as u32;
    d
}

impl DateHour {
    /// Builds a date-hour, checking ranges.
    #[must_use]
    pub fn new(year: i32, month: u32, day: u32, hour: u32) -> Option<Self> {
        if (1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month) && hour < 24 {
            Some(Self {
                year,
                month,
                day,
                hour,
            })
        } else {
            None
        }
    }

    /// Day of year, 1-based.
    #[must_use]
    pub fn day_of_year(self) -> u32 {
        day_of_year(self.year, self.month, self.day)
    }

    /// Hours since 1970-01-01T00.
    #[must_use]
    pub fn epoch_hours(self) -> i64 {
        days_from_civil(self.year, self.month, self.day) * 24 + i64::from(self.hour)
    }

    /// Inverse of [`DateHour::epoch_hours`].
    #[must_use]
    pub fn from_epoch_hours(h: i64) -> Self {
        let (year, month, day) = civil_from_days(h.div_euclid(24));
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let hour = h.rem_euclid(24) as u32;
        Self {
            year,
            month,
            day,
            hour,
        }
    }

    /// The date-hour `hours` after this one.
    #[must_use]
    pub fn plus_hours(self, hours: u64) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        Self::from_epoch_hours(self.epoch_hours() + hours as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_days() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
        assert_eq!(day_of_year(2000, 12, 31), 366);
        assert_eq!(day_of_year(2001, 12, 31), 365);
        assert_eq!(day_of_year(2000, 5, 15), 136);
    }

    #[test]
    fn round_trip_many_days() {
        for z in -800_000..800_000_i64 {
            if z % 997 != 0 {
                continue;
            }
            let (y, m, d) = civil_from_days(z);
            assert_eq!(days_from_civil(y, m, d), z);
        }
    }

    #[test]
    fn plus_hours_crosses_year() {
        let t = DateHour::new(2000, 12, 31, 23).unwrap().plus_hours(2);
        assert_eq!(t, DateHour::new(2001, 1, 1, 1).unwrap());
    }

    #[test]
    fn rejects_bad_dates() {
        assert!(DateHour::new(2001, 2, 29, 0).is_none());
        assert!(DateHour::new(2000, 2, 29, 0).is_some());
        assert!(DateHour::new(2000, 13, 1, 0).is_none());
        assert!(DateHour::new(2000, 1, 1, 24).is_none());
    }
}
