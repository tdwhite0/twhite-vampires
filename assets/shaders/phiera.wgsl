// Phiera Der Tuphello - Fiery Red Energy Orb
// Aggressive pulsing fireball with tendrils and sparks

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct PhieraSettings {
    color: vec4<f32>,
    intensity: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: PhieraSettings;

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

fn fbm(p: vec2<f32>) -> f32 {
    var val = 0.0;
    var amp = 0.5;
    var pos = p;
    for (var i = 0; i < 4; i++) {
        val += amp * noise(pos);
        pos *= 2.1;
        amp *= 0.5;
    }
    return val;
}

fn polar_noise(angle: f32, dist: f32, t: f32) -> f32 {
    let p = vec2<f32>(cos(angle) * dist, sin(angle) * dist);
    return fbm(p * 3.0 + vec2<f32>(t * 1.5, t * 0.7));
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv * 2.0 - 1.0;
    let d = length(uv);
    let angle = atan2(uv.y, uv.x);
    let t = globals.time;

    // === White-hot core with steep exponential falloff ===
    let core_pulse = 1.0 + 0.3 * sin(t * 12.0) + 0.15 * sin(t * 17.3);
    let core = exp(-d * 8.0) * 3.0 * core_pulse;

    // === Pulsing energy ring that oscillates ===
    let ring_radius = 0.3 + 0.08 * sin(t * 6.0);
    let ring = exp(-abs(d - ring_radius) * 16.0) * 0.8;
    let ring_wobble = ring * (0.6 + 0.4 * sin(angle * 5.0 - t * 15.0));

    // === Fiery tendrils radiating outward (polar noise) ===
    let tendril_noise = polar_noise(angle, d, t);
    let tendril_mask = smoothstep(0.8, 0.2, d);
    let tendrils = tendril_noise * tendril_mask * 0.7;

    // Secondary tendrils at different frequency
    let tendril2 = polar_noise(angle * 1.5 + 1.0, d * 1.3, t * 0.8);
    let tendrils2 = tendril2 * tendril_mask * 0.4;

    // === Aggressive flare shape ===
    let flare_count = 8.0;
    let flare = abs(sin(angle * flare_count + t * 10.0));
    let flare2 = abs(sin(angle * 3.0 - t * 7.0));
    let flare_shape = smoothstep(1.1, 0.25, d - flare * 0.12 - flare2 * 0.08);

    // === Particle-like sparks around edges ===
    let spark_angle = angle + t * 4.0;
    let spark_grid = vec2<f32>(spark_angle * 5.0, d * 20.0);
    let spark_hash = hash21(floor(spark_grid));
    let spark_life = fract(spark_hash * 7.0 + t * 3.0);
    let spark_bright = step(0.92, spark_hash) * (1.0 - spark_life) * smoothstep(0.8, 0.3, d) * smoothstep(0.15, 0.25, d);
    let sparks = spark_bright * 2.0;

    // === Outer edge noise distortion ===
    let edge_noise = fbm(uv * 6.0 + t * 2.5) * 0.2;
    let outer = smoothstep(1.0 + edge_noise, 0.4, d);

    // === Combine all elements ===
    let energy = (flare_shape * 0.4 + core + ring_wobble + tendrils + tendrils2 + sparks) * outer;

    // === Color: white-hot core blending to fiery red/orange ===
    let core_white = vec3<f32>(1.0, 0.95, 0.8);
    let mid_orange = vec3<f32>(1.0, 0.5, 0.1);
    let blend1 = smoothstep(0.0, 0.25, d);
    let blend2 = smoothstep(0.25, 0.6, d);
    let base_col = mix(core_white, mid_orange, blend1);
    let col = mix(base_col, material.color.rgb, blend2) * energy * material.intensity;

    let alpha = outer * material.color.a;

    return vec4<f32>(col, alpha);
}
