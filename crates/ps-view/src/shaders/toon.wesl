// Illustrated shading for the plant viewer (visual design review, 2026-11-05).
//
// A soft two-tone ramp from the sun with a cool shadow tone instead of black, light
// through backlit leaves, a warm rim, and ink darkening where a surface turns away from
// the camera. No specular. Lens colors arrive as emissive and are added unchanged.

import bevy_pbr::render::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
    forward_io::{VertexOutput, FragmentOutput},
    mesh_view_bindings::{view, lights},
    shadows::fetch_directional_shadow,
};

const INV_PI: f32 = 0.31830988;

// x: 0 = natural (standard PBR lighting), 1 = illustrated. y: 1 for background.
@group(constants::MATERIAL_BIND_GROUP) @binding(100)
var<uniform> toon_params: vec4<f32>;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(
        pbr_input.material.flags,
        pbr_input.material.alpha_cutoff,
        pbr_input.material.base_color,
    );
    let base = pbr_input.material.base_color;
    var out: FragmentOutput;
    if (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_UNLIT_BIT) != 0u {
        out.color = main_pass_post_lighting_processing(pbr_input, base);
        return out;
    }
    if toon_params.x < 0.5 {
        out.color = main_pass_post_lighting_processing(pbr_input, apply_pbr_lighting(pbr_input));
        return out;
    }
    // Vertex alpha below 1 marks an inked edge (leaf margins); opaque materials ignore
    // alpha otherwise, so the natural style never sees it.
    var edge = 0.0;
    @if(VERTEX_COLORS)
    edge = 1.0 - in.color.a;

    let N = pbr_input.N;
    let V = pbr_input.V;
    let sun = lights.directional_lights[0];
    let L = sun.direction_to_light;
    let sun_rgb = sun.color.rgb * view.exposure * INV_PI;

    let view_z = dot(vec4<f32>(
        view.view_from_world[0].z,
        view.view_from_world[1].z,
        view.view_from_world[2].z,
        view.view_from_world[3].z,
    ), pbr_input.world_position);
    let shadow = fetch_directional_shadow(
        0u, pbr_input.world_position, pbr_input.world_normal, view_z, in.position.xy);

    // Two tones with a soft edge: lit or in shade, little in between.
    let ndl = dot(N, L);
    let background = toon_params.y;
    // Background (lawn, trees) is shaded flat, so faceted shapes read as simple masses.
    let lit = mix(
        smoothstep(0.0, 0.12, ndl) * smoothstep(0.3, 0.7, shadow),
        0.55 + 0.45 * smoothstep(0.3, 0.7, shadow),
        background,
    );

    // The shade tone is a cool sky fill, a third of the sun, brighter facing up.
    let up = clamp(N.y * 0.5 + 0.5, 0.0, 1.0);
    let sky_tint = mix(vec3<f32>(0.42, 0.40, 0.36), vec3<f32>(0.50, 0.58, 0.78), up);
    let fill = (sun_rgb * 0.38 + lights.ambient_color.rgb * view.exposure * 14.0) * sky_tint;

    // Leaves let light through: a backlit blade glows a little.
    let through = pbr_input.material.diffuse_transmission
        * smoothstep(0.0, 0.25, -ndl) * shadow * 0.8;

    // Illustrations run a little richer than life.
    let luma = dot(base.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    // Illustrations run a little richer than life; the background runs paler and
    // greyer, so the plant is the most saturated thing on screen.
    let sat = mix(1.12, 0.55, background);
    let paint = max(mix(vec3<f32>(luma), base.rgb, sat), vec3<f32>(0.0))
        * mix(1.0, 1.25, background);
    var color = paint * (fill + sun_rgb * 0.8 * (lit + through));

    // A warm rim where the surface turns away from the camera, on the sunny side.
    let ndv = max(dot(N, V), 0.0);
    let rim = pow(1.0 - ndv, 4.0) * smoothstep(-0.2, 0.4, ndl);
    color += sun_rgb * rim * 0.18 * vec3<f32>(1.0, 0.9, 0.7);

    // Ink: surfaces seen edge-on (stem and pot silhouettes) darken toward a deep
    // brown-green line.
    // Ink: silhouettes and leaf margins, none on the background.
    let ink = max(1.0 - smoothstep(0.1, 0.38, ndv), smoothstep(0.3, 0.8, edge))
        * (1.0 - background);
    color = mix(color, vec3<f32>(0.012, 0.018, 0.008), ink * 0.9);

    color += pbr_input.material.emissive.rgb;
    out.color = main_pass_post_lighting_processing(pbr_input, vec4<f32>(color, base.a));
    return out;
}
