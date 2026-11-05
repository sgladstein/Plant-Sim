//! Every root organ ends inside the pot: lateral cohorts keep their tips in active soil
//! like axis segments do (decision 0028), and no root rises above the surface.

use std::path::Path;

use ps_export::{ORGAN_COLUMNS, organs_csv};
use ps_world::{Catalog, Scenario, World};

#[test]
fn root_tips_stay_in_the_pot() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let text = std::fs::read_to_string(data.join("scenarios/single_pot.ron")).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.duration_days = 70;
    let c = Catalog::load(&data, &s).unwrap();
    let mut w = World::new(s, c).unwrap();
    w.run_to_end().unwrap();
    let col = |name: &str| ORGAN_COLUMNS.iter().position(|c| *c == name).unwrap();
    let (kind, tx, ty, tz) = (col("kind"), col("tip_x"), col("tip_y"), col("tip_z"));
    // A 24 cm pot, 24 cm deep, in a grid with its corner at the origin. Active columns
    // are those whose centers fall inside the circle, so allow one cell (2 cm) of slack.
    let (r, depth, slack) = (0.12, 0.24, 0.02);
    let mut roots = 0;
    for line in organs_csv(w.state()).lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f[kind] != "Root" {
            continue;
        }
        roots += 1;
        let p: [f64; 3] = [tx, ty, tz].map(|i| f[i].parse().unwrap());
        let off2 = (p[0] - r) * (p[0] - r) + (p[1] - r) * (p[1] - r);
        assert!(
            off2 <= (r + slack) * (r + slack),
            "root tip outside the wall: {line}"
        );
        assert!(
            p[2] <= 0.0 && p[2] >= -depth - 1e-9,
            "root tip at z {}: {line}",
            p[2]
        );
    }
    assert!(roots > 100, "{roots} roots");
}
