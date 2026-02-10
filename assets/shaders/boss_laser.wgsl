// Boss Laser Beam - Sweeping Death Ray
// IQ-style: SDF beam with scrolling energy, hot core, plasma edges

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct BossLaserSettings {
    color: vec4<f32>,
    intensity: f32,
    charge_progress: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: BossLaserSettings;

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash21(i), hash21(i + vec2(1.0, 0.0)), u.x),
        mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), u.x),
        u.y
    );
}

fn fbm(p_in: vec2<f32>) -> f32 {
    var p = p_in;
    var f = 0.0;
    var amp = 0.5;
    for (var i = 0; i < 4; i++) {
        f += amp * noise(p);
        p *= 2.1;
        amp *= 0.5;
    }
    return f;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let t = globals.time;
    let charge = material.charge_progress;

    // Remap: x = along beam (0 origin -> 1 tip), y = across beam centered
    let p = vec2(uv.x, (uv.y - 0.5) * 2.0);

    // Beam width varies along length - wider at origin, tapers at tip
    let beam_width = mix(0.35, 0.15, uv.x) * charge;

    // Noise displacement for organic wobble
    let wobble = (noise(vec2(uv.x * 6.0, t * 8.0)) - 0.5) * 0.15 * charge;
    let dist_to_center = abs(p.y - wobble);

    // SDF: soft beam shape
    let beam_sdf = smoothstep(beam_width, beam_width * 0.3, dist_to_center);

    // Scrolling internal energy - flows from origin to tip
    let energy_uv = vec2(uv.x * 4.0 - t * 6.0, p.y * 3.0);
    let energy = fbm(energy_uv) * beam_sdf;

    // Hot core - exponential falloff from center line
    let core = exp(-dist_to_center * 20.0 / max(charge, 0.01)) * charge;

    // Edge plasma tendrils
    let tendril_uv = vec2(uv.x * 8.0 - t * 4.0, p.y * 5.0 + t * 2.0);
    let tendrils = noise(tendril_uv) * smoothstep(beam_width * 0.5, beam_width * 1.5, dist_to_center)
                 * smoothstep(beam_width * 2.5, beam_width * 1.2, dist_to_center) * charge;

    // Tip fade and origin fade
    let tip_fade = smoothstep(1.0, 0.85, uv.x);
    let origin_fade = smoothstep(0.0, 0.05, uv.x);
    let length_fade = tip_fade * origin_fade;

    // Pulsing intensity
    let pulse = 0.85 + 0.15 * sin(t * 12.0);

    // Combine all effects
    let total = (beam_sdf * 0.6 + energy * 0.4 + core * 1.2 + tendrils * 0.3) * length_fade * pulse;

    // Color gradient: white-hot core -> green -> dark green edges (matching eyeball boss)
    let base_color = material.color.rgb;
    let hot_color = vec3(1.0, 1.0, 1.0);
    let mid_color = base_color * 1.5;
    let edge_color = base_color * 0.5;

    var col: vec3<f32>;
    if core > 0.5 {
        col = mix(mid_color, hot_color, smoothstep(0.5, 1.0, core));
    } else {
        col = mix(edge_color, mid_color, smoothstep(0.0, 0.5, beam_sdf));
    }

    col = col * total * material.intensity;

    // Add some extra brightness to the scrolling energy
    col += vec3(0.2, 1.0, 0.3) * energy * 0.3 * charge;

    let alpha = clamp(total, 0.0, 1.0) * material.color.a;

    return vec4(col, alpha);
}
