use bevy::prelude::*;
use bevy::image::TextureAtlasLayout;
use std::collections::HashMap;
use crate::components::*;

// === Sound Assets ===
#[derive(Resource)]
pub struct SoundAssets {
    pub enemy_death: [Handle<AudioSource>; 5],
    pub xp_gem: [Handle<AudioSource>; 3],
    pub player_damage: Handle<AudioSource>,
    pub level_up: Handle<AudioSource>,
    pub game_start: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
    pub weapon_pickup: Handle<AudioSource>,
    pub upgrade_selected: Handle<AudioSource>,
    pub weapon_projectile: Handle<AudioSource>,
    pub weapon_lightning: Handle<AudioSource>,
    pub weapon_flame: Handle<AudioSource>,
    pub weapon_boomerang: Handle<AudioSource>,
}

// === Game State ===
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    Playing,
    LevelUp,
    GameOver,
}

#[derive(Resource, Default)]
pub struct NeedsGameInit(pub bool);

// === Wave Management ===
#[derive(Resource)]
pub struct WaveManager {
    pub elapsed: f32,
    pub spawn_timer: f32,
    pub spawn_interval: f32,
    pub enemies_alive: u32,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            spawn_timer: 0.0,
            spawn_interval: 1.2,
            enemies_alive: 0,
        }
    }
}

// === Player Weapons ===
#[derive(Clone)]
pub struct ActiveWeapon {
    pub kind: WeaponKind,
    pub level: u32,
    pub damage: f32,
    pub cooldown: f32,
    pub timer: f32,
    pub area: f32,
    pub count: u32,
}

impl ActiveWeapon {
    pub const MAX_LEVEL: u32 = 8;

    /// Ticks the cooldown timer. Returns true when the weapon is ready to fire,
    /// and resets the timer automatically.
    pub fn tick_cooldown(&mut self, dt: f32) -> bool {
        self.timer -= dt;
        if self.timer > 0.0 {
            return false;
        }
        self.timer = self.cooldown;
        true
    }

    pub fn set_level(&mut self, level: u32) {
        let level = level.clamp(1, Self::MAX_LEVEL);
        let def = self.kind.level_def(level);
        self.level = level;
        self.damage = def.damage;
        self.cooldown = def.cooldown;
        self.area = def.area;
        self.count = def.count;
        // Don't reset timer - preserve cooldown progress
    }

    pub fn new(kind: WeaponKind) -> Self {
        let def = kind.level_def(1);
        Self {
            kind,
            level: 1,
            damage: def.damage,
            cooldown: def.cooldown,
            timer: 0.0,
            area: def.area,
            count: def.count,
        }
    }
}

#[derive(Resource, Default)]
pub struct PlayerWeapons {
    pub weapons: Vec<ActiveWeapon>,
}

impl PlayerWeapons {
    pub fn has(&self, kind: WeaponKind) -> bool {
        self.weapons.iter().any(|w| w.kind == kind)
    }

    pub fn get(&self, kind: WeaponKind) -> Option<&ActiveWeapon> {
        self.weapons.iter().find(|w| w.kind == kind)
    }

    pub fn get_mut(&mut self, kind: WeaponKind) -> Option<&mut ActiveWeapon> {
        self.weapons.iter_mut().find(|w| w.kind == kind)
    }
}

// === Player Abilities ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityKind {
    Dash,
}

impl AbilityKind {
    pub fn display_name(&self) -> &str {
        match self {
            AbilityKind::Dash => "Dash",
        }
    }

    pub fn keybind_label(&self) -> &str {
        match self {
            AbilityKind::Dash => "Click",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            AbilityKind::Dash => Color::srgb(0.3, 0.8, 1.0),
        }
    }
}

#[derive(Clone)]
pub struct ActiveAbility {
    pub kind: AbilityKind,
    pub cooldown: f32,
    pub timer: f32,
}

impl ActiveAbility {
    pub fn new(kind: AbilityKind) -> Self {
        match kind {
            AbilityKind::Dash => Self {
                kind,
                cooldown: 3.0,
                timer: 0.0,
            },
        }
    }

    pub fn is_ready(&self) -> bool {
        self.timer <= 0.0
    }

    pub fn trigger(&mut self) {
        self.timer = self.cooldown;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.timer > 0.0 {
            self.timer = (self.timer - dt).max(0.0);
        }
    }

    pub fn cooldown_fraction(&self) -> f32 {
        if self.cooldown <= 0.0 {
            return 0.0;
        }
        (self.timer / self.cooldown).clamp(0.0, 1.0)
    }
}

#[derive(Resource, Default)]
pub struct PlayerAbilities {
    pub abilities: Vec<ActiveAbility>,
}

impl PlayerAbilities {
    pub fn get_mut(&mut self, kind: AbilityKind) -> Option<&mut ActiveAbility> {
        self.abilities.iter_mut().find(|a| a.kind == kind)
    }

    pub fn index_of(&self, kind: AbilityKind) -> Option<usize> {
        self.abilities.iter().position(|a| a.kind == kind)
    }
}

// === Debug Settings ===
#[derive(Resource)]
pub struct DebugSettings {
    pub invincible: bool,
    pub damage_multiplier: f32,
    pub crt_enabled: bool,
    pub crt_curvature: f32,
    pub crt_chromatic_aberration: f32,
    pub crt_scanline_intensity: f32,
    pub crt_phosphor_intensity: f32,
    pub crt_vignette_strength: f32,
    pub show_boss_hitboxes: bool,
    pub settings_tab: SettingsTab,
    pub spawn_rate_multiplier: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Cheats,
    Weapons,
    AudioVideo,
}

impl SettingsTab {
    pub fn label(&self) -> &'static str {
        match self {
            SettingsTab::Cheats => "Cheats",
            SettingsTab::Weapons => "Weapons",
            SettingsTab::AudioVideo => "A/V",
        }
    }

    pub const ALL: [SettingsTab; 3] = [
        SettingsTab::Cheats,
        SettingsTab::Weapons,
        SettingsTab::AudioVideo,
    ];
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            invincible: false,
            damage_multiplier: 1.0,
            crt_enabled: true,
            crt_curvature: 0.15,
            crt_chromatic_aberration: 0.003,
            crt_scanline_intensity: 0.25,
            crt_phosphor_intensity: 0.25,
            crt_vignette_strength: 1.8,
            show_boss_hitboxes: false,
            settings_tab: SettingsTab::default(),
            spawn_rate_multiplier: 1.0,
        }
    }
}

// === Debug Boss Spawning ===
#[derive(Resource)]
pub struct ForceSpawnBoss {
    pub kind: Option<crate::components::BossKind>,
}

// === Music State ===
pub struct MusicTrack {
    pub name: &'static str,
    pub handle: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct MusicState {
    pub selected: usize,
    pub volume: f32,
    pub muted: bool,
    pub tracks: Vec<MusicTrack>,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            selected: 0,
            volume: 0.5,
            muted: false,
            tracks: Vec::new(),
        }
    }
}

// === Pickup Range (Attractorb) ===
#[derive(Resource)]
pub struct PickupRange {
    pub level: u32,
    pub multiplier: f32,
}

impl Default for PickupRange {
    fn default() -> Self {
        Self {
            level: 0,
            multiplier: 1.0,
        }
    }
}

impl PickupRange {
    pub const MAX_LEVEL: u32 = 5;
}

// === Game Stats ===
#[derive(Resource, Default)]
pub struct GameStats {
    pub time_survived: f32,
    pub enemies_killed: u32,
    pub bosses_killed: u32,
    pub xp_collected: f32,
    pub damage_taken: f32,
    pub heals_collected: u32,
    pub weapon_damage: HashMap<WeaponKind, f32>,
}

impl GameStats {
    pub fn record_weapon_damage(&mut self, kind: WeaponKind, amount: f32) {
        *self.weapon_damage.entry(kind).or_insert(0.0) += amount;
    }
}

// === Pickup Spawning ===
#[derive(Resource)]
pub struct PickupSpawnTimer(pub f32);

impl Default for PickupSpawnTimer {
    fn default() -> Self {
        Self(10.0)
    }
}

// === Visual Effects ===
#[allow(dead_code)]
pub struct LightningBolt {
    pub start: Vec2,
    pub end: Vec2,
    pub timer: f32,
}

#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct VisualEffects {
    pub lightning_bolts: Vec<LightningBolt>,
}

// === Boss Spawning ===
#[derive(Resource)]
pub struct BossSpawnTimer {
    pub timer: f32,
    pub interval: f32,
    pub spawn_counter: u32,
}

impl Default for BossSpawnTimer {
    fn default() -> Self {
        Self {
            timer: 60.0,
            interval: 60.0,
            spawn_counter: 0,
        }
    }
}

// === Game Font ===
pub struct GameFont {
    pub name: &'static str,
    pub handle: Handle<Font>,
}

#[derive(Resource)]
pub struct GameFontState {
    pub fonts: Vec<GameFont>,
    pub selected: usize,
}

impl GameFontState {
    pub fn current(&self) -> Handle<Font> {
        self.fonts[self.selected].handle.clone()
    }

    pub fn current_name(&self) -> &'static str {
        self.fonts[self.selected].name
    }

    pub fn cycle(&mut self) {
        self.selected = (self.selected + 1) % self.fonts.len();
    }
}

// === Boss Sprite Assets ===
#[derive(Resource)]
pub struct BossAssets {
    pub eye_texture_1: Handle<Image>,
    pub eye_texture_2: Handle<Image>,
    pub eye_texture_3: Handle<Image>,
    pub eye_layout: Handle<TextureAtlasLayout>,
}

// === Character Selection ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CharacterKind {
    #[default]
    Warrior,
    Robot,
    Mage,
}

impl CharacterKind {
    pub const ALL: [CharacterKind; 3] = [
        CharacterKind::Warrior,
        CharacterKind::Robot,
        CharacterKind::Mage,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            CharacterKind::Warrior => "Warrior",
            CharacterKind::Robot => "Robot",
            CharacterKind::Mage => "Mage",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CharacterKind::Warrior => "Sword-wielding fighter",
            CharacterKind::Robot => "Chrome machine of war",
            CharacterKind::Mage => "Arcane spellcaster",
        }
    }

    pub fn border_color(&self) -> Color {
        match self {
            CharacterKind::Warrior => Color::srgb(1.0, 0.6, 0.2),
            CharacterKind::Robot => Color::srgb(0.3, 0.8, 1.0),
            CharacterKind::Mage => Color::srgb(0.7, 0.3, 1.0),
        }
    }

    /// Returns (texture, layout) keys into SpriteAssets
    pub fn sprite_info(&self, sprites: &SpriteAssets) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
        match self {
            CharacterKind::Warrior => (sprites.hero_texture.clone(), sprites.hero_layout.clone()),
            CharacterKind::Robot => (sprites.robot_texture.clone(), sprites.robot_layout.clone()),
            CharacterKind::Mage => (sprites.mage_texture.clone(), sprites.mage_layout.clone()),
        }
    }

    /// Returns animation config for this character.
    /// (row_front, row_back, row_side, frame_count, cols_per_row)
    pub fn anim_config(&self) -> CharacterAnimConfig {
        match self {
            CharacterKind::Warrior => CharacterAnimConfig {
                // Warrior: 14 cols x 25 rows, 64x64
                // Move-with-sword: front=row9, right=row3, back=row21
                cols: 14,
                row_front: 9,
                row_back: 21,
                row_side: 3,
                frame_count: 7,
            },
            CharacterKind::Robot => CharacterAnimConfig {
                // Robot: 7 cols x 3 rows, 64x64
                // row0=side, row1=front, row2=back
                cols: 7,
                row_front: 1,
                row_back: 2,
                row_side: 0,
                frame_count: 7,
            },
            CharacterKind::Mage => CharacterAnimConfig {
                // Mage: 7 cols x 3 rows, 64x64
                // row0=side, row1=front, row2=back
                cols: 7,
                row_front: 1,
                row_back: 2,
                row_side: 0,
                frame_count: 7,
            },
        }
    }

    /// Returns the default atlas index (front-facing first frame) for preview
    pub fn preview_index(&self) -> usize {
        let config = self.anim_config();
        config.row_front * config.cols
    }
}

#[derive(Resource, Default)]
pub struct SelectedCharacter(pub CharacterKind);

// === Sprite Assets ===
#[derive(Resource)]
pub struct SpriteAssets {
    pub hero_texture: Handle<Image>,
    pub hero_layout: Handle<TextureAtlasLayout>,
    pub robot_texture: Handle<Image>,
    pub robot_layout: Handle<TextureAtlasLayout>,
    pub mage_texture: Handle<Image>,
    pub mage_layout: Handle<TextureAtlasLayout>,
    pub creatures_texture: Handle<Image>,
    pub creatures_layout: Handle<TextureAtlasLayout>,
    pub dog_texture: Handle<Image>,
    pub dog_layout: Handle<TextureAtlasLayout>,
}

// === Weapon Shader Material Handles ===
#[derive(Resource)]
#[allow(dead_code)]
pub struct WeaponShaderHandles {
    pub orbit_shield: [Handle<crate::systems::shader_materials::OrbitShieldMaterial>; 8],
    pub projectile: [Handle<crate::systems::shader_materials::ProjectileMaterial>; 8],
    pub lightning: Handle<crate::systems::shader_materials::LightningMaterial>,
    pub flame_aura: [Handle<crate::systems::shader_materials::FlameAuraMaterial>; 8],
    pub boomerang: [Handle<crate::systems::shader_materials::BoomerangMaterial>; 8],
    pub bone: Handle<crate::systems::shader_materials::BoomerangMaterial>,
    pub holy_water: Handle<crate::systems::shader_materials::HolyWaterMaterial>,
    pub updown: Handle<crate::systems::shader_materials::UpDownMaterial>,
    pub phiera: [Handle<crate::systems::shader_materials::PhieraMaterial>; 8],
    pub boss_laser: Handle<crate::systems::shader_materials::BossLaserMaterial>,
    pub orbit_shield_quad: Handle<Mesh>,
    pub projectile_quad: Handle<Mesh>,
    pub lightning_quad: Handle<Mesh>,
    pub flame_aura_quad: Handle<Mesh>,
    pub boomerang_quad: Handle<Mesh>,
    pub bone_quad: Handle<Mesh>,
    pub holy_water_quad: Handle<Mesh>,
    pub updown_quad: Handle<Mesh>,
    pub phiera_quad: Handle<Mesh>,
    pub whip_quad: Handle<Mesh>,
    pub boss_laser_quad: Handle<Mesh>,
}

// === HUD Bar Shader Handles ===
#[derive(Resource)]
pub struct HudBarHandles {
    pub health_bar_mat: Handle<crate::systems::shader_materials::HealthBarMaterial>,
    pub xp_bar_mat: Handle<crate::systems::shader_materials::XpBarMaterial>,
    pub health_bar_mesh: Handle<Mesh>,
    pub xp_bar_mesh: Handle<Mesh>,
    pub health_bg_mat: Handle<ColorMaterial>,
    pub xp_bg_mat: Handle<ColorMaterial>,
    pub health_bg_mesh: Handle<Mesh>,
    pub xp_bg_mesh: Handle<Mesh>,
}

// === Cached Mesh/Material Handles ===
#[derive(Resource)]
#[allow(dead_code)]
pub struct GameMeshes {
    pub player: Handle<Mesh>,
    pub enemy_basic: Handle<Mesh>,
    pub enemy_fast: Handle<Mesh>,
    pub enemy_tank: Handle<Mesh>,
    pub enemy_swarm: Handle<Mesh>,
    pub enemy_flock: Handle<Mesh>,
    pub projectile: Handle<Mesh>,
    pub xp_gem: Handle<Mesh>,
    pub orbit_shield: Handle<Mesh>,
    pub pickup: Handle<Mesh>,
    pub boomerang: Handle<Mesh>,
    pub healing_dot: Handle<Mesh>,
}

#[derive(Resource)]
#[allow(dead_code)]
pub struct GameMaterials {
    pub player: Handle<ColorMaterial>,
    pub player_hit: Handle<ColorMaterial>,
    pub enemy_basic: Handle<ColorMaterial>,
    pub enemy_fast: Handle<ColorMaterial>,
    pub enemy_tank: Handle<ColorMaterial>,
    pub enemy_swarm: Handle<ColorMaterial>,
    pub enemy_flock: Handle<ColorMaterial>,
    pub projectile: Handle<ColorMaterial>,
    pub xp_gem: Handle<ColorMaterial>,
    pub orbit_shield: Handle<ColorMaterial>,
    pub boomerang: Handle<ColorMaterial>,
    pub pickup_orbit: Handle<ColorMaterial>,
    pub pickup_burst: Handle<ColorMaterial>,
    pub pickup_lightning: Handle<ColorMaterial>,
    pub pickup_flame: Handle<ColorMaterial>,
    pub pickup_boomerang: Handle<ColorMaterial>,
    pub pickup_holy_water: Handle<ColorMaterial>,
    pub pickup_updown: Handle<ColorMaterial>,
    pub pickup_phiera: Handle<ColorMaterial>,
    pub pickup_whip: Handle<ColorMaterial>,
    pub healing_dot: Handle<ColorMaterial>,
}
