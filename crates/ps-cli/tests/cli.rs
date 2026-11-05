//! WP10 done-when: the four §8.3 response scenarios run from one command and write CSV.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn out_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ps-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ps-cli"))
        .current_dir(root())
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn four_response_scenarios_from_one_command() {
    let out = out_dir("response");
    let names = [
        "control",
        "drought_from_flowering",
        "heat_at_flowering",
        "shade_50",
    ];
    let mut args = vec!["run".to_string()];
    for n in names {
        args.push(format!("data/scenarios/response/{n}.ron"));
    }
    args.extend(["--days", "4", "--out"].map(String::from));
    args.push(out.to_string_lossy().into_owned());
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let o = cli(&args);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    for n in names {
        let dir = out.join(n);
        let daily = std::fs::read_to_string(dir.join("daily.csv")).unwrap();
        let mut lines = daily.lines();
        let header = lines.next().unwrap();
        assert_eq!(header, ps_export::DAILY_COLUMNS.join(","));
        let rows: Vec<&str> = lines.collect();
        assert_eq!(rows.len(), 4, "{n}");
        for r in &rows {
            assert_eq!(r.split(',').count(), ps_export::DAILY_COLUMNS.len(), "{r}");
        }
        let organs = std::fs::read_to_string(dir.join("organs.csv")).unwrap();
        for r in organs.lines() {
            assert_eq!(r.split(',').count(), ps_export::ORGAN_COLUMNS.len(), "{r}");
        }
        assert_eq!(
            std::fs::read_to_string(dir.join("hashes.csv"))
                .unwrap()
                .lines()
                .count(),
            5
        );
        assert!(dir.join("log.ron").exists());
    }
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn snapshot_resume_and_replay_give_the_run_hash() {
    let out = out_dir("resume");
    let o_s = out.to_string_lossy().into_owned();
    let o = cli(&[
        "run",
        "data/scenarios/single_pot.ron",
        "--days",
        "6",
        "--snapshot-day",
        "3",
        "--out",
        &o_s,
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let hashes = std::fs::read_to_string(out.join("single_pot/hashes.csv")).unwrap();
    let last = hashes
        .lines()
        .last()
        .unwrap()
        .split(',')
        .nth(1)
        .unwrap()
        .to_string();

    let snap = out.join("single_pot/snapshot_day3.ron");
    let o = cli(&["replay", "--snapshot", &snap.to_string_lossy()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let text = String::from_utf8_lossy(&o.stdout);
    assert!(text.contains(&format!("daily hash {last}")), "{text}");
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn unknown_options_fail() {
    let o = cli(&["run", "data/scenarios/single_pot.ron", "--dayz", "3"]);
    assert!(!o.status.success());
}

#[test]
fn a_missing_golden_file_fails_unless_allowed() {
    let empty = out_dir("golden");
    std::fs::create_dir_all(&empty).unwrap();
    let e = empty.to_string_lossy().into_owned();
    let args = [
        "hash",
        "data/scenarios/single_pot.ron",
        "--days",
        "1",
        "--check-golden",
        &e,
    ];
    assert!(!cli(&args).status.success());
    let mut allowed = args.to_vec();
    allowed.push("--allow-missing");
    assert!(cli(&allowed).status.success());
    let _ = std::fs::remove_dir_all(&empty);
}

/// WP11 done-when: the report fails the build when a target is missed, and a listed
/// known miss does not.
#[test]
fn validate_fails_on_a_missed_target() {
    let fixtures = root().join("crates/ps-cli/tests/fixtures");
    let v = fixtures.join("validation");
    let out = out_dir("validate");
    let o_s = out.to_string_lossy().into_owned();
    let v_s = v.to_string_lossy().into_owned();
    let none = out.join("none.ron").to_string_lossy().into_owned();
    let o = cli(&[
        "validate",
        "--validation",
        &v_s,
        "--known",
        &none,
        "--out",
        &o_s,
    ]);
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("MISS impossible_height"), "{err}");
    let report = std::fs::read_to_string(out.join("report.md")).unwrap();
    assert!(report.contains("| pass | pods_in_range"), "{report}");

    let known = fixtures.join("known_impossible.ron");
    let o = cli(&[
        "validate",
        "--validation",
        &v_s,
        "--known",
        &known.to_string_lossy(),
        "--out",
        &o_s,
    ]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(
        out.join("runs/response/heat_at_flowering/daily.csv")
            .exists()
    );
    assert!(out.join("runs/single_pot/organs_day45.csv").exists());
    let _ = std::fs::remove_dir_all(&out);
}
