// XP Bar - Flowing Energy Stream
// Electric current with pulse waves, branching tendrils, and shimmer

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct XpBarSettings {
    color: vec4<f32>,
    fill_percent: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: XpBarSettings;

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

    // Hard cutoff with small spark overflow at edge
    let spark_extend = noise(vec2(uv.y * 20.0, t * 5.0)) * 0.012;
    if uv.x > fill + spark_extend {
        return vec4(0.0);
    }

    // === Energy core (bright center stripe flowing left-to-right) ===
    let core_dist = abs(uv.y - 0.5) * 2.0;
    let core = exp(-core_dist * 3.0);
    let flow = fbm(vec2(uv.x * 5.0 - t * 2.0, uv.y * 3.0), 3);
    let core_brightness = core * (0.4 + flow * 0.6);

    // === Pulse waves traveling left-to-right ===
    let pulse_phase = fract(uv.x * 2.5 - t * 1.5);
    let pulse = smoothstep(0.0, 0.08, pulse_phase) * smoothstep(0.25, 0.12, pulse_phase);
    let pulse_strength = pulse * 0.5;

    // === Branching tendrils at top/bottom edges ===
    let tendril_noise_top = noise(vec2(uv.x * 12.0 - t * 4.0, 1.0));
    let tendril_noise_bot = noise(vec2(uv.x * 12.0 - t * 3.5, 5.0));
    let top_tendril = smoothstep(0.75, 0.55, uv.y) * step(0.6, tendril_noise_top) * 0.35;
    let bot_tendril = smoothstep(0.25, 0.45, uv.y) * step(0.6, tendril_noise_bot) * 0.35;

    // === Shimmer (electrical interference) ===
    let shimmer = hash21(floor(uv * vec2(80.0, 8.0)) + vec2(t * 3.0, 0.0)) * 0.07;

    // === Edge flare at fill boundary ===
    let dist_to_edge = abs(uv.x - fill);
    let edge_flare = exp(-dist_to_edge * 50.0) * 0.8;
    let edge_sparks = step(0.85, hash21(vec2(uv.y * 30.0, t * 10.0)))
                    * exp(-dist_to_edge * 30.0) * 0.5;

    // === Color composition ===
    let base_blue = material.color.rgb;
    let bright_cyan = vec3(0.5, 0.9, 1.0);
    let white = vec3(1.0, 1.0, 1.0);

    var col = base_blue * 0.25;
    col += mix(base_blue, bright_cyan, core_brightness) * core_brightness;
    col += bright_cyan * pulse_strength;
    col += base_blue * (top_tendril + bot_tendril);
    col += white * shimmer;
    col += white * edge_flare;
    col += bright_cyan * edge_sparks;

    let alpha = material.color.a;
    return vec4(col, alpha);
}
