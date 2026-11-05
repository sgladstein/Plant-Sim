//! WP2 done-when 1: sun position within 0.5° of a reference (pvlib NREL SPA) for
//! 10 dates × 3 latitudes. Fixture: `validation/fixtures/solar_pvlib.py`.

use ps_units::Degrees;
use ps_weather::calendar::day_of_year;
use ps_weather::solar::{day_light, sun_position};

const POSITIONS: &str = include_str!("fixtures/solar_pvlib.csv");
const DAYLIGHT: &str = include_str!("fixtures/daylight_pvlib.csv");

fn rows(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| l.split(',').collect())
}

fn date(s: &str) -> (i32, u32, u32) {
    let p: Vec<&str> = s.split('-').collect();
    (
        p[0].parse().unwrap(),
        p[1].parse().unwrap(),
        p[2].parse().unwrap(),
    )
}

fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    libm::acos(dot).to_degrees()
}

#[test]
fn sun_position_matches_spa_within_half_a_degree() {
    let mut n = 0;
    let mut worst = 0.0_f64;
    let mut dates = std::collections::BTreeSet::new();
    let mut lats = std::collections::BTreeSet::new();
    for r in rows(POSITIONS) {
        let lat: f64 = r[0].parse().unwrap();
        let (y, m, d) = date(r[1]);
        let hour: f64 = r[2].parse().unwrap();
        let elev: f64 = r[3].parse().unwrap();
        let az: f64 = r[4].parse().unwrap();
        let p = sun_position(Degrees(lat), y, day_of_year(y, m, d), hour);
        let de = (p.elevation.0 - elev).abs();
        assert!(
            de < 0.5,
            "{lat} {} {hour}: elevation {} vs {elev}",
            r[1],
            p.elevation.0
        );
        let reference = ps_weather::SunPosition {
            elevation: Degrees(elev),
            azimuth: Degrees(az),
        };
        let sep = angle_between(p.direction(), reference.direction());
        assert!(sep < 0.5, "{lat} {} {hour}: separation {sep}", r[1]);
        worst = worst.max(sep);
        n += 1;
        dates.insert(r[1].to_owned());
        lats.insert(r[0].to_owned());
    }
    assert_eq!(n, 720);
    assert_eq!(dates.len(), 10);
    assert_eq!(lats.len(), 3);
    eprintln!("worst angular separation {worst:.4} deg over {n} points");
}

#[test]
fn sunrise_and_sunset_match_spa_within_three_minutes() {
    let mut n = 0;
    for r in rows(DAYLIGHT) {
        let lat: f64 = r[0].parse().unwrap();
        let (y, m, d) = date(r[1]);
        let sr: f64 = r[2].parse().unwrap();
        let ss: f64 = r[3].parse().unwrap();
        let dl = day_light(Degrees(lat), y, day_of_year(y, m, d));
        assert!(
            (dl.sunrise_h - sr).abs() < 0.05,
            "{lat} {}: sunrise {} vs {sr}",
            r[1],
            dl.sunrise_h
        );
        assert!(
            (dl.sunset_h - ss).abs() < 0.05,
            "{lat} {}: sunset {} vs {ss}",
            r[1],
            dl.sunset_h
        );
        n += 1;
    }
    assert_eq!(n, 30);
}
