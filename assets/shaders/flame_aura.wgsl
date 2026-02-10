// Flame Aura - Ring of Fire
// IQ-style: polar fbm flames, annular SDF, animated tendrils

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct FlameAuraSettings {
    color: vec4<f32>,
    intensity: f32,
    inner_radius: f32,
    outer_radius: f32,
    _pad: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: FlameAuraSettings;

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

// Seamless polar fbm: samples noise on a circle (cos/sin) to avoid
// any angular discontinuity from atan2
fn fbm_polar(angle: f32, r: f32, offset: vec2<f32>, octaves: i32, ang_scale: f32, rad_scale: f32) -> f32 {
    var value = 0.0;
    var amp = 0.5;
    var s = ang_scale;
    var rv = r * rad_scale;
    var off = offset;
    for (var i = 0; i < octaves; i++) {
        let n1 = noise(vec2(cos(angle) * s, rv) + off);
        let n2 = noise(vec2(sin(angle) * s, rv + 17.0) + off);
        value += amp * (n1 + n2) * 0.5;
        s *= 2.17;
        rv *= 2.17;
        off *= 2.17;
        amp *= 0.5;
    }
    return value;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv * 2.0 - 1.0;
    let d = length(uv);
    let angle = atan2(uv.y, uv.x);
    let t = globals.time;

    // Seamless flame fbm using cos/sin circle sampling (no atan2 seam)
    let flame_noise = fbm_polar(angle, d, vec2(t * 0.5, -t * 2.5), 5, 4.0, 4.0);
    let flame_noise2 = fbm_polar(angle, d, vec2(-t * 0.3, -t * 3.0), 4, 6.0, 6.0);
    let flame = flame_noise * 0.6 + flame_noise2 * 0.4;

    // Annular ring SDF
    let ring_center = (material.inner_radius + material.outer_radius) * 0.5;
    let ring_width = (material.outer_radius - material.inner_radius) * 0.5;
    let ring_dist = abs(d - ring_center) - ring_width;

    // Soft ring with flame displacement
    let flame_displacement = flame * 0.15;
    let ring = smoothstep(0.05, -0.1, ring_dist - flame_displacement);

    // Outer flame tendrils
    let tendril_base = smoothstep(material.outer_radius + 0.15, material.outer_radius - 0.05, d);
    let tendrils = flame * tendril_base * 0.6;

    // Inner glow
    let inner_glow = smoothstep(material.inner_radius + 0.05, material.inner_radius - 0.1, d) * 0.2;

    // Pulsing intensity
    let pulse = 0.85 + 0.15 * sin(t * 3.0);

    // Combine
    let energy = (ring * (0.5 + flame * 0.5) + tendrils + inner_glow) * pulse;

    // Fire color gradient: white core -> yellow -> orange -> red
    let fire_gradient = smoothstep(0.0, 1.0, energy);
    let white = vec3(1.0, 1.0, 0.9);
    let yellow = vec3(1.0, 0.9, 0.3);
    let orange = material.color.rgb;
    let red_col = vec3(0.8, 0.1, 0.0);

    var col: vec3<f32>;
    if fire_gradient > 0.7 {
        col = mix(yellow, white, (fire_gradient - 0.7) / 0.3);
    } else if fire_gradient > 0.4 {
        col = mix(orange, yellow, (fire_gradient - 0.4) / 0.3);
    } else {
        col = mix(red_col, orange, fire_gradient / 0.4);
    }
    col = col * energy * material.intensity;

    let alpha = clamp(energy, 0.0, 1.0) * material.color.a;

    return vec4(col, alpha);
}
