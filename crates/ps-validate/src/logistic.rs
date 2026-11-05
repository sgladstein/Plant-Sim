//! A three-parameter logistic fit `y = K / (1 + exp(−r (t − t₀)))` for the §8.3 shape
//! rule ("the biomass curve passes if a logistic fit has R² above 0.95"). For each
//! `(r, t₀)` on a grid, the best `K` is closed-form least squares; the grid is refined
//! twice around the best point.

use ps_world::WorldState;

use crate::plant_rows;

/// A fitted logistic.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Logistic {
    /// Asymptote.
    pub k: f64,
    /// Rate, per day.
    pub r: f64,
    /// Midpoint, days.
    pub t0: f64,
    /// Coefficient of determination.
    pub r2: f64,
}

fn shape(t: f64, r: f64, t0: f64) -> f64 {
    1.0 / (1.0 + libm::exp(-r * (t - t0)))
}

fn score(x: &[f64], y: &[f64], r: f64, t0: f64) -> (f64, f64) {
    let (mut sfy, mut sff) = (0.0, 0.0);
    for (&t, &v) in x.iter().zip(y) {
        let f = shape(t, r, t0);
        sfy += f * v;
        sff += f * f;
    }
    let k = if sff > 0.0 { sfy / sff } else { 0.0 };
    let sse = x
        .iter()
        .zip(y)
        .map(|(&t, &v)| {
            let e = v - k * shape(t, r, t0);
            e * e
        })
        .sum();
    (k, sse)
}

/// Fits the logistic to `(x, y)`.
#[must_use]
#[allow(clippy::cast_precision_loss)] // grid indices and point counts are small
pub fn fit(x: &[f64], y: &[f64]) -> Logistic {
    let n = y.len().max(1) as f64;
    let mean = y.iter().sum::<f64>() / n;
    let sst: f64 = y.iter().map(|v| (v - mean) * (v - mean)).sum();
    let (xmin, xmax) = x
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    let (mut r_lo, mut r_hi) = (0.01, 1.0);
    let (mut t_lo, mut t_hi) = (xmin, xmax);
    let mut best = (f64::MAX, 0.0, 0.0, 0.0);
    for _ in 0..3 {
        const N: usize = 40;
        for i in 0..=N {
            let r = r_lo + (r_hi - r_lo) * i as f64 / N as f64;
            for j in 0..=N {
                let t0 = t_lo + (t_hi - t_lo) * j as f64 / N as f64;
                let (k, sse) = score(x, y, r, t0);
                if sse < best.0 {
                    best = (sse, k, r, t0);
                }
            }
        }
        let (dr, dt) = ((r_hi - r_lo) / 10.0, (t_hi - t_lo) / 10.0);
        r_lo = (best.2 - dr).max(1e-4);
        r_hi = best.2 + dr;
        t_lo = best.3 - dt;
        t_hi = best.3 + dt;
    }
    Logistic {
        k: best.1,
        r: best.2,
        t0: best.3,
        r2: if sst > 0.0 { 1.0 - best.0 / sst } else { 0.0 },
    }
}

/// The time course of `y` against `x` for the first plant, skipping days where either is
/// undefined (a zero denominator). `x` is `day` or any variable.
///
/// # Errors
/// If a variable is unknown, or no day has both values.
pub fn series(state: &WorldState, x: &str, y: &str) -> Result<(Vec<f64>, Vec<f64>), String> {
    let first = state.plants.first().map_or(0, |p| p.id);
    let rows = plant_rows(state, first);
    let mut xs = Vec::with_capacity(rows.len());
    let mut ys = Vec::with_capacity(rows.len());
    for i in 0..rows.len() {
        let xv = crate::derived::row_value(state, &rows, i, x);
        let yv = crate::derived::row_value(state, &rows, i, y);
        match (xv, yv) {
            (Ok(a), Ok(b)) => {
                xs.push(a);
                ys.push(b);
            }
            (Err(e), _) | (_, Err(e)) if e.starts_with("unknown") => return Err(e),
            _ => {}
        }
    }
    if xs.is_empty() {
        return Err(format!("{y} against {x} has no defined day"));
    }
    Ok((xs, ys))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_known_logistic() {
        let x: Vec<f64> = (0..90).map(f64::from).collect();
        let y: Vec<f64> = x.iter().map(|&t| 30.0 * shape(t, 0.12, 45.0)).collect();
        let f = fit(&x, &y);
        assert!(f.r2 > 0.9999, "{f:?}");
        assert!(
            (f.k - 30.0).abs() < 0.3 && (f.t0 - 45.0).abs() < 0.5,
            "{f:?}"
        );
    }

    #[test]
    fn a_straight_line_fits_worse_than_a_logistic() {
        let x: Vec<f64> = (0..90).map(f64::from).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|&t| if t < 45.0 { 1.0 } else { 20.0 })
            .collect();
        assert!(fit(&x, &y).r2 < 0.999);
    }
}
