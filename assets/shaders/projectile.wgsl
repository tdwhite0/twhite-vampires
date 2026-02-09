// Projectile Burst - Energy Bullet
// IQ-style: polar star flare, hot core with smooth radial falloff

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct ProjectileSettings {
    color: vec4<f32>,
    intensity: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ProjectileSettings;

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
    let uv = mesh.uv * 2.0 - 1.0;
    let d = length(uv);
    let angle = atan2(uv.y, uv.x);
    let t = globals.time;

    // Star flare shape - IQ polar modulation
    let rays = 6.0;
    let flare = abs(sin(angle * rays + t * 8.0));
    let flare_shape = smoothstep(1.0, 0.3, d - flare * 0.15);

    // Hot core with exponential falloff
    let core = exp(-d * 6.0) * 2.0;

    // Shimmering energy ring
    let ring = exp(-abs(d - 0.35) * 12.0) * 0.6;
    let ring_shimmer = ring * (0.7 + 0.3 * sin(angle * 4.0 - t * 12.0));

    // Noise distortion on the edges
    let edge_noise = noise(uv * 8.0 + t * 3.0) * 0.15;
    let outer = smoothstep(1.0 + edge_noise, 0.5, d);

    // Combine
    let energy = (flare_shape * 0.5 + core + ring_shimmer) * outer;

    // Color: white-hot core blending to weapon color
    let core_white = vec3(1.0, 1.0, 0.95);
    let blend = smoothstep(0.0, 0.6, d);
    let col = mix(core_white, material.color.rgb, blend) * energy * material.intensity;

    let alpha = outer * material.color.a;

    return vec4(col, alpha);
}
