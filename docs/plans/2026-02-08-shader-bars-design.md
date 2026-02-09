# Shader-Based Health & XP Bars

## Overview

Replace the flat `BackgroundColor` UI node health/XP bars with world-space `Mesh2d` + `MeshMaterial2d` rectangles using custom WGSL shaders. Health bar gets a glowing potion liquid effect; XP bar gets a flowing energy stream effect.

## Architecture

- World-space `Rectangle` meshes rendered by a dedicated HUD camera (order 1, above the main camera)
- Each bar: background mesh (dark, static) + fill mesh (shader-animated, scaled by percentage)
- Parented to anchor entities at fixed screen-space positions (top-left)
- New system updates `Transform.scale.x` and shader `fill_percent` uniform each frame
- Two new materials: `HealthBarMaterial`, `XpBarMaterial` (same pattern as weapon materials)
- Two new shaders: `assets/shaders/health_bar.wgsl`, `assets/shaders/xp_bar.wgsl`
- Existing UI node bars removed; text labels remain as UI nodes

## Health Bar Shader — Glowing Potion

Uniforms: `color` (vec4), `fill_percent` (f32), padding.

Layers:
- Deep liquid base: dark crimson with slow horizontal noise distortion
- Internal glow: pulsing red-orange emissive core, stronger at center
- Rising particles: small bright spots drifting upward via fract-based loops
- Meniscus edge: wavy sine-distorted cutoff at fill boundary with glow bloom
- Top surface highlight: lighter band near top edge

Fill drains from right with animated meniscus at the new edge.

## XP Bar Shader — Flowing Energy Stream

Uniforms: `color` (vec4), `fill_percent` (f32), padding.

Layers:
- Energy core: bright blue-cyan center stripe flowing left-to-right via scrolling noise
- Pulse waves: periodic bright wavefronts traveling left-to-right
- Branching tendrils: jagged edge offshoots from stepped noise, flickering
- Shimmer: high-frequency brightness variation (electrical interference)

Hard cutoff at fill_percent with bright flare at the leading edge. Small sparks extend slightly past fill edge.
