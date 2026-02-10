// UpDown Wave - Vertical energy bar with sparkle particles
// IQ-style: vertical SDF bar, flowing energy, shimmer particles

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct UpDownSettings {
    color: vec4<f32>,
    intensity: f32,
    lifetime_frac: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: UpDownSettings;

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

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv * 2.0 - 1.0; // -1 to 1
    let t = globals.time;

    // Vertical bar shape - narrow on x, full on y
    let bar_sdf = abs(uv.x) - 0.5;
    let bar_mask = smoothstep(0.1, -0.1, bar_sdf);

    // Soft vertical fade at top and bottom
    let y_fade = smoothstep(1.0, 0.7, abs(uv.y));

    // Central core glow (bright center line)
    let core = exp(-abs(uv.x) * 6.0);

    // Flowing energy - vertical sine waves
    let flow1 = sin(uv.y * 12.0 - t * 8.0 + uv.x * 3.0) * 0.5 + 0.5;
    let flow2 = sin(uv.y * 8.0 + t * 5.0 - uv.x * 2.0) * 0.5 + 0.5;
    let flow = flow1 * 0.6 + flow2 * 0.4;

    // Sparkle particles - small bright dots that drift
    let sparkle_uv1 = uv * vec2(4.0, 12.0) + vec2(t * 0.7, t * 3.0);
    let sparkle_uv2 = uv * vec2(5.0, 10.0) + vec2(-t * 0.5, -t * 4.0);
    let sparkle1 = pow(noise(sparkle_uv1), 5.0);
    let sparkle2 = pow(noise(sparkle_uv2), 5.0);
    let sparkle = (sparkle1 + sparkle2) * 2.0;

    // Musical note / wave distortion at edges
    let edge_wave = sin(uv.y * 20.0 - t * 6.0) * 0.03;
    let distorted_x = abs(uv.x) - edge_wave;
    let edge_glow = exp(-max(distorted_x - 0.3, 0.0) * 15.0) * 0.5;

    // Combine all effects
    let energy = bar_mask * y_fade * (core * 0.5 + flow * 0.3 + sparkle * 0.4 + edge_glow);

    // Gentle pulse
    let pulse = 0.85 + 0.15 * sin(t * 3.0);

    // Color: white-hot core -> purple -> transparent edges
    let base_col = material.color.rgb;
    let bright = base_col + vec3(0.4, 0.3, 0.2);
    let white = vec3(1.0, 0.95, 1.0);

    var col: vec3<f32>;
    if energy > 0.7 {
        col = mix(bright, white, (energy - 0.7) / 0.3);
    } else if energy > 0.3 {
        col = mix(base_col, bright, (energy - 0.3) / 0.4);
    } else {
        col = base_col * (energy / 0.3);
    }

    // Add sparkle highlights
    let highlight = sparkle * bar_mask * y_fade * 0.6;
    col += vec3(highlight);

    col = col * pulse * material.intensity;

    // Alpha: shape * lifetime fade
    let base_alpha = clamp(energy, 0.0, 1.0);
    let alpha = base_alpha * material.lifetime_frac * material.color.a;

    return vec4(col, alpha);
}
