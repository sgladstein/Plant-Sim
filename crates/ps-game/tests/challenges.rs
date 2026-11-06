//! Each shipped challenge replays to the same score, and a known strategy earns its
//! medal (WP32 done-when: one challenge playable with a reproducible score).

use std::path::PathBuf;

use ps_game::{Factor, Medal, RuleBreak, load_challenge, load_debrief_refs, play};
use ps_world::{Action, ActionKind, SeedChoice, Sowing};

fn data() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data"))
}

fn sow(day: u64) -> Action {
    Action {
        tick: day * 24 + 8,
        kind: ActionKind::Sow(Sowing {
            soil: 0,
            species: "bush_bean".into(),
            seed: SeedChoice::Default,
            depth_cm: 3.0,
            offset_cm: None,
        }),
    }
}

#[test]
fn heat_wave_scores_reproducibly_and_names_the_heat() {
    let c = load_challenge(&data(), "heat_wave").expect("challenge");
    // Sown on day 4, the bean blooms into the heat wave.
    let log = [sow(4)];
    let (w, a) = play(&data(), &c, &log).expect("play");
    let (_, b) = play(&data(), &c, &log).expect("replay");
    assert_eq!(a, b, "a replay gives the same outcome");
    assert!(a.breaks.is_empty(), "{:?}", a.breaks);
    assert!(a.medal < Some(Medal::Gold));
    let d = ps_game::debrief(w.state(), &load_debrief_refs(&data()).expect("refs"));
    let p = &d.plants[0];
    assert_eq!(p.ranked.first(), Some(&Factor::Heat), "{:?}", p.summary);
    assert!(p.summary[0].contains("during bloom"), "{:?}", p.summary);
}

#[test]
fn rules_are_enforced() {
    let c = load_challenge(&data(), "heat_wave").expect("challenge");
    let log = [
        sow(45),
        Action {
            tick: 50 * 24,
            kind: ActionKind::Water { soil: 0, mm: 5.0 },
        },
    ];
    let scenario = ps_game::load_scenario(&data(), &c).expect("scenario");
    let (breaks, _) = c.check_log(&scenario, &log);
    assert!(breaks.contains(&RuleBreak::SowOutsideWindow { tick: 45 * 24 + 8 }));
    assert!(
        breaks
            .iter()
            .any(|b| matches!(b, RuleBreak::NotAllowed { .. }))
    );
}

#[test]
fn water_budget_counts_litres() {
    let c = load_challenge(&data(), "thirsty_summer").expect("challenge");
    let scenario = ps_game::load_scenario(&data(), &c).expect("scenario");
    let area = ps_game::soil_area(&scenario, 0).expect("pot").0;
    // 7 L in one go: over the 6 L budget.
    let log = [Action {
        tick: 40 * 24,
        kind: ActionKind::Water {
            soil: 0,
            mm: 7.0 / area,
        },
    }];
    let (breaks, used) = c.check_log(&scenario, &log);
    assert!((used.water.0 - 7.0).abs() < 1e-9);
    assert!(matches!(breaks[..], [RuleBreak::OverWaterBudget { .. }]));
}

fn fixture(name: &str) -> Vec<Action> {
    let p = format!(
        "{}/../../tests/fixtures/challenges/{name}.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    ron::from_str(&std::fs::read_to_string(p).expect("fixture")).expect("parse")
}

#[test]
fn thirsty_summer_rewards_water_in_bloom_and_names_the_drought() {
    let c = load_challenge(&data(), "thirsty_summer").expect("challenge");
    let refs = load_debrief_refs(&data()).expect("refs");
    let (_, bloom) = play(&data(), &c, &fixture("thirsty_summer_bloom")).expect("bloom");
    let (w, spread) = play(&data(), &c, &fixture("thirsty_summer_spread")).expect("spread");
    assert!(bloom.breaks.is_empty() && spread.breaks.is_empty());
    assert!(bloom.score > spread.score, "{bloom:?} {spread:?}");
    assert!(bloom.medal >= Some(Medal::Bronze), "{bloom:?}");
    let d = ps_game::debrief(w.state(), &refs);
    assert_eq!(
        d.plants[0].ranked.first(),
        Some(&Factor::Water),
        "{:?}",
        d.plants[0].summary
    );
}
