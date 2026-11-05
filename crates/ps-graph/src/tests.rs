use super::*;
use proptest::prelude::*;
use ps_units::{M2PerG, MPerG};

fn allom() -> Allometry {
    Allometry::generic_herb()
}

fn organ(kind: OrganKind, parent: Option<OrganId>) -> NewOrgan {
    NewOrgan {
        kind,
        parent,
        axis_order: 0,
        birth: Hours(0),
        curve: ExpansionCurve::new(DegreeDays(100.0), DegreeDays(60.0), GramsDM(1.0)).unwrap(),
        specific: match kind {
            OrganKind::Leaf | OrganKind::Cotyledon => SpecificSize::Area(M2PerG(0.025)),
            OrganKind::Internode | OrganKind::Root => SpecificSize::Length(MPerG(2.0)),
            _ => SpecificSize::Bulk,
        },
        orientation: Orientation::UP,
        structural: GramsDM(0.01),
        reserve: GramsC(0.001),
        nitrogen: GramsN(0.0),
    }
}

#[test]
fn children_in_creation_order_and_edges() {
    let a = allom();
    let mut g = OrganArena::new();
    let seed = g.add(&organ(OrganKind::Seed, None), &a).unwrap();
    let hyp = g.add(&organ(OrganKind::Internode, Some(seed)), &a).unwrap();
    let root = g.add(&organ(OrganKind::Root, Some(seed)), &a).unwrap();
    let leaf = g.add(&organ(OrganKind::Leaf, Some(hyp)), &a).unwrap();
    assert_eq!(g.children[seed.index()], vec![hyp, root]);
    assert_eq!(
        g.edges().collect::<Vec<_>>(),
        vec![(seed, hyp), (seed, root), (hyp, leaf)]
    );
    assert_eq!(g.subtree(seed), vec![seed, hyp, leaf, root]);
}

#[test]
fn dead_parent_rejected_and_edges_drop() {
    let a = allom();
    let mut g = OrganArena::new();
    let s = g.add(&organ(OrganKind::Seed, None), &a).unwrap();
    let i = g.add(&organ(OrganKind::Internode, Some(s)), &a).unwrap();
    let l = g.add(&organ(OrganKind::Leaf, Some(i)), &a).unwrap();
    let killed = g.kill_subtree(i, Hours(10)).unwrap();
    assert_eq!(killed.iter().map(|k| k.0).collect::<Vec<_>>(), vec![i, l]);
    assert_eq!(
        g.add(&organ(OrganKind::Leaf, Some(i)), &a),
        Err(GraphError::Dead(i))
    );
    assert_eq!(g.edges().count(), 0);
    assert_eq!(g.death[l.index()], Some(Hours(10)));
}

#[test]
fn negative_growth_rejected() {
    let a = allom();
    let mut g = OrganArena::new();
    let s = g.add(&organ(OrganKind::Seed, None), &a).unwrap();
    assert!(matches!(
        g.grow(s, GramsDM(-0.1), &a),
        Err(GraphError::NegativeGrowth { .. })
    ));
    assert!(matches!(
        g.grow(s, GramsDM(f64::NAN), &a),
        Err(GraphError::NegativeGrowth { .. })
    ));
    assert!(g.grow(OrganId(99), GramsDM(0.1), &a).is_err());
}

#[test]
fn growth_updates_size() {
    let a = allom();
    let mut g = OrganArena::new();
    let s = g.add(&organ(OrganKind::Seed, None), &a).unwrap();
    let l = g.add(&organ(OrganKind::Leaf, Some(s)), &a).unwrap();
    g.grow(l, GramsDM(0.29), &a).unwrap();
    assert!((g.size[l.index()].area.0 - 0.3 * 0.025).abs() < 1e-15);
}

#[test]
fn unmet_demand_shrinks_effective_potential() {
    let a = allom();
    let mut g = OrganArena::new();
    let s = g.add(&organ(OrganKind::Seed, None), &a).unwrap();
    assert_eq!(g.effective_potential(s), GramsDM(1.0));
    g.record_demand(s, GramsC(0.4), GramsC(0.1));
    assert!((g.effective_potential(s).0 - 0.75).abs() < 1e-15);
}

#[test]
fn hash_visit_changes_with_state() {
    let a = allom();
    let mut g = OrganArena::new();
    let s = g.add(&organ(OrganKind::Seed, None), &a).unwrap();
    let collect = |g: &OrganArena| {
        let mut v = Vec::new();
        g.visit_bits(|x| v.push(x));
        v
    };
    let before = collect(&g);
    g.grow(s, GramsDM(1e-12), &a).unwrap();
    assert_ne!(before, collect(&g));
}

#[derive(Clone, Debug)]
enum Op {
    Add { parent: usize, kind: u8 },
    Grow { organ: usize, delta: f64 },
    Kill { organ: usize },
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (any::<usize>(), 0u8..4).prop_map(|(parent, kind)| Op::Add { parent, kind }),
        (any::<usize>(), -1.0f64..1.0).prop_map(|(organ, delta)| Op::Grow { organ, delta }),
        any::<usize>().prop_map(|organ| Op::Kill { organ }),
    ]
}

proptest! {
    /// WP3 done-when: structural mass never shrinks, except that it freezes at death.
    #[test]
    fn structural_mass_never_shrinks(ops in proptest::collection::vec(op(), 1..200)) {
        let a = allom();
        let mut g = OrganArena::new();
        g.add(&organ(OrganKind::Seed, None), &a).unwrap();
        let mut tick = 0u64;
        for o in ops {
            tick += 1;
            let before: Vec<GramsDM> = g.structural_all().to_vec();
            let alive_before = g.alive.clone();
            match o {
                Op::Add { parent, kind } => {
                    let p = OrganId(u32::try_from(parent % g.len()).unwrap());
                    let kind = [OrganKind::Internode, OrganKind::Leaf, OrganKind::Root, OrganKind::Bud][kind as usize];
                    let _ = g.add(&organ(kind, Some(p)), &a);
                }
                Op::Grow { organ, delta } => {
                    let id = OrganId(u32::try_from(organ % g.len()).unwrap());
                    let _ = g.grow(id, GramsDM(delta), &a);
                }
                Op::Kill { organ } => {
                    let id = OrganId(u32::try_from(organ % g.len()).unwrap());
                    let _ = g.kill_subtree(id, Hours(tick));
                }
            }
            for (i, m) in before.iter().enumerate() {
                let now = g.structural_all()[i];
                prop_assert!(now.0 >= m.0, "organ {i} shrank: {} -> {}", m.0, now.0);
                if !alive_before[i] {
                    prop_assert_eq!(now, *m, "dead organ {} changed", i);
                }
            }
        }
    }
}
