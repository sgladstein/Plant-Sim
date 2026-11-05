use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Deserialize, Debug)] #[serde(deny_unknown_fields)]
struct Source { citation: String, doi: Option<String>, url: String, licence: String, cultivar: String, conditions: String }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct Sources { schema: u32, sources: BTreeMap<String, Source> }
#[derive(Deserialize, Debug, PartialEq)]
enum Stage { Sown, Germinated, Emerged, FirstTrifoliate, Flowering, PodSet, SeedFill, Mature }
#[derive(Deserialize, Debug)]
enum At { Final, Day(u32), Max, Stage(Stage) }
#[derive(Deserialize, Debug)]
enum Expect { Range { min: f64, max: f64 }, Approx { value: f64, rel_tol: f64 } }
#[derive(Deserialize, Debug)]
enum RExpect { Decrease, Increase, RelChange { min: f64, max: f64 }, Cited { rel_change: f64 }, FallsMoreThan(String) }
#[derive(Deserialize, Debug)] enum Gate { M0, Info, Later }
#[derive(Deserialize, Debug)] enum Tag { Encoded, PartlyEncoded, Emergent }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct Target { id: String, scenario: String, variable: String, at: At, expect: Expect, unit: String, basis: Option<String>, sources: Vec<String>, gate: Gate, note: String }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct ValidationTargets { schema: u32, species: String, targets: Vec<Target> }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct Response { id: String, scenario: String, baseline: String, variable: String, at: At, expect: RExpect, sources: Vec<String>, gate: Gate, tag: Tag, note: String }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct ValidationResponses { schema: u32, species: String, responses: Vec<Response> }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct ParamCheck { parameter: String, expect: Expect, unit: String, basis: Option<String>, sources: Vec<String>, gate: Gate, note: String }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct ParameterChecks { schema: u32, species: String, checks: Vec<ParamCheck> }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct Axis { variable: String, unit: String }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct Point { x: f64, y: f64, sd: Option<f64> }
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
struct Curve { schema: u32, id: String, source: String, conditions: String, x: Axis, y: Axis, basis: Option<String>, method: String, gate: Gate, points: Vec<Point> }

fn load<T: for<'a> Deserialize<'a>>(p: &std::path::Path) -> T {
    let s = std::fs::read_to_string(p).unwrap();
    ron::from_str(&s).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}
fn walk(d: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for e in std::fs::read_dir(d).unwrap() { let p = e.unwrap().path(); if p.is_dir() { walk(&p, out) } else { out.push(p) } }
}
fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let src: Sources = load(&dir.join("sources.ron"));
    let has = |k: &String, w: &str| assert!(src.sources.contains_key(k), "{w}: unknown source {k}");
    let t: ValidationTargets = load(&dir.join("targets.ron"));
    for x in &t.targets { for k in &x.sources { has(k, &x.id) } if let Expect::Range{min,max} = x.expect { assert!(min < max, "{}", x.id) } }
    let r: ValidationResponses = load(&dir.join("responses.ron"));
    for x in &r.responses { for k in &x.sources { has(k, &x.id) } if let RExpect::RelChange{min,max} = x.expect { assert!(min < max, "{}", x.id) } }
    let p: ParameterChecks = load(&dir.join("parameters.ron"));
    for x in &p.checks { for k in &x.sources { has(k, &x.parameter) } }
    let mut files = vec![]; walk(&dir.join("curves"), &mut files);
    let mut ids = std::collections::BTreeSet::new();
    for f in &files { let c: Curve = load(f); has(&c.source, &c.id); assert!(ids.insert(c.id.clone()), "dup {}", c.id); assert!(!c.points.is_empty());
        for w in c.points.windows(2) { assert!(w[0].x < w[1].x, "{}: x not increasing", c.id) } }
    // every source used?
    let mut used = std::collections::BTreeSet::new();
    for x in &t.targets { used.extend(x.sources.iter().cloned()) } for x in &r.responses { used.extend(x.sources.iter().cloned()) }
    for x in &p.checks { used.extend(x.sources.iter().cloned()) }
    for f in &files { let c: Curve = load(f); used.insert(c.source); }
    for k in src.sources.keys() { if !used.contains(k) { println!("unused source: {k}") } }
    println!("ok: {} sources, {} targets, {} responses, {} params, {} curves", src.sources.len(), t.targets.len(), r.responses.len(), p.checks.len(), files.len());
}
