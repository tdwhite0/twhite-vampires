// CRT screen emulation shader
// Inspired by Inigo Quilez's techniques for screen-space effects
//
// Features: barrel distortion, scanlines, chromatic aberration,
//           phosphor dot pattern, vignette, subtle flicker

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct CrtSettings {
    time: f32,
    curvature: f32,
    chromatic_aberration: f32,
    scanline_intensity: f32,
    phosphor_intensity: f32,
    vignette_strength: f32,
    _padding1: f32,
    _padding2: f32,
}

@group(0) @binding(2) var<uniform> settings: CrtSettings;

// Barrel distortion - warps UV outward from center
fn barrel_distort(uv: vec2<f32>, amount: f32) -> vec2<f32> {
    let centered = uv - 0.5;
    let r2 = dot(centered, centered);
    let distorted = centered * (1.0 + amount * r2 + amount * 0.5 * r2 * r2);
    return distorted + 0.5;
}

// Check if UV is in bounds after distortion
fn in_bounds(uv: vec2<f32>) -> f32 {
    let edge = smoothstep(0.0, 0.005, uv.x)
             * smoothstep(0.0, 0.005, uv.y)
             * smoothstep(0.0, 0.005, 1.0 - uv.x)
             * smoothstep(0.0, 0.005, 1.0 - uv.y);
    return edge;
}

// Scanline pattern - horizontal lines that dim alternating rows
fn scanline(uv: vec2<f32>, resolution_y: f32, intensity: f32) -> f32 {
    let line = sin(uv.y * resolution_y * 3.14159) * 0.5 + 0.5;
    return mix(1.0 - intensity, 1.0, line);
}

// Phosphor RGB sub-pixel pattern
fn phosphor(uv: vec2<f32>, resolution_x: f32) -> vec3<f32> {
    let pixel_x = uv.x * resolution_x;
    let sub = fract(pixel_x / 3.0) * 3.0;

    var mask = vec3<f32>(0.6, 0.6, 0.6);
    if sub < 1.0 {
        mask.x = 1.0;
    } else if sub < 2.0 {
        mask.y = 1.0;
    } else {
        mask.z = 1.0;
    }
    return mask;
}

// Vignette - darken edges of screen
fn vignette(uv: vec2<f32>, strength: f32) -> f32 {
    let centered = uv - 0.5;
    let d = dot(centered, centered);
    return 1.0 - d * strength;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(textureDimensions(screen_texture));

    // Distort UVs with barrel effect
    let uv = barrel_distort(in.uv, settings.curvature);

    // Black outside the curved screen edge
    let bounds = in_bounds(uv);
    if bounds < 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Chromatic aberration - shift RGB channels slightly
    let ca_offset = (uv - 0.5) * settings.chromatic_aberration;
    let r = textureSample(screen_texture, texture_sampler, uv + ca_offset).r;
    let g = textureSample(screen_texture, texture_sampler, uv).g;
    let b = textureSample(screen_texture, texture_sampler, uv - ca_offset).b;
    var color = vec3<f32>(r, g, b);

    // Scanlines
    let scan = scanline(uv, resolution.y, settings.scanline_intensity);
    color *= scan;

    // Phosphor sub-pixel mask
    let phos = phosphor(uv, resolution.x);
    color *= mix(vec3<f32>(1.0), phos, settings.phosphor_intensity);

    // Vignette
    let vig = vignette(uv, settings.vignette_strength);
    color *= clamp(vig, 0.0, 1.0);

    // Subtle brightness flicker
    let flicker = 0.98 + 0.02 * sin(settings.time * 8.0);
    color *= flicker;

    // Slight green/warm tint like old CRT phosphors
    color *= vec3<f32>(0.95, 1.0, 0.92);

    // Boost brightness slightly to compensate for scanlines + vignette dimming
    color *= 1.15;

    // Edge darkening at screen border (smooth transition to black)
    color *= bounds;

    return vec4<f32>(color, 1.0);
}
