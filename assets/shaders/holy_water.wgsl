// Holy Water - Glowing Liquid Pool
// IQ-style: caustic ripples, concentric rings, soft circular SDF

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct HolyWaterSettings {
    color: vec4<f32>,
    intensity: f32,
    lifetime_frac: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: HolyWaterSettings;

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

// Caustic pattern - overlapping sine waves that create light refraction look
fn caustics(p: vec2<f32>, t: f32) -> f32 {
    var c = 0.0;
    // Three rotated layers of sine waves
    let p1 = p * 3.0 + vec2(t * 0.4, t * 0.3);
    c += sin(p1.x * 2.1 + sin(p1.y * 1.7 + t)) * 0.5 + 0.5;

    let a2 = 2.094; // 120 degrees
    let p2 = vec2(p.x * cos(a2) - p.y * sin(a2), p.x * sin(a2) + p.y * cos(a2)) * 3.0 + vec2(t * -0.3, t * 0.5);
    c += sin(p2.x * 2.3 + sin(p2.y * 1.9 - t * 0.7)) * 0.5 + 0.5;

    let a3 = 4.189; // 240 degrees
    let p3 = vec2(p.x * cos(a3) - p.y * sin(a3), p.x * sin(a3) + p.y * cos(a3)) * 3.0 + vec2(t * 0.2, t * -0.4);
    c += sin(p3.x * 1.8 + sin(p3.y * 2.1 + t * 0.5)) * 0.5 + 0.5;

    return c / 3.0;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv * 2.0 - 1.0;
    let d = length(uv);
    let t = globals.time;

    // Circular pool SDF with soft edges
    let pool_edge = smoothstep(1.0, 0.7, d);

    // Caustic light pattern
    let caust = caustics(uv, t);

    // Concentric expanding ripple rings
    let ripple_freq = 8.0;
    let ripple_speed = 2.5;
    let ripple = sin(d * ripple_freq - t * ripple_speed) * 0.5 + 0.5;
    let ripple_mask = smoothstep(0.9, 0.3, d); // Fade ripples toward edge
    let ripple_effect = ripple * ripple_mask * 0.3;

    // Gentle noise distortion at edges
    let edge_noise = noise(uv * 5.0 + t * 0.3) * 0.15;
    let distorted_edge = smoothstep(1.0 + edge_noise, 0.65, d);

    // Combine effects
    let energy = distorted_edge * (0.4 + caust * 0.4 + ripple_effect);

    // Gentle pulse
    let pulse = 0.9 + 0.1 * sin(t * 2.0);

    // Color gradient: white center -> teal -> transparent edges
    let teal = material.color.rgb;
    let bright_teal = teal + vec3(0.3, 0.1, 0.2);
    let white = vec3(1.0, 1.0, 0.95);

    var col: vec3<f32>;
    if energy > 0.6 {
        col = mix(bright_teal, white, (energy - 0.6) / 0.4);
    } else if energy > 0.3 {
        col = mix(teal, bright_teal, (energy - 0.3) / 0.3);
    } else {
        col = teal * (energy / 0.3);
    }

    // Add caustic highlights
    let highlight = pow(caust, 3.0) * pool_edge * 0.4;
    col += vec3(highlight);

    col = col * pulse * material.intensity;

    // Alpha: pool shape * lifetime fade
    let base_alpha = clamp(energy, 0.0, 1.0) * pool_edge;
    let alpha = base_alpha * material.lifetime_frac * material.color.a;

    return vec4(col, alpha);
}
