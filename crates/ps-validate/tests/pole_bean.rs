//! WP22 done-when (design plan §7, decision 0021): a staked pole bean twines up its
//! support; without one it falls over and yields less.

use std::path::{Path, PathBuf};

use ps_arch::Climb;
use ps_validate::{At, replicate_mean};
use ps_world::{Catalog, Scenario, World};

fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn mean(w: &World, variable: &str, at: &At) -> f64 {
    replicate_mean(w.state(), variable, at).unwrap().value
}

/// The world seeds the validation report averages over (decision 0026).
const SEEDS: [u64; 4] = [7, 8, 9, 10];

/// Runs `name` at world seed `seed`, to its end.
fn run_seed(name: &str, seed: u64) -> World {
    let text = std::fs::read_to_string(data().join("scenarios").join(name)).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.world_seed = seed;
    let c = Catalog::load(&data(), &s).unwrap();
    let mut w = World::new(s, c).unwrap();
    w.run_to_end().unwrap();
    w
}

/// In a row of 8, every vine on a string catches it, coils its main stem around it,
/// and climbs past 1.5 m; in the same row without strings they fall over and stay
/// under 30 cm, and yield much less: -30 to -70% in the field trials (IITA, Reckling;
/// decision 0021). One world seed spans about -15 to -80%, so the yield reads the mean
/// over the validation report's four seeds (decisions 0026 and 0032), and every plant
/// of every seed must climb or lodge.
#[test]
fn staked_pole_beans_climb_and_unstaked_ones_lodge() {
    let runs: Vec<(World, World)> = std::thread::scope(|scope| {
        let jobs: Vec<_> = SEEDS
            .iter()
            .map(|&seed| {
                let staked = scope.spawn(move || run_seed("m2/pole_row_trellis.ron", seed));
                let unstaked = scope.spawn(move || run_seed("m2/pole_row_no_trellis.ron", seed));
                (staked, unstaked)
            })
            .collect();
        jobs.into_iter()
            .map(|(a, b)| (a.join().unwrap(), b.join().unwrap()))
            .collect()
    });
    let (mut tall, mut flat, mut a, mut b) = (0.0, 0.0, 0.0, 0.0);
    for (seed, (staked, unstaked)) in SEEDS.iter().zip(&runs) {
        for p in &staked.state().plants {
            let plant = &p.plant;
            let main = &plant.axes[0];
            assert!(
                matches!(main.climb, Climb::Twining(_) | Climb::Topped),
                "seed {seed} plant {} main stem {:?}",
                p.id,
                main.climb
            );
            let coiled = plant.meta.iter().filter(|m| m.twine.is_some()).count();
            assert!(
                coiled >= 10,
                "seed {seed} plant {} has {coiled} coiled internodes",
                p.id
            );
        }
        for p in &unstaked.state().plants {
            assert_eq!(
                p.plant.axes[0].climb,
                Climb::Lodged,
                "seed {seed} plant {}",
                p.id
            );
            assert!(p.plant.meta.iter().all(|m| m.twine.is_none()));
        }
        tall += mean(staked, "height_m", &At::Final);
        flat += mean(unstaked, "height_m", &At::Final);
        a += mean(staked, "seed_mass_g", &At::Final);
        b += mean(unstaked, "seed_mass_g", &At::Final);
    }
    #[allow(clippy::cast_precision_loss)]
    let n = SEEDS.len() as f64;
    let (tall, flat) = (tall / n, flat / n);
    assert!(tall > 1.5, "staked height {tall:.2} m");
    assert!(flat < 0.3, "unstaked height {flat:.2} m");
    let change = b / a - 1.0;
    assert!(
        (-0.7..=-0.3).contains(&change),
        "unstaked seed {:.2} g vs staked {:.2} g ({:+.0}%)",
        b / n,
        a / n,
        change * 100.0
    );
}

/// Supports change nothing for a determinate bush bean.
#[test]
fn bush_beans_ignore_supports() {
    let text = std::fs::read_to_string(data().join("scenarios/single_pot.ron")).unwrap();
    let plain = Scenario::from_ron(&text).unwrap();
    let staked_text = text.replace(
        "  duration_days: 90,",
        "  supports: [ (soil: 0, offset_cm: (4.0, 0.0), height_cm: 100.0, diameter_cm: 1.0) ],\n  duration_days: 90,",
    );
    let staked = Scenario::from_ron(&staked_text).unwrap();
    let go = |mut s: Scenario| {
        s.duration_days = 50;
        let c = Catalog::load(&data(), &s).unwrap();
        let mut w = World::new(s, c).unwrap();
        w.run_to_end().unwrap();
        w.state().plants[0].plant.report()
    };
    assert_eq!(go(plain), go(staked));
}
