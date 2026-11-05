//! WP7 done-when 1: gas exchange within 2% of the plantecophys fixture over the test
//! grid of design plan §8.1 (PAR 50 to 2000 × T 10 to 40 °C × VPD 0.5 to 4 kPa × CO2
//! 280, 420, 800 ppm).

use std::fmt::Write as _;

use ps_physio::{LeafEnv, LeafParams, gas};
use ps_species::Species;
use ps_units::{Celsius, KPa, Ppm, UmolPerM2S};

const FIXTURE: &str = include_str!("fixtures/photosyn.csv");

#[test]
fn gas_exchange_matches_plantecophys() {
    let s = Species::from_ron(include_str!("../../../data/species/bush_bean.ron")).unwrap();
    let t = s.phenotype(&s.default_genotype()).traits().unwrap();
    let p = LeafParams::from_species(&s, &t);
    let mut worst = [0.0_f64; 3];
    let mut fails = String::new();
    let mut n = 0;
    for line in FIXTURE.lines().filter(|l| !l.starts_with('#')).skip(1) {
        let v: Vec<f64> = line.split(',').map(|x| x.parse().unwrap()).collect();
        let env = LeafEnv {
            par: UmolPerM2S(v[0]),
            t_leaf: Celsius(v[1]),
            vpd: KPa(v[2]),
            ca: Ppm(v[3]),
            patm: KPa(100.0),
        };
        let g = gas::solve(&p, &env, 1.0, 1.0);
        let got = [g.a_net.0, g.gs.0, g.e.0];
        let want = [v[4], v[5], v[6]];
        for k in 0..3 {
            let rel = (got[k] - want[k]).abs() / want[k].abs().max(1e-9);
            worst[k] = worst[k].max(rel);
            if rel > 0.02 {
                writeln!(fails, "{line}: got {got:?}").unwrap();
            }
        }
        n += 1;
    }
    println!(
        "{n} cases; worst relative error A {:.4}, gs {:.4}, E {:.4}",
        worst[0], worst[1], worst[2]
    );
    assert_eq!(n, 1008);
    assert!(fails.is_empty(), "outside 2%:\n{fails}");
}
