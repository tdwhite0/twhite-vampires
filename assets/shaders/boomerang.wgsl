// Boomerang - Spinning Energy Crescent
// IQ-style: arc SDF, internal energy swirl, edge glow

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct BoomerangSettings {
    color: vec4<f32>,
    intensity: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: BoomerangSettings;

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

// IQ's SDF for an arc
fn sd_arc(p: vec2<f32>, sc: vec2<f32>, ra: f32, rb: f32) -> f32 {
    var q = p;
    q.x = abs(q.x);
    var k: f32;
    if sc.y * q.x > sc.x * q.y {
        k = dot(q, sc);
    } else {
        k = length(q);
    }
    return sqrt(dot(q, q) + ra * ra - 2.0 * ra * k) - rb;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv * 2.0 - 1.0;
    let t = globals.time;

    // Rotate the UV space - spinning crescent
    let spin = t * 10.0;
    let cs = cos(spin);
    let sn = sin(spin);
    let ruv = vec2(uv.x * cs - uv.y * sn, uv.x * sn + uv.y * cs);

    // Arc SDF - crescent moon shape (~120 degree arc)
    let arc_angle = 2.1;
    let sc = vec2(sin(arc_angle), cos(arc_angle));
    let arc_dist = sd_arc(ruv, sc, 0.5, 0.12);

    // Soft shape with glow
    let shape = smoothstep(0.08, -0.02, arc_dist);
    let glow = exp(-max(arc_dist, 0.0) * 15.0) * 0.6;

    // Internal energy swirl
    let swirl_uv = ruv * 4.0 + vec2(t * 3.0, t * 2.0);
    let energy_pattern = noise(swirl_uv) * 0.4 + noise(swirl_uv * 2.0) * 0.2;

    // Edge highlight - IQ's SDF gradient rim lighting
    let edge = exp(-abs(arc_dist) * 40.0) * 0.8;

    // Trail afterimage
    let trail_d = length(uv);
    let trail = smoothstep(0.7, 0.3, trail_d) * 0.15;

    // Combine
    let energy = shape * (0.6 + energy_pattern) + glow + edge + trail;

    // Color: bright core with colored glow
    let core = vec3(1.0, 1.0, 1.0);
    let col_blend = smoothstep(0.5, 0.0, shape);
    let col = mix(core, material.color.rgb, col_blend) * energy * material.intensity;

    let alpha = clamp(energy, 0.0, 1.0) * material.color.a;

    return vec4(col, alpha);
}
