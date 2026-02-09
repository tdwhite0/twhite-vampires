use bevy::prelude::*;
use bevy::core_pipeline::{
    core_2d::graph::{Core2d, Node2d},
    fullscreen_material::FullscreenMaterial,
};
use bevy::render::{
    extract_component::ExtractComponent,
    render_graph::{InternedRenderLabel, InternedRenderSubGraph, RenderLabel, RenderSubGraph},
    render_resource::{AsBindGroup, ShaderType},
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

// === Orbit Shield Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct OrbitShieldData {
    pub color: Vec4,
    pub intensity: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct OrbitShieldMaterial {
    #[uniform(0)]
    pub data: OrbitShieldData,
}

impl Material2d for OrbitShieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/orbit_shield.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === Projectile Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct ProjectileData {
    pub color: Vec4,
    pub intensity: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ProjectileMaterial {
    #[uniform(0)]
    pub data: ProjectileData,
}

impl Material2d for ProjectileMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/projectile.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === Lightning Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct LightningData {
    pub color: Vec4,
    pub intensity: f32,
    pub lifetime: f32,
    pub seed: f32,
    pub _pad: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct LightningMaterial {
    #[uniform(0)]
    pub data: LightningData,
}

impl Material2d for LightningMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lightning.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === Flame Aura Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct FlameAuraData {
    pub color: Vec4,
    pub intensity: f32,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub _pad: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct FlameAuraMaterial {
    #[uniform(0)]
    pub data: FlameAuraData,
}

impl Material2d for FlameAuraMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/flame_aura.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === Boss Laser Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct BossLaserData {
    pub color: Vec4,
    pub intensity: f32,
    pub charge_progress: f32,
    pub _pad1: f32,
    pub _pad2: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct BossLaserMaterial {
    #[uniform(0)]
    pub data: BossLaserData,
}

impl Material2d for BossLaserMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/boss_laser.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === Holy Water Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct HolyWaterData {
    pub color: Vec4,
    pub intensity: f32,
    pub lifetime_frac: f32,
    pub _pad1: f32,
    pub _pad2: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct HolyWaterMaterial {
    #[uniform(0)]
    pub data: HolyWaterData,
}

impl Material2d for HolyWaterMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/holy_water.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === UpDown Wave Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct UpDownData {
    pub color: Vec4,
    pub intensity: f32,
    pub lifetime_frac: f32,
    pub _pad1: f32,
    pub _pad2: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct UpDownMaterial {
    #[uniform(0)]
    pub data: UpDownData,
}

impl Material2d for UpDownMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/updown.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === Boomerang Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct BoomerangData {
    pub color: Vec4,
    pub intensity: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct BoomerangMaterial {
    #[uniform(0)]
    pub data: BoomerangData,
}

impl Material2d for BoomerangMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/boomerang.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === Phiera Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct PhieraData {
    pub color: Vec4,
    pub intensity: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct PhieraMaterial {
    #[uniform(0)]
    pub data: PhieraData,
}

impl Material2d for PhieraMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/phiera.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === Health Bar Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct HealthBarData {
    pub color: Vec4,
    pub fill_percent: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct HealthBarMaterial {
    #[uniform(0)]
    pub data: HealthBarData,
}

impl Material2d for HealthBarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/health_bar.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === XP Bar Material ===

#[derive(ShaderType, Clone, Debug)]
pub struct XpBarData {
    pub color: Vec4,
    pub fill_percent: f32,
    pub _pad1: f32,
    pub _pad2: f32,
    pub _pad3: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct XpBarMaterial {
    #[uniform(0)]
    pub data: XpBarData,
}

impl Material2d for XpBarMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/xp_bar.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// === CRT Post-Processing Effect ===

#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
pub struct CrtEffect {
    pub time: f32,
    pub curvature: f32,
    pub chromatic_aberration: f32,
    pub scanline_intensity: f32,
    pub phosphor_intensity: f32,
    pub vignette_strength: f32,
}

impl FullscreenMaterial for CrtEffect {
    fn fragment_shader() -> ShaderRef {
        "shaders/crt_effect.wgsl".into()
    }

    fn node_edges() -> Vec<InternedRenderLabel> {
        vec![
            Node2d::Tonemapping.intern(),
            Self::node_label().intern(),
            Node2d::EndMainPassPostProcessing.intern(),
        ]
    }

    fn sub_graph() -> Option<InternedRenderSubGraph> {
        Some(Core2d.intern())
    }
}

pub fn update_crt_time(
    time: Res<Time>,
    mut camera_query: Query<&mut CrtEffect>,
) {
    for mut crt in camera_query.iter_mut() {
        crt.time += time.delta_secs();
    }
}
