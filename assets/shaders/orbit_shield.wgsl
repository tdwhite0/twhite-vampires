// Orbit Shield - Plasma Energy Orb
// IQ-style: fbm noise for plasma turbulence, soft SDF glow, pulsing core

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct OrbitShieldSettings {
    color: vec4<f32>,
    intensity: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: OrbitShieldSettings;

// IQ's classic hash & noise
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
    let uv = mesh.uv * 2.0 - 1.0;
    let d = length(uv);
    let t = globals.time;

    // Soft circle SDF with smooth falloff
    let circle = smoothstep(1.0, 0.2, d);

    // Plasma turbulence - two overlapping fbm layers
    let plasma1 = fbm(uv * 3.0 + vec2(t * 0.8, t * 0.6), 5);
    let plasma2 = fbm(uv * 2.5 - vec2(t * 0.5, t * 0.9), 4);
    let plasma = plasma1 * 0.6 + plasma2 * 0.4;

    // Pulsing hot core - IQ's exponential falloff
    let pulse = 0.8 + 0.2 * sin(t * 4.0);
    let core_glow = exp(-d * 4.0) * pulse * 1.5;

    // Outer energy wisps using polar coordinate distortion
    let angle = atan2(uv.y, uv.x);
    let wisp_angle = angle + t * 2.0;
    let wisp = sin(wisp_angle * 3.0) * 0.15 * smoothstep(0.8, 0.3, d);

    // Combine layers
    let energy = circle * (0.4 + plasma * 0.6 + wisp) + core_glow;

    // Color blend: bright core fades to tinted edge
    let core_color = vec3(1.0, 1.0, 1.0);
    let edge_color = material.color.rgb;
    let col = mix(edge_color, core_color, core_glow * 0.7) * energy * material.intensity;

    let alpha = circle * material.color.a;

    return vec4(col, alpha);
}
