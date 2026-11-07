//! `ps-cli fidelity`: the detailed and simplified water network compared (R3c,
//! decision 0045; docs/research/realism-plan/fidelity-levels.md §5.1).
//!
//! Each scenario runs on four world seeds at both levels. An outcome passes when the
//! gap between the two levels' seed means is at most the larger of 5% of the detailed
//! mean and half the detailed level's seed-to-seed range. Across the scenarios the mean
//! signed gap in seed yield must stay within ±2% (the bias check: the player picks the
//! level by watching, so a systematic gap would make watching a strategy), or within
//! twice its own standard error from the seeds' paired gaps: a level switch reshuffles
//! pod set, so one seed's gap runs to ±30% in a pot and four seeds cannot resolve 2%
//! (decision 0045). A switch run (detailed, simplified from day 30, detailed again from
//! day 60) must pass the same rule against the detailed run.

use std::fmt::Write as _;

use ps_graph::OrganKind;
use ps_physio::Stage;
use ps_world::{Fidelity, HydraulicsLevel, WorldState};

/// Scenarios of the comparison, by path under `data/scenarios/` without `.ron`.
pub const SCENARIOS: [&str; 6] = [
    "single_pot",
    "response/drought_from_flowering",
    "m2/split_pot",
    "plot_30_beans",
    "m2/plot_low_n",
    "m2/wheat_stand",
];

/// The switch run: `single_pot` with `SetFidelity` events at days 30 and 60.
pub const SWITCH_SCENARIO: &str = "fidelity_switch";

/// World seed offsets the means run over (the validation's).
pub const SEED_OFFSETS: [u64; 4] = [0, 1, 2, 3];

/// Largest allowed gap as a share of the detailed mean (or half the seed range).
const SHARE: f64 = 0.05;
/// Largest allowed mean signed gap in seed yield across scenarios.
const BIAS: f64 = 0.02;
/// Depth splitting shallow from deep root mass, m.
const ROOT_SPLIT_DEPTH: f64 = 0.2;

/// Outcome names, in report order.
pub const OUTCOMES: [&str; 16] = [
    "total dry mass, g",
    "leaf mass, g",
    "stem mass, g",
    "root mass, g",
    "seed yield, g",
    "pods",
    "height, m",
    "water use, kg",
    "water use efficiency, g/kg",
    "root mass 0-20 cm, g",
    "root mass below 20 cm, g",
    "N uptake, g",
    "plant N, g",
    "flowering day",
    "maturity day",
    "lowest leaf potential, MPa",
];

/// Index of the seed yield in [`OUTCOMES`].
const SEED_YIELD: usize = 4;

/// The scenario set as each level runs it.
#[must_use]
pub fn forced(level: HydraulicsLevel) -> Fidelity {
    match level {
        HydraulicsLevel::Detailed => Fidelity::Detailed,
        HydraulicsLevel::Simplified => Fidelity::Simplified,
    }
}

/// One run's outcomes, per plant (the mean over its plants), in [`OUTCOMES`] order;
/// `None` where no plant got there (a date never reached).
#[must_use]
pub fn outcomes(st: &WorldState) -> Vec<Option<f64>> {
    let n = st.plants.len().max(1);
    #[allow(clippy::cast_precision_loss)] // a plant count
    let per = |v: f64| v / n as f64;
    let mut sums = [0.0_f64; OUTCOMES.len()];
    let mut flowered = (0.0, 0_usize);
    let mut matured = (0.0, 0_usize);
    let mut psi_min = 0.0_f64;
    for (k, p) in st.plants.iter().enumerate() {
        let rows: Vec<_> = st
            .history
            .iter()
            .filter(|r| r.plant as usize == k)
            .collect();
        let Some(last) = rows.last().map(|r| &r.report) else {
            continue;
        };
        sums[0] += last.total_dry_mass;
        sums[1] += last.leaf_mass;
        sums[2] += last.stem_mass;
        sums[3] += last.root_mass;
        sums[4] += rows.iter().map(|r| r.report.seed_mass).fold(0.0, f64::max);
        sums[5] += rows
            .iter()
            .map(|r| f64::from(r.report.pods))
            .fold(0.0, f64::max);
        sums[6] += rows.iter().map(|r| r.report.height).fold(0.0, f64::max);
        let water: f64 = rows.iter().map(|r| r.report.transpiration).sum();
        sums[7] += water;
        if water > 0.0 {
            sums[8] += last.total_dry_mass / water;
        }
        let (shallow, deep) = root_mass_by_depth(&p.plant);
        sums[9] += shallow;
        sums[10] += deep;
        sums[11] += rows.iter().map(|r| r.report.n_uptake).sum::<f64>();
        sums[12] += last.nitrogen;
        let first = |s: Stage| {
            rows.iter()
                .find(|r| r.report.stage.is_some_and(|x| x >= s))
                .map(|r| f64::from(r.day))
        };
        if let Some(d) = first(Stage::Flowering) {
            flowered = (flowered.0 + d, flowered.1 + 1);
        }
        if let Some(d) = first(Stage::Mature) {
            matured = (matured.0 + d, matured.1 + 1);
        }
        psi_min += rows
            .iter()
            .map(|r| r.report.leaf_psi_min)
            .fold(0.0, f64::min);
    }
    #[allow(clippy::cast_precision_loss)] // plant counts
    let mean_day = |(sum, count): (f64, usize)| (count > 0).then(|| sum / count as f64);
    let mut out: Vec<Option<f64>> = sums.iter().map(|&s| Some(per(s))).collect();
    out[13] = mean_day(flowered);
    out[14] = mean_day(matured);
    out[15] = Some(per(psi_min));
    out
}

/// A plant's living root structure above and below [`ROOT_SPLIT_DEPTH`], g, by each
/// root organ's middle.
fn root_mass_by_depth(plant: &ps_arch::Plant) -> (f64, f64) {
    let a = &plant.arena;
    let (mut shallow, mut deep) = (0.0, 0.0);
    for id in a.alive_of(OrganKind::Root) {
        let m = a.structural(id).0;
        let depth = plant
            .positions
            .get(id.index())
            .map_or(0.0, |p| -0.5 * (p.0[2] + p.1[2]));
        if depth <= ROOT_SPLIT_DEPTH {
            shallow += m;
        } else {
            deep += m;
        }
    }
    (shallow, deep)
}

/// The seed mean and the seed-to-seed range of one outcome.
fn mean_range(values: &[Option<f64>]) -> Option<(f64, f64)> {
    let v: Vec<f64> = values.iter().copied().collect::<Option<_>>()?;
    if v.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)] // a seed count
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    let lo = v.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    Some((mean, hi - lo))
}

/// One compared outcome.
#[derive(Clone, Debug)]
pub struct Row {
    /// Scenario (or the switch run).
    pub scenario: String,
    /// Outcome name.
    pub outcome: &'static str,
    /// Detailed seed mean.
    pub detailed: f64,
    /// Simplified (or switched) seed mean.
    pub other: f64,
    /// Largest allowed gap.
    pub allowed: f64,
    /// Standard error of the gap from the seeds' paired gaps (0 when the seeds don't
    /// pair).
    pub gap_se: f64,
}

impl Row {
    /// Whether the gap is within the allowed one.
    #[must_use]
    pub fn passes(&self) -> bool {
        (self.other - self.detailed).abs() <= self.allowed
    }
}

/// Compares one scenario's runs: `detailed` and `other` hold one outcome list per seed.
#[must_use]
pub fn compare(
    scenario: &str,
    detailed: &[Vec<Option<f64>>],
    other: &[Vec<Option<f64>>],
) -> Vec<Row> {
    let column = |runs: &[Vec<Option<f64>>], k: usize| -> Vec<Option<f64>> {
        runs.iter().map(|r| r.get(k).copied().flatten()).collect()
    };
    let mut rows = Vec::new();
    for (k, &name) in OUTCOMES.iter().enumerate() {
        let (Some((d, range)), Some((s, _))) = (
            mean_range(&column(detailed, k)),
            mean_range(&column(other, k)),
        ) else {
            continue;
        };
        rows.push(Row {
            scenario: scenario.to_string(),
            outcome: name,
            detailed: d,
            other: s,
            allowed: (SHARE * d.abs()).max(0.5 * range),
            gap_se: paired_se(&column(detailed, k), &column(other, k)),
        });
    }
    rows
}

/// Standard error of the mean paired gap `other_k − detailed_k` over seeds `k`; 0 when
/// the lists don't pair or hold fewer than two seeds.
fn paired_se(detailed: &[Option<f64>], other: &[Option<f64>]) -> f64 {
    let gaps: Option<Vec<f64>> = detailed
        .iter()
        .zip(other)
        .map(|(d, o)| Some((*o)? - (*d)?))
        .collect();
    let Some(gaps) = gaps.filter(|g| g.len() >= 2 && detailed.len() == other.len()) else {
        return 0.0;
    };
    #[allow(clippy::cast_precision_loss)] // a seed count
    let n = gaps.len() as f64;
    let mean = gaps.iter().sum::<f64>() / n;
    let var = gaps.iter().map(|g| (g - mean) * (g - mean)).sum::<f64>() / (n - 1.0);
    libm::sqrt(var / n)
}

/// The bias check: the mean over scenarios of the signed seed-yield gap, as a share of
/// the detailed yield, over scenarios that set seed, and its standard error.
#[must_use]
pub fn seed_bias(rows: &[Row]) -> Option<(f64, f64)> {
    let gaps: Vec<(f64, f64)> = rows
        .iter()
        .filter(|r| r.outcome == OUTCOMES[SEED_YIELD] && r.detailed > 0.0)
        .filter(|r| !r.scenario.ends_with("(switch)"))
        .map(|r| ((r.other - r.detailed) / r.detailed, r.gap_se / r.detailed))
        .collect();
    #[allow(clippy::cast_precision_loss)] // a scenario count
    let n = gaps.len() as f64;
    let mean = gaps.iter().map(|g| g.0).sum::<f64>() / n;
    let se = libm::sqrt(gaps.iter().map(|g| g.1 * g.1).sum::<f64>()) / n;
    (!gaps.is_empty()).then_some((mean, se))
}

/// Whether a bias fails: beyond ±2% and beyond twice its standard error.
#[must_use]
pub fn bias_fails((bias, se): (f64, f64)) -> bool {
    bias.abs() > BIAS && bias.abs() > 2.0 * se
}

/// The report as Markdown, and the number of failures (outcomes and the bias check).
#[must_use]
pub fn report(rows: &[Row]) -> (String, usize) {
    let mut md = String::new();
    let failed = rows.iter().filter(|r| !r.passes()).count();
    let bias = seed_bias(rows);
    let bias_failed = bias.is_some_and(bias_fails);
    let _ = writeln!(
        md,
        "# Fidelity: detailed against simplified water network\n"
    );
    let _ = writeln!(
        md,
        "{} of {} outcomes pass; seed-yield bias {} (passes within ±{:.0}% or twice its standard error){}.\n",
        rows.len() - failed,
        rows.len(),
        bias.map_or_else(
            || "-".into(),
            |(b, se)| format!("{:+.2}% ± {:.2}%", b * 100.0, se * 100.0)
        ),
        BIAS * 100.0,
        if bias_failed { ": **FAIL**" } else { "" }
    );
    let _ = writeln!(
        md,
        "Four-seed means. An outcome passes when the gap is at most the larger of {:.0}% of the detailed mean and half its seed range.\n",
        SHARE * 100.0
    );
    let _ = writeln!(
        md,
        "| scenario | outcome | detailed | simplified | gap | allowed | |\n|---|---|---:|---:|---:|---:|---|"
    );
    for r in rows {
        let gap = if r.detailed.abs() > 0.0 {
            format!("{:+.1}%", (r.other - r.detailed) / r.detailed.abs() * 100.0)
        } else {
            format!("{:+.3}", r.other - r.detailed)
        };
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {gap} | {} | {} |",
            r.scenario,
            r.outcome,
            ps_validate::fmt_num(r.detailed),
            ps_validate::fmt_num(r.other),
            ps_validate::fmt_num(r.allowed),
            if r.passes() { "pass" } else { "**FAIL**" }
        );
    }
    (md, failed + usize::from(bias_failed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runs(v: &[f64]) -> Vec<Vec<Option<f64>>> {
        v.iter()
            .map(|&x| {
                let mut r = vec![Some(0.0); OUTCOMES.len()];
                r[SEED_YIELD] = Some(x);
                r
            })
            .collect()
    }

    #[test]
    fn the_rule_takes_the_larger_of_five_percent_and_half_the_range() {
        // Detailed mean 10, range 4: allowed 2 (half the range beats 0.5).
        let rows = compare("s", &runs(&[8.0, 10.0, 12.0, 10.0]), &runs(&[11.9; 4]));
        let r = rows
            .iter()
            .find(|r| r.outcome == OUTCOMES[SEED_YIELD])
            .expect("row");
        assert!((r.allowed - 2.0).abs() < 1e-12 && r.passes());
        // A tight detailed level: range 0, allowed 5%.
        let rows = compare("s", &runs(&[10.0; 4]), &runs(&[10.6; 4]));
        let r = rows
            .iter()
            .find(|r| r.outcome == OUTCOMES[SEED_YIELD])
            .expect("row");
        assert!((r.allowed - 0.5).abs() < 1e-12 && !r.passes());
    }

    #[test]
    fn bias_reads_seed_yield_over_scenarios_that_set_seed() {
        let mut rows = compare("a", &runs(&[10.0; 4]), &runs(&[10.3; 4]));
        rows.extend(compare("b", &runs(&[10.0; 4]), &runs(&[9.9; 4])));
        rows.extend(compare("c", &runs(&[0.0; 4]), &runs(&[0.0; 4])));
        rows.extend(compare("a (switch)", &runs(&[10.0; 4]), &runs(&[20.0; 4])));
        let (b, se) = seed_bias(&rows).expect("bias");
        assert!((b - 0.01).abs() < 1e-12, "{b}");
        assert!(se.abs() < 1e-12, "identical seeds have no spread: {se}");
        assert!(!bias_fails((b, se)));
        assert!(bias_fails((0.03, 0.01)));
        assert!(!bias_fails((0.03, 0.02)), "inside twice its standard error");
    }
}
