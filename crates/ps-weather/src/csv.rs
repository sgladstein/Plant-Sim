//! Weather file readers (design plan §6.2).
//!
//! Hourly files: `timestamp, t_air_c, rh_pct, radiation_w_m2, rain_mm` and optional
//! `wind_m_s`, `cloud_fraction`, in any column order. Timestamps are ISO 8601 local
//! clock time, `YYYY-MM-DDTHH[:MM[:SS]]`; minutes must be zero.
//!
//! Daily files: `date, t_min_c, t_max_c, rh_pct, radiation_mj_m2, rain_mm`, disaggregated
//! with [`crate::hourly::disaggregate`].
//!
//! Plain comma-separated text with a header row; `#` starts a comment line; no quoting.

use ps_units::{Celsius, Degrees, JPerM2, KPa, Millimeters, WPerM2};

use crate::calendar::{DateHour, day_of_year};
use crate::hourly::{HourWeather, WeatherSeries};
use crate::presets::DailyWeather;
use crate::radiation::{
    cloud_fraction_from_kt, erbs_diffuse_fraction, saturation_vapor_pressure_kpa,
};
use crate::solar::{extraterrestrial_horizontal_j_m2, sun_position};

/// Errors from reading weather files.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CsvError {
    /// The file has no header row.
    #[error("weather file is empty")]
    Empty,
    /// A required column is missing.
    #[error("missing column \"{0}\"")]
    MissingColumn(&'static str),
    /// A row has the wrong number of fields.
    #[error("line {line}: expected {expected} fields, found {found}")]
    FieldCount {
        /// 1-based line number.
        line: usize,
        /// Fields in the header.
        expected: usize,
        /// Fields in the row.
        found: usize,
    },
    /// A field could not be parsed.
    #[error("line {line}, column \"{column}\": cannot parse \"{value}\"")]
    Parse {
        /// 1-based line number.
        line: usize,
        /// Column name.
        column: String,
        /// Raw text.
        value: String,
    },
    /// Hourly rows are not consecutive hours.
    #[error("line {line}: expected timestamp {expected:?}, found {found:?}")]
    Gap {
        /// 1-based line number.
        line: usize,
        /// The hour that should come next.
        expected: DateHour,
        /// The hour found.
        found: DateHour,
    },
    /// A value is physically impossible.
    #[error("line {line}, column \"{column}\": value {value} out of range")]
    Range {
        /// 1-based line number.
        line: usize,
        /// Column name.
        column: &'static str,
        /// The value.
        value: f64,
    },
}

struct Table<'a> {
    header: Vec<&'a str>,
    rows: Vec<(usize, Vec<&'a str>)>,
}

fn read_table(text: &str) -> Result<Table<'_>, CsvError> {
    let mut lines = text
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'));
    let (_, head) = lines.next().ok_or(CsvError::Empty)?;
    let header: Vec<&str> = head.split(',').map(str::trim).collect();
    let mut rows = Vec::new();
    for (line, l) in lines {
        let fields: Vec<&str> = l.split(',').map(str::trim).collect();
        if fields.len() != header.len() {
            return Err(CsvError::FieldCount {
                line,
                expected: header.len(),
                found: fields.len(),
            });
        }
        rows.push((line, fields));
    }
    Ok(Table { header, rows })
}

impl Table<'_> {
    fn col(&self, name: &'static str) -> Result<usize, CsvError> {
        self.header
            .iter()
            .position(|h| *h == name)
            .ok_or(CsvError::MissingColumn(name))
    }
    fn opt_col(&self, name: &str) -> Option<usize> {
        self.header.iter().position(|h| *h == name)
    }
}

fn num(line: usize, column: &str, v: &str) -> Result<f64, CsvError> {
    v.parse::<f64>()
        .ok()
        .filter(|x| x.is_finite())
        .ok_or_else(|| CsvError::Parse {
            line,
            column: column.to_owned(),
            value: v.to_owned(),
        })
}

fn check(line: usize, column: &'static str, v: f64, lo: f64, hi: f64) -> Result<f64, CsvError> {
    if (lo..=hi).contains(&v) {
        Ok(v)
    } else {
        Err(CsvError::Range {
            line,
            column,
            value: v,
        })
    }
}

fn parse_date(line: usize, column: &str, s: &str) -> Result<(i32, u32, u32), CsvError> {
    let bad = || CsvError::Parse {
        line,
        column: column.to_owned(),
        value: s.to_owned(),
    };
    let mut it = s.split('-');
    let y = it
        .next()
        .and_then(|x| x.parse::<i32>().ok())
        .ok_or_else(bad)?;
    let m = it
        .next()
        .and_then(|x| x.parse::<u32>().ok())
        .ok_or_else(bad)?;
    let d = it
        .next()
        .and_then(|x| x.parse::<u32>().ok())
        .ok_or_else(bad)?;
    if it.next().is_some() {
        return Err(bad());
    }
    DateHour::new(y, m, d, 0).ok_or_else(bad)?;
    Ok((y, m, d))
}

fn parse_timestamp(line: usize, s: &str) -> Result<DateHour, CsvError> {
    let bad = || CsvError::Parse {
        line,
        column: "timestamp".to_owned(),
        value: s.to_owned(),
    };
    let (date, time) = s.split_once('T').ok_or_else(bad)?;
    let (y, m, d) = parse_date(line, "timestamp", date)?;
    let mut parts = time.split(':');
    let h = parts
        .next()
        .and_then(|x| x.parse::<u32>().ok())
        .ok_or_else(bad)?;
    for p in parts {
        if p.parse::<u32>().ok() != Some(0) {
            return Err(bad());
        }
    }
    DateHour::new(y, m, d, h).ok_or_else(bad)
}

/// Reads an hourly weather file. The first row must be 00:00 of the start date or
/// earlier; rows must be consecutive hours.
///
/// # Errors
/// Any [`CsvError`].
pub fn read_hourly(
    text: &str,
    latitude: Degrees,
    start: DateHour,
) -> Result<WeatherSeries, CsvError> {
    let t = read_table(text)?;
    let c_ts = t.col("timestamp")?;
    let c_t = t.col("t_air_c")?;
    let c_rh = t.col("rh_pct")?;
    let c_g = t.col("radiation_w_m2")?;
    let c_r = t.col("rain_mm")?;
    let c_cloud = t.opt_col("cloud_fraction");

    let day0 = DateHour::new(start.year, start.month, start.day, 0)
        .ok_or(CsvError::Empty)?
        .epoch_hours();
    let mut hours = Vec::new();
    let mut expected: Option<DateHour> = None;
    for (line, f) in &t.rows {
        let line = *line;
        let ts = parse_timestamp(line, f[c_ts])?;
        if let Some(e) = expected
            && e != ts
        {
            return Err(CsvError::Gap {
                line,
                expected: e,
                found: ts,
            });
        }
        expected = Some(ts.plus_hours(1));
        if ts.epoch_hours() < day0 {
            continue;
        }
        let ta = check(line, "t_air_c", num(line, "t_air_c", f[c_t])?, -60.0, 60.0)?;
        let rh = check(line, "rh_pct", num(line, "rh_pct", f[c_rh])?, 0.0, 100.0)?;
        let g = check(
            line,
            "radiation_w_m2",
            num(line, "radiation_w_m2", f[c_g])?,
            0.0,
            1500.0,
        )?;
        let rain = check(line, "rain_mm", num(line, "rain_mm", f[c_r])?, 0.0, 500.0)?;
        let doy = day_of_year(ts.year, ts.month, ts.day);
        let h = f64::from(ts.hour);
        let h0 = extraterrestrial_horizontal_j_m2(latitude, ts.year, doy, h, h + 1.0) / 3600.0;
        let kt = if h0 > 0.0 { (g / h0).min(1.0) } else { 0.0 };
        let fd = if h0 > 0.0 {
            erbs_diffuse_fraction(kt)
        } else {
            1.0
        };
        let es = saturation_vapor_pressure_kpa(ta);
        let cloud = match c_cloud {
            Some(c) => check(
                line,
                "cloud_fraction",
                num(line, "cloud_fraction", f[c])?,
                0.0,
                1.0,
            )?,
            None => cloud_fraction_from_kt(kt),
        };
        hours.push(HourWeather {
            t_air: Celsius(ta),
            rh_pct: rh,
            vpd: KPa(es * (1.0 - rh / 100.0)),
            global: WPerM2(g),
            direct: WPerM2(g * (1.0 - fd)),
            diffuse: WPerM2(g * fd),
            rain: Millimeters(rain),
            cloud_fraction: cloud,
            sun: sun_position(latitude, ts.year, doy, h + 0.5),
        });
    }
    Ok(WeatherSeries {
        latitude,
        start,
        hours,
        days: Vec::new(),
    })
}

/// Reads a daily weather file and disaggregates it to hours.
///
/// # Errors
/// Any [`CsvError`].
pub fn read_daily(
    text: &str,
    latitude: Degrees,
    start: DateHour,
) -> Result<WeatherSeries, CsvError> {
    let t = read_table(text)?;
    let c_d = t.col("date")?;
    let c_lo = t.col("t_min_c")?;
    let c_hi = t.col("t_max_c")?;
    let c_rh = t.col("rh_pct")?;
    let c_g = t.col("radiation_mj_m2")?;
    let c_r = t.col("rain_mm")?;
    let mut days = Vec::new();
    for (line, f) in &t.rows {
        let line = *line;
        let (year, month, day) = parse_date(line, "date", f[c_d])?;
        if (year, month, day) < (start.year, start.month, start.day) {
            continue;
        }
        let lo = check(line, "t_min_c", num(line, "t_min_c", f[c_lo])?, -60.0, 60.0)?;
        let hi = check(line, "t_max_c", num(line, "t_max_c", f[c_hi])?, lo, 60.0)?;
        days.push(DailyWeather {
            year,
            month,
            day,
            t_min: Celsius(lo),
            t_max: Celsius(hi),
            rh_mean_pct: check(line, "rh_pct", num(line, "rh_pct", f[c_rh])?, 0.0, 100.0)?,
            global: JPerM2(
                check(
                    line,
                    "radiation_mj_m2",
                    num(line, "radiation_mj_m2", f[c_g])?,
                    0.0,
                    50.0,
                )? * 1e6,
            ),
            rain: Millimeters(check(
                line,
                "rain_mm",
                num(line, "rain_mm", f[c_r])?,
                0.0,
                500.0,
            )?),
        });
    }
    Ok(WeatherSeries::from_daily(latitude, start, days))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ps_units::Hours;

    const HOURLY: &str = "\
# test file
timestamp,t_air_c,rh_pct,radiation_w_m2,rain_mm,wind_m_s
2000-05-15T00:00,12.0,80,0,0,1.0
2000-05-15T01:00,11.5,82,0,0.2,1.0
2000-05-15T02:00,11.0,85,0,0,1.0
";

    #[test]
    fn reads_hourly() {
        let start = DateHour::new(2000, 5, 15, 1).unwrap();
        let s = read_hourly(HOURLY, Degrees(45.0), start).unwrap();
        assert_eq!(s.hours.len(), 3);
        let h = s.at(Hours(0)).unwrap();
        assert!((h.t_air.0 - 11.5).abs() < 1e-12);
        assert!((h.rain.0 - 0.2).abs() < 1e-12);
        assert!(h.vpd.0 > 0.0);
    }

    #[test]
    fn hourly_gap_is_an_error() {
        let text = "timestamp,t_air_c,rh_pct,radiation_w_m2,rain_mm\n\
                    2000-05-15T00,10,50,0,0\n2000-05-15T02,10,50,0,0\n";
        let e = read_hourly(text, Degrees(45.0), DateHour::new(2000, 5, 15, 0).unwrap());
        assert!(matches!(e, Err(CsvError::Gap { line: 3, .. })), "{e:?}");
    }

    #[test]
    fn missing_column_is_named() {
        let e = read_hourly(
            "timestamp,t_air_c\n",
            Degrees(45.0),
            DateHour::new(2000, 1, 1, 0).unwrap(),
        );
        assert_eq!(e.unwrap_err().to_string(), "missing column \"rh_pct\"");
    }

    #[test]
    fn reads_daily_and_preserves_radiation() {
        let text = "date,t_min_c,t_max_c,rh_pct,radiation_mj_m2,rain_mm\n\
                    2000-06-01,12,25,60,22.5,0\n2000-06-02,13,27,55,24.0,4\n";
        let s = read_daily(text, Degrees(45.0), DateHour::new(2000, 6, 1, 0).unwrap()).unwrap();
        assert_eq!(s.hours.len(), 48);
        let day2: f64 = s.hours[24..].iter().map(|h| h.global.0 * 3600.0).sum();
        assert!((day2 - 24.0e6).abs() < 1e-3);
        let rain: f64 = s.hours.iter().map(|h| h.rain.0).sum();
        assert!((rain - 4.0).abs() < 1e-12);
    }

    #[test]
    fn out_of_range_rejected() {
        let text =
            "date,t_min_c,t_max_c,rh_pct,radiation_mj_m2,rain_mm\n2000-06-01,12,25,160,22.5,0\n";
        let e = read_daily(text, Degrees(45.0), DateHour::new(2000, 6, 1, 0).unwrap());
        assert!(matches!(
            e,
            Err(CsvError::Range {
                column: "rh_pct",
                ..
            })
        ));
    }
}
