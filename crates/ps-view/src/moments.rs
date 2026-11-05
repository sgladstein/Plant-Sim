//! The flowering cue (design plan §2.3, WP20): "flowering as an event with a camera
//! cue". When the garden's first flowers open, the auto-framing camera moves in on the
//! shoots that flowered and a banner names them, then the view eases back out.
//!
//! The cue runs on sim time, not wall time, so a time-lapse renders it the same on any
//! machine, and a replay or seek shows it again. The posture moments (the emerging hook,
//! wilting, the lean toward the light) are geometry: see `ps_mesh::moments`.

use bevy::prelude::*;

use crate::scene::sim_to_bevy;
use crate::{CameraRig, Garden};

/// How long the flowering cue lasts, sim hours: about three days, so a time-lapse at
/// 14 daylight frames a day holds it for about two seconds.
pub const CUE_HOURS: f64 = 72.0;

/// Sim hours the camera takes to move in, and to ease back out at the end.
pub const CUE_EASE: (f64, f64) = (8.0, 24.0);

/// Plants that flower within this many hours of the first count as the same event.
pub const SAME_EVENT_HOURS: f64 = 24.0;

/// How much closer the camera comes at the height of the cue (a factor on the framing
/// distance).
pub const CUE_ZOOM: f32 = 0.82;

/// What the moments show this frame.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct Moments {
    /// A line for the caption or a toast, while a cue runs.
    pub banner: Option<String>,
    /// Strength of the camera cue, 0 (none) to 1 (fully in).
    pub cue: f32,
    /// Pots in the cue, by index in [`Garden::pots`].
    pub pots: Vec<usize>,
}

fn smooth(t: f64) -> f32 {
    let t = t.clamp(0.0, 1.0) as f32;
    t * t * (3.0 - 2.0 * t)
}

/// Strength of the cue `since` sim hours after the first flower: eases in, holds, eases
/// out; 0 outside the cue.
#[must_use]
pub fn cue_weight(since: f64) -> f32 {
    if !(0.0..CUE_HOURS).contains(&since) {
        return 0.0;
    }
    let (ease_in, ease_out) = CUE_EASE;
    smooth(since / ease_in).min(smooth((CUE_HOURS - since) / ease_out))
}

/// The flowering event in `garden`: its strength now and the pots in it.
#[must_use]
pub fn flowering(garden: &Garden) -> (f32, Vec<usize>) {
    let Some(first) = garden
        .pots
        .iter()
        .filter_map(|p| p.first_flower)
        .min_by(f64::total_cmp)
    else {
        return (0.0, Vec::new());
    };
    let w = cue_weight(garden.hour - first);
    if w <= 0.0 {
        return (0.0, Vec::new());
    }
    let pots = garden
        .pots
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            p.first_flower
                .is_some_and(|t| t - first <= SAME_EVENT_HOURS && t <= garden.hour)
        })
        .map(|(k, _)| k)
        .collect();
    (w, pots)
}

/// The banner for a flowering event in `pots`: "First flowers" and which plants, by
/// the short name before a label's colon ("Pot 1" of "Pot 1: 50% shade").
#[must_use]
pub fn banner(garden: &Garden, pots: &[usize]) -> String {
    let n = pots.len();
    let plants = garden.pots.len();
    if n == 0 {
        return String::new();
    }
    if n == plants && n > 1 {
        return if n == 2 {
            "First flowers on both plants".to_owned()
        } else {
            format!("First flowers on all {n} plants")
        };
    }
    let names: Vec<&str> = pots
        .iter()
        .filter_map(|&k| garden.pots.get(k))
        .map(|p| p.label.split(':').next().unwrap_or("").trim())
        .filter(|l| !l.is_empty())
        .collect();
    match (names.as_slice(), n) {
        (_, 1) if names.len() == 1 => format!("First flower · {}", names[0]),
        (_, 1) => "First flower".to_owned(),
        ([], _) => format!("First flowers on {n} plants"),
        (many, _) => format!("First flowers · {}", many.join(", ")),
    }
}

/// Runs after the scene sync: blends the auto-frame toward the flowering shoots and
/// fills [`Moments`].
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn flowering_cue(
    garden: Res<Garden>,
    mut rig: ResMut<CameraRig>,
    mut moments: ResMut<Moments>,
) {
    let (w, pots) = flowering(&garden);
    let next = Moments {
        banner: (w > 0.0).then(|| banner(&garden, &pots)),
        cue: w,
        pots,
    };
    if *moments != next {
        *moments = next;
    }
    // A user's close-up (focus) or manual camera wins over the cue.
    let active = w > 0.0 && rig.focus.is_none();
    let zoom = if active {
        1.0 + (CUE_ZOOM - 1.0) * w
    } else {
        1.0
    };
    if (rig.cue_zoom - zoom).abs() > 1e-6 {
        rig.cue_zoom = zoom;
    }
    if !active {
        return;
    }
    let mut shoots: Option<(Vec3, Vec3)> = None;
    for &k in &moments.pots {
        let Some(pot) = garden.pots.get(k) else {
            continue;
        };
        let Some((lo, hi)) = pot.skeleton.bounds() else {
            continue;
        };
        let c = Vec3::new(pot.center[0], pot.center[1], 0.0);
        for p in [c + lo.max(Vec3::new(lo.x, lo.y, 0.0)), c + hi] {
            let b = sim_to_bevy(p);
            shoots = Some(match shoots {
                None => (b, b),
                Some((l, h)) => (l.min(b), h.max(b)),
            });
        }
    }
    if let (Some((lo, hi)), Some((flo, fhi))) = (rig.frame_bounds, shoots) {
        rig.frame_bounds = Some((lo.lerp(flo, w), hi.lerp(fhi, w)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn garden(labels: &[&str], flowered: &[Option<f64>], hour: f64) -> Garden {
        use crate::{PotSize, PotView, SoilFace};
        Garden {
            pots: labels
                .iter()
                .zip(flowered)
                .map(|(label, first_flower)| PotView {
                    label: (*label).to_owned(),
                    center: [0.0, 0.0],
                    size: PotSize::RHIZOBOX,
                    skeleton: ps_mesh::Skeleton::default(),
                    water_stress: 1.0,
                    first_flower: *first_flower,
                    organ_values: std::collections::BTreeMap::new(),
                    soil_water: SoilFace::default(),
                    container: true,
                    supports: Vec::new(),
                    obstacles: Vec::new(),
                })
                .collect(),
            hour,
            stand_in_lenses: Vec::new(),
        }
    }

    #[test]
    fn the_first_flowering_is_one_event() {
        let labels = ["Pot 0: full sun", "Pot 1: 50% shade", "Pot 2: full sun"];
        // Pot 1 flowers first, pot 0 within a day: one event; pot 2 days later is not in it.
        let g = garden(&labels, &[Some(110.0), Some(100.0), Some(200.0)], 120.0);
        let (w, pots) = flowering(&g);
        assert!(w > 0.0);
        assert_eq!(pots, vec![0, 1]);
        assert_eq!(banner(&g, &pots), "First flowers · Pot 0, Pot 1");
        // Before the first flower, and after the cue, nothing.
        let before = flowering(&garden(&labels, &[None, Some(100.0), None], 99.0));
        assert!(before.0.abs() < f32::EPSILON && before.1.is_empty());
        let late = garden(
            &labels,
            &[Some(110.0), Some(100.0), None],
            100.0 + CUE_HOURS,
        );
        assert_eq!(flowering(&late), (0.0, Vec::new()));
        let both = garden(&labels[..2], &[Some(100.0), Some(100.0)], 101.0);
        assert_eq!(banner(&both, &[0, 1]), "First flowers on both plants");
        assert_eq!(banner(&both, &[1]), "First flower · Pot 1");
    }

    #[test]
    fn the_cue_eases_in_and_out_on_sim_time() {
        assert!(cue_weight(-1.0).abs() < f32::EPSILON);
        assert!(cue_weight(0.0).abs() < f32::EPSILON);
        assert!(cue_weight(4.0) > 0.0 && cue_weight(4.0) < 1.0);
        assert!((cue_weight(30.0) - 1.0).abs() < 1e-6);
        assert!(cue_weight(CUE_HOURS - 1.0) < 0.1);
        assert!(cue_weight(CUE_HOURS).abs() < f32::EPSILON);
    }
}
