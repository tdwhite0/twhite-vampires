// Whip Slash - Clean horizontal slash line
// Thin bright line that flashes and fades, like VS whip

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct WhipSettings {
    color: vec4<f32>,
    intensity: f32,
    lifetime_frac: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: WhipSettings;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv; // 0 to 1
    let life = material.lifetime_frac; // 1.0 = just spawned, 0.0 = about to die

    // x: 0 = near player, 1 = tip
    // y: 0.5 = center line
    let x = uv.x;
    let y = abs(uv.y - 0.5) * 2.0; // 0 = center, 1 = edge

    // Thickness tapers: thick near player, thin at tip (like a whip)
    let taper = mix(1.0, 0.2, x * x);

    // Sharp horizontal line: Gaussian falloff from center, scaled by taper
    let line_width = 0.25 * taper;
    let line = exp(-(y * y) / (line_width * line_width + 0.001));

    // Slightly wider soft glow around the line
    let glow_width = 0.5 * taper;
    let glow = exp(-(y * y) / (glow_width * glow_width + 0.001)) * 0.3;

    // Fade the tip end smoothly
    let tip_fade = smoothstep(1.0, 0.85, x);

    // Combine line + glow
    let shape = (line + glow) * tip_fade;

    // Color: bright white core, base color edges
    let base_col = material.color.rgb;
    let white = vec3(1.0, 0.95, 0.85);
    let col = mix(base_col, white, line * tip_fade) * material.intensity;

    // Alpha: quick flash then fade out
    let fade = smoothstep(0.0, 0.4, life);
    let alpha = clamp(shape, 0.0, 1.0) * fade * material.color.a;

    if alpha < 0.02 {
        discard;
    }

    return vec4(col, alpha);
}
