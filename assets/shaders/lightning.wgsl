// Lightning Zap - Procedural Jagged Bolt
// IQ-style: noise-displaced SDF segment, branching, electric glow

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct LightningSettings {
    color: vec4<f32>,
    intensity: f32,
    lifetime: f32,
    seed: f32,
    _pad: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: LightningSettings;

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
    let uv = mesh.uv;
    let t = globals.time;
    let seed = material.seed;

    // Remap y to centered coordinates (-1 to 1)
    let p = vec2(uv.x, (uv.y - 0.5) * 2.0);

    // Main bolt: noise-displaced center line - multiple octaves for jagged look
    let d1 = (noise(vec2(uv.x * 8.0 + seed, t * 20.0)) - 0.5) * 0.6;
    let d2 = (noise(vec2(uv.x * 16.0 + seed * 2.0, t * 30.0)) - 0.5) * 0.2;
    let d3 = (noise(vec2(uv.x * 32.0 + seed * 3.0, t * 40.0)) - 0.5) * 0.08;
    let bolt_center = d1 + d2 + d3;

    // Distance to displaced center line
    let dist_to_bolt = abs(p.y - bolt_center);

    // Main bolt glow - IQ's exponential falloff
    let main_bolt = exp(-dist_to_bolt * 25.0) * 1.5;

    // Wider glow halo
    let glow = exp(-dist_to_bolt * 8.0) * 0.5;

    // Secondary branch bolt
    let branch_offset = (noise(vec2(uv.x * 6.0 + seed + 5.0, t * 15.0)) - 0.5) * 0.8;
    let branch_strength = smoothstep(0.3, 0.5, uv.x) * smoothstep(0.9, 0.7, uv.x);
    let dist_to_branch = abs(p.y - branch_offset);
    let branch = exp(-dist_to_branch * 35.0) * 0.6 * branch_strength;

    // Flickering intensity
    let flicker = 0.7 + 0.3 * sin(t * 60.0 + seed * 10.0);

    // Fade at endpoints
    let endpoint_fade = smoothstep(0.0, 0.05, uv.x) * smoothstep(1.0, 0.95, uv.x);

    // Lifetime fade
    let life_fade = smoothstep(0.0, 0.05, material.lifetime);

    // Combine
    let energy = (main_bolt + glow + branch) * flicker * endpoint_fade * life_fade;

    // Color: white core with colored glow
    let core_color = vec3(1.0, 1.0, 1.0);
    let glow_color = material.color.rgb;
    let bolt_mix = smoothstep(0.8, 0.2, main_bolt);
    let col = mix(core_color, glow_color, bolt_mix) * energy * material.intensity;

    let alpha = clamp(energy, 0.0, 1.0) * material.color.a;

    return vec4(col, alpha);
}
