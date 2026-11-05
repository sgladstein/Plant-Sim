//! Every CSV column has a numeric accessor and every row has one field per column.

use std::path::Path;

use ps_export::{DAILY_COLUMNS, ORGAN_COLUMNS, daily_csv, daily_number, organs_csv};
use ps_world::{Catalog, Scenario, World};

#[test]
fn columns_line_up() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let text = std::fs::read_to_string(data.join("scenarios/single_pot.ron")).unwrap();
    let mut s = Scenario::from_ron(&text).unwrap();
    s.duration_days = 12;
    let c = Catalog::load(&data, &s).unwrap();
    let mut w = World::new(s, c).unwrap();
    w.run_to_end().unwrap();
    let st = w.state();
    let row = st.history.last().unwrap();
    for col in DAILY_COLUMNS {
        assert!(daily_number(st, row, col).is_some(), "{col}");
    }
    assert!(daily_number(st, row, "nonsense").is_none());
    for (text, n) in [
        (daily_csv(st), DAILY_COLUMNS.len()),
        (organs_csv(st), ORGAN_COLUMNS.len()),
    ] {
        assert!(text.lines().count() > 2);
        for line in text.lines() {
            assert_eq!(line.split(',').count(), n, "{line}");
        }
    }
}
