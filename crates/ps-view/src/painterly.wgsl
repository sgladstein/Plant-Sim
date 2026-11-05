// Painterly soft look: a full-screen pass after tonemapping (ps_view::painterly).
//
// 1. Wobble: the lookup position drifts by a pixel or two with low-frequency noise, so
//    edges read as drawn by hand instead of rasterized.
// 2. A four-sector Kuwahara filter flattens fine detail into soft patches of color, the
//    way a loaded brush does, while keeping shapes' edges.
// 3. Soft value steps: luminance is pulled toward a few bands with wide, smooth
//    transitions (stepped soft shading, not hard cel bands).
// 4. A muted watercolor palette: partly desaturated, shadows cooled toward blue-violet,
//    lights warmed toward cream, never pure black.
// 5. Paper: fine grain plus soft blotches, and pigment that pools a little at edges.
//
// With a lens on, the steps, palette shift and pigment pooling fall away, so lens colors
// stay where their legends put them.

@group(0) @binding(0) var screen: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

struct Painterly {
    // Kuwahara radius at a 1440 px tall view, px (scaled to the real height).
    radius: f32,
    // 0 to 1: how far values are pulled to their band.
    steps: f32,
    // 0 to 1: desaturation and the warm/cool split.
    mute: f32,
    // 0 to 1: paper grain and blotches.
    paper: f32,
    // Wobble amplitude at 1440 px, px.
    wobble: f32,
    // 0 to 1: pigment pooling at edges.
    edge: f32,
    // Number of value bands.
    bands: f32,
    // 0 to 1: share of the unfiltered pixel kept, so thin roots and stems survive.
    detail: f32,
}
@group(0) @binding(2) var<uniform> settings: Painterly;

struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

fn hash2(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(123.34, 456.21));
    let r = q + dot(q, q + 45.32);
    return fract(r.x * r.y);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash2(i);
    let b = hash2(i + vec2<f32>(1.0, 0.0));
    let c = hash2(i + vec2<f32>(0.0, 1.0));
    let d = hash2(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var s = 0.0;
    var amp = 0.5;
    var q = p;
    for (var i = 0; i < 4; i = i + 1) {
        s = s + amp * value_noise(q);
        q = q * 2.03 + vec2<f32>(17.1, 9.7);
        amp = amp * 0.5;
    }
    return s;
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn fetch(uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(screen, screen_sampler, uv, 0.0).rgb;
}

// Mean of the quadrant (sx, sy) of a (r+1)² box, and its variance (summed over channels).
fn sector(uv: vec2<f32>, texel: vec2<f32>, r: i32, sx: f32, sy: f32) -> vec4<f32> {
    var sum = vec3<f32>(0.0);
    var sq = vec3<f32>(0.0);
    var n = 0.0;
    for (var j = 0; j <= r; j = j + 1) {
        for (var i = 0; i <= r; i = i + 1) {
            let c = fetch(uv + vec2<f32>(f32(i) * sx, f32(j) * sy) * texel);
            sum = sum + c;
            sq = sq + c * c;
            n = n + 1.0;
        }
    }
    let mean = sum / n;
    let v = sq / n - mean * mean;
    return vec4<f32>(mean, v.r + v.g + v.b);
}

fn kuwahara(uv: vec2<f32>, texel: vec2<f32>, r: i32) -> vec3<f32> {
    if (r < 1) {
        return fetch(uv);
    }
    let a = sector(uv, texel, r, -1.0, -1.0);
    let b = sector(uv, texel, r, 1.0, -1.0);
    let c = sector(uv, texel, r, -1.0, 1.0);
    let d = sector(uv, texel, r, 1.0, 1.0);
    // Soft choice: weight each quadrant by inverse variance, so flat areas blend instead
    // of snapping between quadrants (less blocky than the classic minimum).
    let wa = 1.0 / (1e-4 + a.w * a.w * 400.0);
    let wb = 1.0 / (1e-4 + b.w * b.w * 400.0);
    let wc = 1.0 / (1e-4 + c.w * c.w * 400.0);
    let wd = 1.0 / (1e-4 + d.w * d.w * 400.0);
    return (a.rgb * wa + b.rgb * wb + c.rgb * wc + d.rgb * wd) / (wa + wb + wc + wd);
}

// Pulls a value toward `bands` levels with smooth, wide transitions.
fn soft_step(v: f32, bands: f32) -> f32 {
    let x = v * bands;
    let i = floor(x);
    let f = x - i;
    let s = smoothstep(0.25, 0.75, f);
    return (i + s) / bands;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(screen));
    let texel = 1.0 / dims;
    let scale = dims.y / 1440.0;
    let px = in.uv * dims;

    // 1. Wobble.
    let wob = settings.wobble * scale;
    let w = vec2<f32>(
        value_noise(px / (36.0 * scale)) - 0.5,
        value_noise(px / (36.0 * scale) + vec2<f32>(31.7, 11.3)) - 0.5,
    ) * 2.0 * wob;
    let uv = in.uv + w * texel;

    // 2. Brush patches.
    let r = i32(round(settings.radius * scale));
    let src = fetch(uv);
    // Thin features (roots, fine stems) are narrower than a brush patch; keep part of
    // the pixel itself, more where it stands out from the brush.
    let brush = kuwahara(uv, texel, r);
    let stand_out = smoothstep(0.04, 0.2, abs(luma(src) - luma(brush)));
    var col = mix(brush, src, settings.detail * (0.35 + 0.65 * stand_out));

    // 3. Soft value steps, applied to luminance so hue is kept.
    let l = max(luma(col), 1e-4);
    // Bands are spaced in a perceptual (square-root) value, not linear light.
    let sp = soft_step(sqrt(l), settings.bands);
    let stepped = sp * sp;
    // Fade the steps out in the darkest values, so a dusk scene keeps its shapes.
    let l2 = mix(l, stepped, settings.steps * smoothstep(0.08, 0.3, sqrt(l)));
    col = col * (l2 / l);

    // 4. Watercolor palette.
    let grey = vec3<f32>(luma(col));
    col = mix(col, grey, 0.18 * settings.mute);
    let lv = luma(col);
    let cool = vec3<f32>(0.88, 0.91, 1.04);
    let warm = vec3<f32>(1.05, 1.0, 0.90);
    col = col * mix(vec3<f32>(1.0), mix(cool, warm, smoothstep(0.15, 0.7, lv)), settings.mute);
    // Lifted darks: watercolor shadows are transparent washes, never ink black.
    let paper_white = vec3<f32>(0.97, 0.94, 0.86);
    // Lift relative to the pixel, so a dusk scene stays dark instead of turning grey.
    let lift = 0.35 * settings.mute * (1.0 - smoothstep(0.0, 0.3, lv));
    col = col + lift * (paper_white * 0.06 + col * 0.25) * vec3<f32>(0.9, 0.95, 1.1);
    // A gentle S-curve restores the contrast the muting took out.
    // In a perceptual (square-root) value, so darks are not crushed.
    let pc = sqrt(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)));
    let sc = mix(pc, pc * pc * (3.0 - 2.0 * pc), 0.3 * settings.mute);
    col = sc * sc;

    // 5. Pigment pooling: a faint, wide darkening on the darker side of a value change,
    // where a wash dries at its edge (a soft rim, not an ink line).
    let o = 5.0 * scale;
    let around = 0.25 * (luma(fetch(uv + vec2<f32>(o, 0.0) * texel))
        + luma(fetch(uv - vec2<f32>(o, 0.0) * texel))
        + luma(fetch(uv + vec2<f32>(0.0, o) * texel))
        + luma(fetch(uv - vec2<f32>(0.0, o) * texel)));
    let pool = smoothstep(0.02, 0.15, around - luma(src));
    col = col * (1.0 - settings.edge * 0.12 * pool);

    // Paper: grain, soft blotches, and granulation that settles in the darker washes.
    let grain = hash2(floor(px)) - 0.5;
    let blotch = fbm(px / (180.0 * scale)) - 0.5;
    let fibre = value_noise(vec2<f32>(px.x / (3.0 * scale), px.y / (14.0 * scale))) - 0.5;
    let dark = 1.0 - smoothstep(0.1, 0.8, luma(col));
    let p = settings.paper;
    col = col * (1.0 + p * (0.035 * grain + 0.10 * blotch + 0.03 * fibre) * (0.5 + dark));
    col = mix(col, col * paper_white, 0.2 * p);

    return vec4<f32>(clamp(col, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
