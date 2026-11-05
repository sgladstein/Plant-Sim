use glam::Vec3;
use ps_graph::{
    Allometry, ExpansionCurve, NewOrgan, OrganArena, OrganId, OrganKind, Orientation, SpecificSize,
};
use ps_units::{GramsC, GramsDM, GramsN, Hours, MPerG, Radians};

use crate::demo::{DemoParams, demo_bean, demo_origin};
use crate::{
    Axis, AxisKind, AxisSignature, AxisStyle, InstanceOptions, MeshData, Skeleton, Template,
    TemplateKey, TubeNode, axes, axis_mesh, build_template, droop_level, instances, tube,
};

fn organ(kind: OrganKind, parent: Option<OrganId>, order: u8, mass: f64, d: [f64; 3]) -> NewOrgan {
    NewOrgan {
        kind,
        parent,
        axis_order: order,
        birth: Hours(0),
        curve: ExpansionCurve::none(),
        specific: match kind {
            OrganKind::Internode | OrganKind::Root => SpecificSize::Length(MPerG(0.2)),
            OrganKind::Leaf | OrganKind::Cotyledon => SpecificSize::Area(ps_units::M2PerG(0.025)),
            _ => SpecificSize::Bulk,
        },
        orientation: Orientation {
            direction: d,
            azimuth: Radians(0.0),
        },
        structural: GramsDM(mass),
        reserve: GramsC(0.0),
        nitrogen: GramsN(0.0),
    }
}

/// Seed, three main-stem internodes, a two-internode branch off the second, a leaf, a
/// two-segment root.
fn small_plant(mass: f64) -> OrganArena {
    let a = Allometry::generic_herb();
    let mut g = OrganArena::new();
    let up = [0.0, 0.0, 1.0];
    let seed = g
        .add(&organ(OrganKind::Seed, None, 0, 0.4, up), &a)
        .unwrap();
    let i1 = g
        .add(&organ(OrganKind::Internode, Some(seed), 0, mass, up), &a)
        .unwrap();
    let i2 = g
        .add(&organ(OrganKind::Internode, Some(i1), 0, mass, up), &a)
        .unwrap();
    let side = [1.0, 0.0, 1.0];
    let b1 = g
        .add(&organ(OrganKind::Internode, Some(i2), 1, mass, side), &a)
        .unwrap();
    let i3 = g
        .add(&organ(OrganKind::Internode, Some(i2), 0, mass, up), &a)
        .unwrap();
    let _b2 = g
        .add(&organ(OrganKind::Internode, Some(b1), 1, mass, side), &a)
        .unwrap();
    let _leaf = g
        .add(
            &organ(OrganKind::Leaf, Some(i3), 0, mass, [0.0, 1.0, 0.3]),
            &a,
        )
        .unwrap();
    let r1 = g
        .add(
            &organ(OrganKind::Root, Some(seed), 0, mass, [0.0, 0.0, -1.0]),
            &a,
        )
        .unwrap();
    let _r2 = g
        .add(
            &organ(OrganKind::Root, Some(r1), 0, mass, [0.0, 0.0, -1.0]),
            &a,
        )
        .unwrap();
    g
}

fn assert_sound(m: &MeshData) {
    assert!(m.is_well_formed(), "mesh not well formed");
    assert!(m.is_finite(), "mesh has NaN or infinity");
    for n in &m.normals {
        let len = Vec3::from(*n).length();
        assert!((len - 1.0).abs() < 1e-3, "normal not unit: {len}");
    }
}

/// Every triangle with area winds so its face normal agrees with its vertex normals.
fn assert_winding_matches_normals(m: &MeshData) {
    for t in m.indices.chunks_exact(3) {
        let p = [t[0], t[1], t[2]].map(|i| Vec3::from(m.positions[i as usize]));
        let face = (p[1] - p[0]).cross(p[2] - p[0]);
        if face.length() < 1e-12 {
            continue;
        }
        let n: Vec3 = t.iter().map(|&i| Vec3::from(m.normals[i as usize])).sum();
        assert!(
            face.dot(n) > 0.0,
            "triangle {t:?} winds against its normals"
        );
    }
}

#[test]
fn tube_counts() {
    let nodes: Vec<TubeNode> = (0..4)
        .map(|k| TubeNode {
            position: Vec3::new(0.0, 0.0, k as f32 * 0.1),
            radius: 0.01,
        })
        .collect();
    let mut m = MeshData::new();
    tube(&nodes, 6, false, Vec3::Z, &mut m);
    assert_eq!(m.vertex_count(), 4 * 7);
    assert_eq!(m.triangle_count(), 3 * 6 * 2);
    let mut capped = MeshData::new();
    tube(&nodes, 6, true, Vec3::Z, &mut capped);
    assert_eq!(capped.vertex_count(), 4 * 7 + 6 + 2);
    assert_eq!(capped.triangle_count(), 3 * 6 * 2 + 6);
    assert_sound(&capped);
    assert_winding_matches_normals(&capped);
}

#[test]
fn tube_normals_point_outward() {
    // A bent, tapered tube: every side normal points away from its ring's center.
    let pts = [
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, 0.1),
        Vec3::new(0.05, 0.0, 0.18),
        Vec3::new(0.12, 0.02, 0.2),
    ];
    let nodes: Vec<TubeNode> = pts
        .iter()
        .enumerate()
        .map(|(k, &p)| TubeNode {
            position: p,
            radius: 0.01 * (1.0 - 0.2 * k as f32),
        })
        .collect();
    let mut m = MeshData::new();
    tube(&nodes, 8, false, Vec3::Z, &mut m);
    assert_sound(&m);
    assert_winding_matches_normals(&m);
    for (k, node) in nodes.iter().enumerate() {
        for j in 0..9 {
            let v = k * 9 + j;
            let out = Vec3::from(m.positions[v]) - node.position;
            assert!(out.dot(Vec3::from(m.normals[v])) > 0.0);
        }
    }
}

#[test]
fn tube_survives_degenerate_input() {
    // All nodes at one point, zero radius: no NaN anywhere.
    let nodes = [TubeNode {
        position: Vec3::ONE,
        radius: 0.0,
    }; 5];
    let mut m = MeshData::new();
    tube(&nodes, 5, true, Vec3::Z, &mut m);
    assert_sound(&m);
    // A hairpin.
    let hairpin = [
        TubeNode {
            position: Vec3::ZERO,
            radius: 0.01,
        },
        TubeNode {
            position: Vec3::Z,
            radius: 0.01,
        },
        TubeNode {
            position: Vec3::ZERO,
            radius: 0.01,
        },
    ];
    let mut h = MeshData::new();
    tube(&hairpin, 5, false, Vec3::Z, &mut h);
    assert_sound(&h);
    // Too short: nothing.
    let mut e = MeshData::new();
    tube(&nodes[..1], 5, false, Vec3::Z, &mut e);
    assert!(e.is_empty());
}

#[test]
fn placement_chains_tips_to_bases() {
    let g = small_plant(0.1);
    let s = Skeleton::from_arena(&g, [0.0, 0.0, -0.03]);
    assert_eq!(s.len(), g.len());
    let seed = s.organs[0];
    assert_eq!(seed.base, Vec3::new(0.0, 0.0, -0.03));
    for p in &s.organs {
        if let Some(parent) = p.parent {
            assert!((p.base - s.organs[parent.index()].tip()).length() < 1e-6);
        }
    }
    // The leaf sits at the tip of the third main-stem internode.
    let leaf = s.organs[6];
    assert_eq!(leaf.kind, OrganKind::Leaf);
    assert!((leaf.base - s.organs[4].tip()).length() < 1e-6);
}

#[test]
fn axes_follow_continuation() {
    let g = small_plant(0.1);
    let s = Skeleton::from_arena(&g, [0.0; 3]);
    let a = axes(&s, true);
    let ids = |ax: &crate::Axis| ax.organs.iter().map(|o| o.0).collect::<Vec<_>>();
    assert_eq!(a.len(), 3);
    assert_eq!((a[0].kind, ids(&a[0])), (AxisKind::Shoot, vec![1, 2, 4]));
    assert_eq!((a[1].kind, ids(&a[1])), (AxisKind::Shoot, vec![3, 5]));
    assert_eq!((a[2].kind, ids(&a[2])), (AxisKind::Root, vec![7, 8]));
    for ax in &a {
        let style = if ax.kind == AxisKind::Root {
            AxisStyle::ROOT
        } else {
            AxisStyle::STEM
        };
        let m = axis_mesh(&s, ax, style);
        assert_eq!(
            m.vertex_count(),
            (ax.organs.len() + 1) * (style.sides as usize + 1)
        );
        assert_sound(&m);
        assert_winding_matches_normals(&m);
    }
}

#[test]
fn zero_size_plant_has_no_nan() {
    let g = small_plant(0.0);
    let s = Skeleton::from_arena(&g, [0.0; 3]);
    for ax in axes(&s, true) {
        assert_sound(&axis_mesh(&s, &ax, AxisStyle::STEM));
    }
    for inst in instances(&s, InstanceOptions::default()) {
        assert!(inst.translation.is_finite() && inst.scale.is_finite());
        assert!(inst.rotation.is_finite() && inst.rotation.is_normalized());
    }
}

#[test]
fn every_template_is_sound() {
    for key in TemplateKey::all() {
        let m = build_template(key);
        assert!(!m.is_empty(), "{key:?} is empty");
        assert_sound(&m);
        assert_winding_matches_normals(&m);
    }
    assert_eq!(TemplateKey::all().len(), 3 * 5 + 6);
}

#[test]
fn leaf_templates_have_unit_blade_area() {
    for t in [Template::Unifoliate, Template::Trifoliate] {
        // Both faces of the blade plus a little petiole: about 2 m² at droop 0 before
        // bending (bending and cupping add a few percent).
        let m = build_template(TemplateKey {
            template: t,
            droop: 0,
        });
        let a = m.area();
        assert!((1.95..2.3).contains(&a), "{t:?} area {a}");
        // The blade faces up: the mean normal of the front half points up.
        let up: Vec3 = m
            .normals
            .iter()
            .map(|n| Vec3::from(*n))
            .filter(|n| n.z > 0.0)
            .sum();
        assert!(up.normalize().z > 0.8);
    }
}

#[test]
fn droop_lowers_the_blade() {
    let low = |d: u8| {
        build_template(TemplateKey {
            template: Template::Trifoliate,
            droop: d,
        })
        .bounds()
        .unwrap()
        .0
        .z
    };
    assert!(low(4) < low(0) - 0.2);
    assert_eq!(droop_level(1.0), 0);
    assert_eq!(droop_level(0.0), 4);
    assert_eq!(droop_level(f32::NAN), 0);
}

#[test]
fn rebuild_threshold_is_two_percent() {
    let s1 = Skeleton::from_arena(&small_plant(0.1), [0.0; 3]);
    let s2 = Skeleton::from_arena(&small_plant(0.101), [0.0; 3]);
    let s3 = Skeleton::from_arena(&small_plant(0.103), [0.0; 3]);
    let a = &axes(&s1, true)[0];
    let sig = |s: &Skeleton| AxisSignature::of(s, a, AxisStyle::STEM);
    assert!(!sig(&s2).needs_rebuild(&sig(&s1), 0.02));
    assert!(sig(&s3).needs_rebuild(&sig(&s1), 0.02));
}

#[test]
fn lerp_hits_both_ends_and_new_organs_start_at_zero() {
    let p = DemoParams::SUN;
    let h0 = 20 * 24;
    let a = Skeleton::from_arena(&demo_bean(p, h0), demo_origin(0.0, 0.0));
    let b = Skeleton::from_arena(&demo_bean(p, h0 + 24), demo_origin(0.0, 0.0));
    assert!(
        b.len() > a.len(),
        "the demo plant should add organs in a day"
    );
    let at0 = Skeleton::lerp(&a, &b, 0.0);
    let at1 = Skeleton::lerp(&a, &b, 1.0);
    for (i, pa) in a.organs.iter().enumerate() {
        assert!((at0.organs[i].tip() - pa.tip()).length() < 1e-6);
        assert!((at0.organs[i].length - pa.length).abs() < 1e-7);
    }
    for (i, pb) in b.organs.iter().enumerate() {
        assert!((at1.organs[i].tip() - pb.tip()).length() < 1e-6);
        if i >= a.len() {
            assert!(at0.organs[i].length.abs() < 1e-9 && at0.organs[i].area.abs() < 1e-12);
        }
    }
}

#[test]
fn geometry_is_continuous_between_hours() {
    // Hour-to-hour, no node of any axis jumps by more than 5 mm, and a fractional time
    // halfway lies between the two hours.
    let p = DemoParams::SUN;
    for day in [3_u64, 10, 25, 45, 70] {
        let h = day * 24;
        let a = Skeleton::from_arena(&demo_bean(p, h), demo_origin(0.0, 0.0));
        let b = Skeleton::from_arena(&demo_bean(p, h + 1), demo_origin(0.0, 0.0));
        let mid = Skeleton::lerp(&a, &b, 0.5);
        for (i, pa) in a.organs.iter().enumerate() {
            let jump = (b.organs[i].tip() - pa.tip()).length();
            assert!(
                jump < 0.005,
                "day {day} organ {i} jumped {jump} m in an hour"
            );
            let to_mid = (mid.organs[i].tip() - pa.tip()).length();
            assert!(to_mid <= jump + 1e-6);
        }
    }
}

#[test]
fn demo_plant_is_plausible_at_day_70() {
    let g = demo_bean(DemoParams::SUN, 70 * 24);
    let s = Skeleton::from_arena(&g, demo_origin(0.0, 0.0));
    let (lo, hi) = s.bounds().unwrap();
    let height = hi.z;
    assert!((0.25..0.6).contains(&height), "height {height}");
    assert!(lo.z < -0.2, "roots reach {}", lo.z);
    let pods = g.alive_of(OrganKind::Pod).count();
    assert!((8..=20).contains(&pods), "pods {pods}");
    let shade = demo_bean(DemoParams::SHADE, 70 * 24);
    let shade_h = Skeleton::from_arena(&shade, demo_origin(0.0, 0.0))
        .bounds()
        .unwrap()
        .1
        .z;
    assert!(shade_h > height, "shade stretches the stem");
    assert!(shade.alive_of(OrganKind::Pod).count() < pods);
    // Every mesh of the full plant is sound.
    for ax in axes(&s, true) {
        assert_sound(&axis_mesh(&s, &ax, AxisStyle::STEM));
    }
    let inst = instances(&s, InstanceOptions::default());
    assert!(inst.iter().any(|i| i.key.template == Template::Unifoliate));
    assert!(inst.iter().any(|i| i.key.template == Template::Trifoliate));
    assert!(
        !inst.iter().any(|i| i.key.template == Template::Cotyledon),
        "cotyledons dropped"
    );
}

#[test]
fn ids_are_stable_across_hours() {
    let a = demo_bean(DemoParams::SUN, 30 * 24);
    let b = demo_bean(DemoParams::SUN, 60 * 24);
    for i in 0..a.len() {
        assert_eq!(a.kind[i], b.kind[i]);
        assert_eq!(a.parent[i], b.parent[i]);
    }
}

#[test]
fn partial_attachment_and_root_cohorts() {
    let a = Allometry::generic_herb();
    let mut g = OrganArena::new();
    let down = [0.0, 0.0, -1.0];
    let seed = g
        .add(&organ(OrganKind::Seed, None, 0, 0.4, down), &a)
        .unwrap();
    let tap = g
        .add(&organ(OrganKind::Root, Some(seed), 0, 1.0, down), &a)
        .unwrap();
    let _lat = g
        .add(
            &organ(OrganKind::Root, Some(tap), 1, 1.0, [1.0, 0.0, 0.0]),
            &a,
        )
        .unwrap();
    let plain = Skeleton::from_arena(&g, [0.0; 3]);
    let s = Skeleton::from_arena_with(&g, [0.0; 3], &[1.0, 1.0, 0.25], &[1.0, 1.0, 4.0]);
    let tap_p = s.organs[1];
    let lat = s.organs[2];
    // The lateral cohort starts a quarter of the way down the taproot...
    let expect = tap_p.base.lerp(tap_p.tip(), 0.25);
    assert!((lat.base - expect).length() < 1e-6);
    // ...and is drawn as one of its four roots.
    assert!((lat.length - plain.organs[2].length / 4.0).abs() < 1e-6);
    // Blending keeps the attachment.
    let half = Skeleton::lerp(&s, &s, 0.5);
    assert!((half.organs[2].base - expect).length() < 1e-6);
}

#[test]
fn root_cohorts_draw_as_spread_copies() {
    let a = Allometry::generic_herb();
    let mut g = OrganArena::new();
    let down = [0.0, 0.0, -1.0];
    let seed = g
        .add(&organ(OrganKind::Seed, None, 0, 0.4, down), &a)
        .unwrap();
    let tap = g
        .add(&organ(OrganKind::Root, Some(seed), 0, 1.0, down), &a)
        .unwrap();
    let _lat = g
        .add(
            &organ(OrganKind::Root, Some(tap), 1, 1.0, [1.0, 0.0, 0.0]),
            &a,
        )
        .unwrap();
    let s = Skeleton::from_arena_with(&g, [0.0; 3], &[1.0, 1.0, 0.5], &[1.0, 1.0, 4.0]);
    let all = axes(&s, true);
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].copies, 1);
    let cohort = &all[1];
    assert_eq!(cohort.copies, 4);
    let single = Axis {
        copies: 1,
        ..cohort.clone()
    };
    let one = axis_mesh(&s, &single, AxisStyle::ROOT);
    let four = axis_mesh(&s, cohort, AxisStyle::ROOT);
    assert_eq!(four.positions.len(), 4 * one.positions.len());
    assert!(four.is_well_formed());
    // Every copy starts at the same point on the taproot and points a different way
    // around it (the taproot runs along z, so the copies differ in x-y direction).
    let n = one.positions.len();
    let tips: Vec<Vec3> = (0..4)
        .map(|k| Vec3::from(four.positions[k * n + n - 1]))
        .collect();
    for i in 0..4 {
        for j in i + 1..4 {
            assert!(
                (tips[i] - tips[j]).length() > 1e-4,
                "copies {i} and {j} overlap"
            );
        }
    }
    // Huge cohorts are capped.
    let big = Skeleton::from_arena_with(&g, [0.0; 3], &[1.0, 1.0, 0.5], &[1.0, 1.0, 500.0]);
    assert_eq!(big.organs[2].copies, crate::MAX_COPIES);
}

#[test]
fn a_grass_draws_blades_and_ears_at_the_sims_size() {
    let arena = small_plant(0.5);
    let mut s = Skeleton::from_arena(&arena, [0.0; 3]);
    s.blade_aspect = Some(18.0);
    for inst in instances(&s, InstanceOptions::default()) {
        let p = &s.organs[inst.organ.index()];
        match p.kind {
            OrganKind::Leaf => {
                assert_eq!(inst.key.template, Template::GrassBlade);
                // Length 18 widths, and length × width × shape is the blade's area.
                let (l, w) = (inst.scale.x, inst.scale.y);
                assert!((l / w - 18.0).abs() < 1e-3);
                assert!(
                    (l * w * crate::templates::BLADE_SHAPE - p.area).abs()
                        < 1e-6 * p.area.max(1e-6) + 1e-9
                );
            }
            OrganKind::Pod => assert_eq!(inst.key.template, Template::WheatEar),
            k => assert!(!matches!(k, OrganKind::Cotyledon | OrganKind::Flower)),
        }
    }
    // The blade template's area is the shape factor (one side; the mesh has two).
    let m = build_template(TemplateKey {
        template: Template::GrassBlade,
        droop: 0,
    });
    let area = m.area() / 2.0;
    assert!(
        (area - crate::templates::BLADE_SHAPE).abs() < 0.05,
        "{area}"
    );
}
