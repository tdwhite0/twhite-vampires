// Health Bar - Glowing Potion Liquid
// Diablo-style horizontal health bar with bubbling liquid, internal glow, and meniscus edge

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct HealthBarSettings {
    color: vec4<f32>,
    fill_percent: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: HealthBarSettings;

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

fn fbm(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amp = 0.5;
    var pos = p;
    for (var i = 0; i < octaves; i++) {
        value += amp * noise(pos);
        pos = pos * 2.0 + vec2(1.7, 9.2);
        amp *= 0.5;
    }
    return value;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let t = globals.time;
    let fill = material.fill_percent;

    // Wavy meniscus at fill edge
    let wave = sin(uv.y * 20.0 + t * 3.0) * 0.008
             + sin(uv.y * 12.0 - t * 2.0) * 0.005;
    let fill_edge = fill + wave;

    // Discard past fill
    if uv.x > fill_edge {
        return vec4(0.0);
    }

    // === Deep liquid base ===
    let base_noise = fbm(uv * vec2(6.0, 3.0) + vec2(t * 0.2, t * 0.1), 4);
    // Darker at bottom, brighter at top (light catches the surface)
    let depth = mix(0.35, 0.75, 1.0 - uv.y);
    let liquid = material.color.rgb * depth * (0.5 + base_noise * 0.5);

    // === Internal glow (pulsing warm core) ===
    let glow_pulse = 0.7 + 0.3 * sin(t * 2.5);
    let center_y = abs(uv.y - 0.45) * 2.0;
    let glow_strength = exp(-center_y * 2.5) * glow_pulse;
    let glow_color = vec3(1.0, 0.35, 0.15);

    // === Rising particles (potion bubbles) ===
    var bubbles = 0.0;
    for (var i = 0; i < 10; i++) {
        let fi = f32(i);
        let bx = hash21(vec2(fi * 7.13, 3.17)) * fill;
        let speed = 0.25 + hash21(vec2(fi * 2.71, 8.91)) * 0.35;
        let by = fract(t * speed + hash21(vec2(fi * 5.43, 1.23)));
        let size = 0.008 + hash21(vec2(fi * 9.87, 4.56)) * 0.014;
        let d = length(uv - vec2(bx, 1.0 - by));
        bubbles += smoothstep(size, size * 0.2, d) * 0.45;
    }

    // === Edge bloom at fill boundary ===
    let dist_to_edge = abs(uv.x - fill_edge);
    let edge_glow = exp(-dist_to_edge * 60.0) * 0.7;

    // === Top surface highlight ===
    let top_hl = exp(-uv.y * 6.0) * 0.25;

    // === Bottom shadow ===
    let bot_shadow = exp(-(1.0 - uv.y) * 8.0) * 0.15;

    // === Combine layers ===
    var col = liquid;
    col += glow_color * glow_strength * 0.6;
    col += vec3(1.0, 0.8, 0.6) * bubbles;
    col += material.color.rgb * 1.5 * edge_glow;
    col += vec3(1.0, 0.85, 0.7) * top_hl;
    col -= vec3(0.1, 0.0, 0.0) * bot_shadow;

    let alpha = material.color.a;
    return vec4(col, alpha);
}
