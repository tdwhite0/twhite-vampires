use bevy::prelude::*;

// === Player ===
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct DamageCooldown(pub f32);

#[derive(Component)]
pub struct Experience {
    pub current: f32,
    pub next_level: f32,
    pub level: u32,
}

#[derive(Component)]
pub struct MoveSpeed(pub f32);

// === Enemies ===
#[derive(Component)]
pub struct Enemy;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyKind {
    Basic,
    Fast,
    Tank,
    Swarm,
    Flock,
}

#[derive(Component)]
pub struct FlockMember {
    pub flock_id: u32,
}

#[derive(Component)]
pub struct EnemyType(pub EnemyKind);

#[derive(Component)]
pub struct EnemyHealth(pub f32);

#[derive(Component)]
pub struct ContactDamage(pub f32);

#[derive(Component)]
pub struct EnemySpeed(pub f32);

// === Weapon Level Tables ===
pub struct WeaponLevelDef {
    pub damage: f32,
    pub cooldown: f32,
    pub area: f32,
    pub count: u32,
}

pub const ORBIT_SHIELD_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 15.0, cooldown: 0.0, area: 60.0, count: 1 },
    WeaponLevelDef { damage: 18.0, cooldown: 0.0, area: 65.0, count: 2 },
    WeaponLevelDef { damage: 22.0, cooldown: 0.0, area: 70.0, count: 2 },
    WeaponLevelDef { damage: 26.0, cooldown: 0.0, area: 80.0, count: 3 },
    WeaponLevelDef { damage: 32.0, cooldown: 0.0, area: 90.0, count: 4 },
    WeaponLevelDef { damage: 38.0, cooldown: 0.0, area: 100.0, count: 5 },
    WeaponLevelDef { damage: 45.0, cooldown: 0.0, area: 110.0, count: 6 },
    WeaponLevelDef { damage: 55.0, cooldown: 0.0, area: 130.0, count: 8 },
];

pub const PROJECTILE_BURST_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 10.0, cooldown: 2.0, area: 0.0, count: 4 },
    WeaponLevelDef { damage: 12.0, cooldown: 1.8, area: 0.0, count: 5 },
    WeaponLevelDef { damage: 15.0, cooldown: 1.6, area: 0.0, count: 6 },
    WeaponLevelDef { damage: 18.0, cooldown: 1.4, area: 0.0, count: 8 },
    WeaponLevelDef { damage: 22.0, cooldown: 1.2, area: 0.0, count: 8 },
    WeaponLevelDef { damage: 28.0, cooldown: 1.0, area: 0.0, count: 10 },
    WeaponLevelDef { damage: 35.0, cooldown: 0.8, area: 0.0, count: 12 },
    WeaponLevelDef { damage: 45.0, cooldown: 0.5, area: 0.0, count: 16 },
];

pub const LIGHTNING_ZAP_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 25.0, cooldown: 1.5, area: 200.0, count: 1 },
    WeaponLevelDef { damage: 30.0, cooldown: 1.3, area: 220.0, count: 1 },
    WeaponLevelDef { damage: 35.0, cooldown: 1.1, area: 250.0, count: 2 },
    WeaponLevelDef { damage: 42.0, cooldown: 1.0, area: 280.0, count: 2 },
    WeaponLevelDef { damage: 50.0, cooldown: 0.8, area: 320.0, count: 3 },
    WeaponLevelDef { damage: 60.0, cooldown: 0.6, area: 360.0, count: 4 },
    WeaponLevelDef { damage: 75.0, cooldown: 0.5, area: 400.0, count: 5 },
    WeaponLevelDef { damage: 100.0, cooldown: 0.3, area: 500.0, count: 8 },
];

pub const FLAME_AURA_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 8.0, cooldown: 0.5, area: 60.0, count: 0 },
    WeaponLevelDef { damage: 10.0, cooldown: 0.45, area: 70.0, count: 0 },
    WeaponLevelDef { damage: 13.0, cooldown: 0.4, area: 85.0, count: 0 },
    WeaponLevelDef { damage: 16.0, cooldown: 0.35, area: 100.0, count: 0 },
    WeaponLevelDef { damage: 20.0, cooldown: 0.3, area: 120.0, count: 0 },
    WeaponLevelDef { damage: 26.0, cooldown: 0.25, area: 145.0, count: 0 },
    WeaponLevelDef { damage: 33.0, cooldown: 0.2, area: 175.0, count: 0 },
    WeaponLevelDef { damage: 45.0, cooldown: 0.15, area: 220.0, count: 0 },
];

pub const BOOMERANG_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 15.0, cooldown: 2.5, area: 250.0, count: 1 },
    WeaponLevelDef { damage: 18.0, cooldown: 2.2, area: 280.0, count: 1 },
    WeaponLevelDef { damage: 22.0, cooldown: 2.0, area: 300.0, count: 2 },
    WeaponLevelDef { damage: 28.0, cooldown: 1.7, area: 330.0, count: 2 },
    WeaponLevelDef { damage: 35.0, cooldown: 1.4, area: 360.0, count: 3 },
    WeaponLevelDef { damage: 42.0, cooldown: 1.1, area: 400.0, count: 4 },
    WeaponLevelDef { damage: 52.0, cooldown: 0.8, area: 450.0, count: 5 },
    WeaponLevelDef { damage: 65.0, cooldown: 0.5, area: 500.0, count: 8 },
];

pub const HOLY_WATER_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 6.0, cooldown: 3.0, area: 40.0, count: 1 },
    WeaponLevelDef { damage: 8.0, cooldown: 2.8, area: 55.0, count: 1 },
    WeaponLevelDef { damage: 10.0, cooldown: 2.5, area: 65.0, count: 2 },
    WeaponLevelDef { damage: 13.0, cooldown: 2.2, area: 75.0, count: 2 },
    WeaponLevelDef { damage: 16.0, cooldown: 2.0, area: 90.0, count: 3 },
    WeaponLevelDef { damage: 20.0, cooldown: 1.6, area: 100.0, count: 4 },
    WeaponLevelDef { damage: 25.0, cooldown: 1.2, area: 115.0, count: 5 },
    WeaponLevelDef { damage: 35.0, cooldown: 0.8, area: 140.0, count: 8 },
];

pub const UPDOWN_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 10.0, cooldown: 2.0, area: 80.0, count: 1 },
    WeaponLevelDef { damage: 13.0, cooldown: 1.8, area: 95.0, count: 1 },
    WeaponLevelDef { damage: 16.0, cooldown: 1.6, area: 110.0, count: 2 },
    WeaponLevelDef { damage: 20.0, cooldown: 1.4, area: 130.0, count: 2 },
    WeaponLevelDef { damage: 25.0, cooldown: 1.2, area: 150.0, count: 3 },
    WeaponLevelDef { damage: 32.0, cooldown: 1.0, area: 175.0, count: 4 },
    WeaponLevelDef { damage: 40.0, cooldown: 0.7, area: 200.0, count: 5 },
    WeaponLevelDef { damage: 55.0, cooldown: 0.4, area: 250.0, count: 8 },
];

pub const PHIERA_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 8.0, cooldown: 0.8, area: 0.0, count: 1 },
    WeaponLevelDef { damage: 10.0, cooldown: 0.7, area: 0.0, count: 1 },
    WeaponLevelDef { damage: 12.0, cooldown: 0.6, area: 0.0, count: 1 },
    WeaponLevelDef { damage: 15.0, cooldown: 0.5, area: 0.0, count: 2 },
    WeaponLevelDef { damage: 18.0, cooldown: 0.4, area: 0.0, count: 2 },
    WeaponLevelDef { damage: 22.0, cooldown: 0.3, area: 0.0, count: 3 },
    WeaponLevelDef { damage: 28.0, cooldown: 0.2, area: 0.0, count: 3 },
    WeaponLevelDef { damage: 35.0, cooldown: 0.12, area: 0.0, count: 4 },
];

pub const WHIP_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 10.0, cooldown: 1.35, area: 150.0, count: 1 },
    WeaponLevelDef { damage: 13.0, cooldown: 1.35, area: 150.0, count: 2 },
    WeaponLevelDef { damage: 17.0, cooldown: 1.25, area: 165.0, count: 2 },
    WeaponLevelDef { damage: 20.0, cooldown: 1.25, area: 180.0, count: 2 },
    WeaponLevelDef { damage: 25.0, cooldown: 1.15, area: 180.0, count: 2 },
    WeaponLevelDef { damage: 30.0, cooldown: 1.15, area: 195.0, count: 2 },
    WeaponLevelDef { damage: 35.0, cooldown: 1.05, area: 195.0, count: 2 },
    WeaponLevelDef { damage: 40.0, cooldown: 1.05, area: 210.0, count: 2 },
];

// === Weapon Visual Tables ===
pub struct WeaponVisualDef {
    pub color: [f32; 4],
    pub intensity: f32,
}

pub const ORBIT_SHIELD_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [0.0, 0.800, 0.800, 1.0], intensity: 1.200 },
    WeaponVisualDef { color: [0.1, 0.807, 0.829, 1.0], intensity: 1.386 },
    WeaponVisualDef { color: [0.2, 0.814, 0.857, 1.0], intensity: 1.571 },
    WeaponVisualDef { color: [0.3, 0.821, 0.886, 1.0], intensity: 1.757 },
    WeaponVisualDef { color: [0.4, 0.829, 0.914, 1.0], intensity: 1.943 },
    WeaponVisualDef { color: [0.5, 0.836, 0.943, 1.0], intensity: 2.129 },
    WeaponVisualDef { color: [0.6, 0.843, 0.971, 1.0], intensity: 2.314 },
    WeaponVisualDef { color: [0.7, 0.850, 1.000, 1.0], intensity: 2.500 },
];

pub const PROJECTILE_BURST_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [0.900, 0.900, 1.000, 1.0], intensity: 1.500 },
    WeaponVisualDef { color: [0.914, 0.800, 0.971, 1.0], intensity: 1.714 },
    WeaponVisualDef { color: [0.929, 0.700, 0.943, 1.0], intensity: 1.929 },
    WeaponVisualDef { color: [0.943, 0.600, 0.914, 1.0], intensity: 2.143 },
    WeaponVisualDef { color: [0.957, 0.500, 0.886, 1.0], intensity: 2.357 },
    WeaponVisualDef { color: [0.971, 0.400, 0.857, 1.0], intensity: 2.571 },
    WeaponVisualDef { color: [0.986, 0.300, 0.829, 1.0], intensity: 2.786 },
    WeaponVisualDef { color: [1.000, 0.200, 0.800, 1.0], intensity: 3.000 },
];

pub const LIGHTNING_ZAP_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [0.300, 0.600, 1.000, 1.0], intensity: 2.000 },
    WeaponVisualDef { color: [0.386, 0.643, 1.000, 1.0], intensity: 2.286 },
    WeaponVisualDef { color: [0.471, 0.686, 1.000, 1.0], intensity: 2.571 },
    WeaponVisualDef { color: [0.557, 0.729, 1.000, 1.0], intensity: 2.857 },
    WeaponVisualDef { color: [0.643, 0.771, 1.000, 1.0], intensity: 3.143 },
    WeaponVisualDef { color: [0.729, 0.814, 1.000, 1.0], intensity: 3.429 },
    WeaponVisualDef { color: [0.814, 0.857, 1.000, 1.0], intensity: 3.714 },
    WeaponVisualDef { color: [0.900, 0.900, 1.000, 1.0], intensity: 4.000 },
];

pub const FLAME_AURA_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [1.000, 0.500, 0.000, 0.8], intensity: 1.500 },
    WeaponVisualDef { color: [1.000, 0.557, 0.100, 0.8], intensity: 1.786 },
    WeaponVisualDef { color: [1.000, 0.614, 0.200, 0.8], intensity: 2.071 },
    WeaponVisualDef { color: [1.000, 0.671, 0.300, 0.8], intensity: 2.357 },
    WeaponVisualDef { color: [1.000, 0.729, 0.400, 0.8], intensity: 2.643 },
    WeaponVisualDef { color: [1.000, 0.786, 0.500, 0.8], intensity: 2.929 },
    WeaponVisualDef { color: [1.000, 0.843, 0.600, 0.8], intensity: 3.214 },
    WeaponVisualDef { color: [1.000, 0.900, 0.700, 0.8], intensity: 3.500 },
];

pub const BOOMERANG_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [0.700, 0.300, 1.000, 1.0], intensity: 1.500 },
    WeaponVisualDef { color: [0.743, 0.371, 1.000, 1.0], intensity: 1.714 },
    WeaponVisualDef { color: [0.786, 0.443, 1.000, 1.0], intensity: 1.929 },
    WeaponVisualDef { color: [0.829, 0.514, 1.000, 1.0], intensity: 2.143 },
    WeaponVisualDef { color: [0.871, 0.586, 1.000, 1.0], intensity: 2.357 },
    WeaponVisualDef { color: [0.914, 0.657, 1.000, 1.0], intensity: 2.571 },
    WeaponVisualDef { color: [0.957, 0.729, 1.000, 1.0], intensity: 2.786 },
    WeaponVisualDef { color: [1.000, 0.800, 1.000, 1.0], intensity: 3.000 },
];

pub const HOLY_WATER_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [0.200, 0.900, 0.700, 0.85], intensity: 1.500 },
    WeaponVisualDef { color: [0.243, 0.914, 0.700, 0.85], intensity: 1.714 },
    WeaponVisualDef { color: [0.286, 0.929, 0.700, 0.85], intensity: 1.929 },
    WeaponVisualDef { color: [0.329, 0.943, 0.700, 0.85], intensity: 2.143 },
    WeaponVisualDef { color: [0.371, 0.957, 0.700, 0.85], intensity: 2.357 },
    WeaponVisualDef { color: [0.414, 0.971, 0.700, 0.85], intensity: 2.571 },
    WeaponVisualDef { color: [0.457, 0.986, 0.700, 0.85], intensity: 2.786 },
    WeaponVisualDef { color: [0.500, 1.000, 0.700, 0.85], intensity: 3.000 },
];

pub const UPDOWN_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [0.800, 0.200, 0.900, 0.9], intensity: 1.800 },
    WeaponVisualDef { color: [0.829, 0.286, 0.914, 0.9], intensity: 2.043 },
    WeaponVisualDef { color: [0.857, 0.371, 0.929, 0.9], intensity: 2.286 },
    WeaponVisualDef { color: [0.886, 0.457, 0.943, 0.9], intensity: 2.529 },
    WeaponVisualDef { color: [0.914, 0.543, 0.957, 0.9], intensity: 2.771 },
    WeaponVisualDef { color: [0.943, 0.629, 0.971, 0.9], intensity: 3.014 },
    WeaponVisualDef { color: [0.971, 0.714, 0.986, 0.9], intensity: 3.257 },
    WeaponVisualDef { color: [1.000, 0.800, 1.000, 0.9], intensity: 3.500 },
];

pub const PHIERA_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [1.000, 0.200, 0.100, 1.0], intensity: 1.500 },
    WeaponVisualDef { color: [1.000, 0.250, 0.120, 1.0], intensity: 1.700 },
    WeaponVisualDef { color: [1.000, 0.300, 0.150, 1.0], intensity: 1.900 },
    WeaponVisualDef { color: [1.000, 0.350, 0.180, 1.0], intensity: 2.100 },
    WeaponVisualDef { color: [1.000, 0.400, 0.200, 1.0], intensity: 2.300 },
    WeaponVisualDef { color: [1.000, 0.450, 0.250, 1.0], intensity: 2.600 },
    WeaponVisualDef { color: [1.000, 0.500, 0.300, 1.0], intensity: 2.900 },
    WeaponVisualDef { color: [1.000, 0.600, 0.350, 1.0], intensity: 3.200 },
];

pub const WHIP_VISUALS: [WeaponVisualDef; 8] = [
    WeaponVisualDef { color: [0.900, 0.350, 0.150, 1.0], intensity: 1.500 },
    WeaponVisualDef { color: [0.920, 0.380, 0.170, 1.0], intensity: 1.700 },
    WeaponVisualDef { color: [0.940, 0.410, 0.190, 1.0], intensity: 1.900 },
    WeaponVisualDef { color: [0.950, 0.440, 0.210, 1.0], intensity: 2.100 },
    WeaponVisualDef { color: [0.960, 0.480, 0.230, 1.0], intensity: 2.300 },
    WeaponVisualDef { color: [0.970, 0.520, 0.260, 1.0], intensity: 2.600 },
    WeaponVisualDef { color: [0.980, 0.560, 0.290, 1.0], intensity: 2.900 },
    WeaponVisualDef { color: [1.000, 0.620, 0.330, 1.0], intensity: 3.200 },
];

// === Weapons ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponKind {
    OrbitShield,
    ProjectileBurst,
    LightningZap,
    FlameAura,
    Boomerang,
    HolyWater,
    UpDown,
    Phiera,
    Whip,
}

impl WeaponKind {
    pub fn level_def(&self, level: u32) -> &'static WeaponLevelDef {
        let idx = (level.clamp(1, 8) - 1) as usize;
        match self {
            WeaponKind::OrbitShield => &ORBIT_SHIELD_LEVELS[idx],
            WeaponKind::ProjectileBurst => &PROJECTILE_BURST_LEVELS[idx],
            WeaponKind::LightningZap => &LIGHTNING_ZAP_LEVELS[idx],
            WeaponKind::FlameAura => &FLAME_AURA_LEVELS[idx],
            WeaponKind::Boomerang => &BOOMERANG_LEVELS[idx],
            WeaponKind::HolyWater => &HOLY_WATER_LEVELS[idx],
            WeaponKind::UpDown => &UPDOWN_LEVELS[idx],
            WeaponKind::Phiera => &PHIERA_LEVELS[idx],
            WeaponKind::Whip => &WHIP_LEVELS[idx],
        }
    }

    pub fn visual_def(&self, level: u32) -> &'static WeaponVisualDef {
        let idx = (level.clamp(1, 8) - 1) as usize;
        match self {
            WeaponKind::OrbitShield => &ORBIT_SHIELD_VISUALS[idx],
            WeaponKind::ProjectileBurst => &PROJECTILE_BURST_VISUALS[idx],
            WeaponKind::LightningZap => &LIGHTNING_ZAP_VISUALS[idx],
            WeaponKind::FlameAura => &FLAME_AURA_VISUALS[idx],
            WeaponKind::Boomerang => &BOOMERANG_VISUALS[idx],
            WeaponKind::HolyWater => &HOLY_WATER_VISUALS[idx],
            WeaponKind::UpDown => &UPDOWN_VISUALS[idx],
            WeaponKind::Phiera => &PHIERA_VISUALS[idx],
            WeaponKind::Whip => &WHIP_VISUALS[idx],
        }
    }

    pub fn all() -> &'static [WeaponKind] {
        &[
            WeaponKind::OrbitShield,
            WeaponKind::ProjectileBurst,
            WeaponKind::LightningZap,
            WeaponKind::FlameAura,
            WeaponKind::Boomerang,
            WeaponKind::HolyWater,
            WeaponKind::UpDown,
            WeaponKind::Phiera,
            WeaponKind::Whip,
        ]
    }

    pub fn display_name(&self) -> &str {
        match self {
            WeaponKind::OrbitShield => "Orbit Shield",
            WeaponKind::ProjectileBurst => "Projectile Burst",
            WeaponKind::LightningZap => "Lightning Zap",
            WeaponKind::FlameAura => "Flame Aura",
            WeaponKind::Boomerang => "Boomerang",
            WeaponKind::HolyWater => "Holy Water",
            WeaponKind::UpDown => "UpDown",
            WeaponKind::Phiera => "Phiera Der Tuphello",
            WeaponKind::Whip => "Whip",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            WeaponKind::OrbitShield => Color::srgb(0.0, 1.0, 1.0),
            WeaponKind::ProjectileBurst => Color::srgb(0.9, 0.9, 1.0),
            WeaponKind::LightningZap => Color::srgb(0.3, 0.5, 1.0),
            WeaponKind::FlameAura => Color::srgb(1.0, 0.5, 0.0),
            WeaponKind::Boomerang => Color::srgb(0.7, 0.3, 1.0),
            WeaponKind::HolyWater => Color::srgb(0.2, 0.9, 0.7),
            WeaponKind::UpDown => Color::srgb(0.8, 0.2, 0.9),
            WeaponKind::Phiera => Color::srgb(1.0, 0.3, 0.1),
            WeaponKind::Whip => Color::srgb(0.9, 0.35, 0.15),
        }
    }
}

#[derive(Component)]
pub struct OrbitShield {
    pub orbit_index: u32,
    pub hit_cooldown: f32,
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime: f32,
    pub speed: f32,
    pub direction: Vec2,
    pub piercing: bool,
    pub hit_enemies: Vec<Entity>,
    pub source: WeaponKind,
}

#[derive(Component)]
pub struct BoomerangProjectile {
    pub damage: f32,
    pub elapsed: f32,
    pub total_time: f32,
    pub target_dir: Vec2,
    #[allow(dead_code)]
    pub origin: Vec2,
    #[allow(dead_code)]
    pub max_range: f32,
    pub hit_enemies: Vec<Entity>,
}

// === Pet ===
#[derive(Component)]
pub struct PetDog {
    pub bone_timer: f32,
    pub bone_cooldown: f32,
    pub bone_damage: f32,
}

#[derive(Component)]
pub struct BoneProjectile {
    pub damage: f32,
    pub elapsed: f32,
    pub total_time: f32,
    pub target_dir: Vec2,
    #[allow(dead_code)]
    pub owner_pos: Vec2,
    pub hit_enemies: Vec<Entity>,
}

// === Holy Water Zone ===
#[derive(Component)]
pub struct HolyWaterZone {
    pub damage: f32,
    pub tick_timer: f32,
    pub tick_rate: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub radius: f32,
}

// === UpDown Wave ===
#[derive(Component)]
pub struct UpDownWave {
    pub damage: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub direction: f32, // 1.0 = up, -1.0 = down
    pub speed: f32,
    pub hit_enemies: Vec<Entity>,
}

// === Whip Slash ===
#[derive(Component)]
pub struct WhipSlash {
    pub damage: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub delay: f32,
    pub direction: Vec2,
    pub half_length: f32,
    pub half_width: f32,
    pub hit_enemies: Vec<Entity>,
}

// === Boss ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossKind {
    Eyeball,
    DragonLord,
    Necromancer,
    SlimeKing,
}

impl BossKind {
    pub fn display_name(&self) -> &str {
        match self {
            BossKind::Eyeball => "EYEBALL TERROR",
            BossKind::DragonLord => "DRAGON LORD",
            BossKind::Necromancer => "NECROMANCER",
            BossKind::SlimeKing => "SLIME KING",
        }
    }

    pub fn bar_color(&self) -> Color {
        match self {
            BossKind::Eyeball => Color::srgb(0.2, 0.8, 0.3),
            BossKind::DragonLord => Color::srgb(0.9, 0.4, 0.1),
            BossKind::Necromancer => Color::srgb(0.6, 0.2, 0.9),
            BossKind::SlimeKing => Color::srgb(0.3, 0.9, 0.3),
        }
    }

    pub fn name_color(&self) -> Color {
        match self {
            BossKind::Eyeball => Color::srgb(0.3, 1.0, 0.4),
            BossKind::DragonLord => Color::srgb(1.0, 0.5, 0.2),
            BossKind::Necromancer => Color::srgb(0.7, 0.3, 1.0),
            BossKind::SlimeKing => Color::srgb(0.4, 1.0, 0.4),
        }
    }
}

#[derive(Component)]
pub struct Boss {
    pub kind: BossKind,
}

#[derive(Component)]
pub struct BossHealth {
    pub current: f32,
    pub max: f32,
}

// === Eyeball Boss ===
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BossLaserPhase {
    Idle,
    Charging { timer: f32, angle: f32 },
    Firing { timer: f32, angle: f32, sweep_dir: f32 },
    Cooldown { timer: f32 },
}

#[derive(Component)]
pub struct BossLaser {
    pub phase: BossLaserPhase,
    pub range: f32,
    pub damage_per_tick: f32,
    pub hit_cooldown: f32,
}

#[derive(Component)]
pub struct BossLaserBeam {
    pub owner: Entity,
}

// === Dragon Lord Boss ===
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragonBreathPhase {
    Idle,
    Windup { timer: f32 },
    Firing { timer: f32, shots_fired: u32 , shot_timer: f32 },
    Cooldown { timer: f32 },
}

#[derive(Component)]
pub struct DragonBreath {
    pub phase: DragonBreathPhase,
}

#[derive(Component)]
pub struct DragonFireball {
    pub damage: f32,
    pub speed: f32,
    pub direction: Vec2,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct DragonFireZone {
    pub damage: f32,
    pub tick_timer: f32,
    pub lifetime: f32,
    pub radius: f32,
}

// === Necromancer Boss ===
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NecroPhase {
    Idle { timer: f32 },
    Casting { timer: f32 },
}

#[derive(Component)]
pub struct NecromancerMagic {
    pub phase: NecroPhase,
    pub attack_index: u32,
}

#[derive(Component)]
pub struct NecroSummon {
    pub owner: Entity,
}

#[derive(Component)]
pub struct NecroOrb {
    pub damage: f32,
    pub speed: f32,
    pub turn_rate: f32,
    pub lifetime: f32,
    pub direction: Vec2,
}

// === Slime King Boss ===
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlimePhase {
    Idle { timer: f32 },
    Windup { timer: f32 },
    Jumping { timer: f32, target: Vec2 },
    Landing { timer: f32 },
    Cooldown { timer: f32 },
}

#[derive(Component)]
pub struct SlimeKingAbility {
    pub phase: SlimePhase,
}

#[derive(Component)]
pub struct SlimeShockwave {
    pub damage: f32,
    pub current_radius: f32,
    pub max_radius: f32,
    pub lifetime: f32,
    pub hit_player: bool,
}

#[derive(Component)]
pub struct SlimeSplit {
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct BossHealthBarBg;

#[derive(Component)]
pub struct BossHealthBarFill;

#[derive(Component)]
pub struct BossNameText;

#[derive(Component)]
pub struct BossHealthBarOwner(pub Entity);

// === Pickups ===
#[derive(Component)]
pub struct WeaponPickup(pub WeaponKind);

#[derive(Component)]
pub struct XpGem(pub f32);

#[derive(Component)]
pub struct HealingDot(pub f32);

/// Once a pickup enters magnet range, it stays locked on. Tracks elapsed chase time
/// so the pickup accelerates if the player outruns it.
#[derive(Component)]
pub struct MagnetLocked(pub f32);

// === Character Animation Config ===
#[derive(Component, Clone, Copy)]
pub struct CharacterAnimConfig {
    pub cols: usize,
    pub row_front: usize,
    pub row_back: usize,
    pub row_side: usize,
    pub frame_count: usize,
}

// === Sprite Animation ===
#[derive(Component)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

// === Shader Effect Entities ===
#[derive(Component)]
pub struct FlameAuraEntity;

#[derive(Component)]
pub struct LightningBoltEntity {
    pub lifetime: f32,
}

// === Visual Effects ===
#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct FloatingText {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub base_color: Vec3,
}

// === Abilities ===
#[derive(Component)]
pub struct Dashing {
    pub direction: Vec2,
    pub remaining: f32,
    pub speed: f32,
    pub damage: f32,
    pub hit_enemies: Vec<Entity>,
}

// === Notification Toasts ===
#[derive(Debug, Clone, Copy)]
pub enum NotificationType {
    Weapon,
    Boss,
    Buff,
    Info,
}

impl NotificationType {
    pub fn color(&self) -> Color {
        match self {
            NotificationType::Weapon => Color::srgb(0.2, 1.0, 0.3),
            NotificationType::Boss => Color::srgb(1.0, 0.4, 0.2),
            NotificationType::Buff => Color::srgb(0.3, 0.8, 1.0),
            NotificationType::Info => Color::srgb(1.0, 1.0, 1.0),
        }
    }
}

#[derive(Message)]
pub struct NotificationEvent {
    pub message: String,
    pub kind: NotificationType,
}

#[derive(Component)]
pub struct NotificationToast {
    pub age: f32,
    pub lifetime: f32,
    pub slot: u32,
}

// === UI Markers ===
#[derive(Component)]
pub struct HudCamera;

#[derive(Component)]
pub struct HudBarAnchor;

#[derive(Component)]
pub struct HealthBarShader;

#[derive(Component)]
pub struct XpBarShader;

#[derive(Component)]
pub struct KillCountText;

#[derive(Component)]
pub struct TimerText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct TitleScreenEntity;

#[derive(Component)]
pub struct CharacterSelectBox(pub usize);

#[derive(Component)]
pub struct GameOverEntity;

#[derive(Component)]
pub struct GameOverAnimTimer {
    pub elapsed: f32,
}

#[derive(Component)]
pub struct GameOverStatRow(pub u32);

#[derive(Component)]
pub struct GameOverRestartPrompt;

#[derive(Component)]
pub struct LevelUpEntity;

#[derive(Component)]
pub struct GameEntity;

#[derive(Component)]
pub struct HudEntity;

#[derive(Component)]
pub struct UpgradeButton(pub UpgradeKind);

#[derive(Component)]
pub struct AbilitySlotUI(pub usize);

#[derive(Component)]
pub struct AbilityCooldownFill(pub usize);

#[derive(Component)]
pub struct AbilityCooldownFlash {
    pub index: usize,
    pub remaining: f32,
}

// === Settings Menu ===
#[derive(Component)]
pub struct SettingsGearButton;

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component)]
pub struct SettingsInvincibleToggle;

#[derive(Component)]
pub struct SettingsInvincibleCheck;

#[derive(Component)]
pub struct SettingsWeaponButton(pub WeaponKind);

#[derive(Component)]
pub struct SettingsWeaponLevelButton(pub WeaponKind, pub u32);

#[derive(Component)]
pub struct SettingsDamageButton(pub f32);

#[derive(Component)]
pub struct SettingsDamageCheck(pub f32);

#[derive(Component)]
pub struct SettingsCrtToggle;

#[derive(Component)]
pub struct SettingsCrtCheck;

#[derive(Component)]
pub struct SettingsTabButton(pub crate::resources::SettingsTab);

#[derive(Component)]
pub struct SettingsSpawnBossButton;

#[derive(Component)]
pub struct SettingsBossHitboxToggle;

#[derive(Component)]
pub struct SettingsBossHitboxCheck;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum SettingsButtonKind {
    Invincible,
    Weapon(WeaponKind),
    WeaponLevel(WeaponKind, u32),
    Damage(f32),
    Gear,
    MusicTrack(usize),
    MusicMute,
    Tab(crate::resources::SettingsTab),
    SpawnBoss,
    BossHitbox,
    Crt,
    FontCycle,
}

#[derive(Component)]
pub struct SettingsFontCycleButton;

#[derive(Component)]
pub struct SpawnRateSliderTrack;

#[derive(Component)]
pub struct SpawnRateSliderFill;

#[derive(Component)]
pub struct SpawnRateText;

// === CRT Slider UI ===
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrtParam {
    Curvature,
    ChromaticAberration,
    ScanlineIntensity,
    PhosphorIntensity,
    VignetteStrength,
}

impl CrtParam {
    pub const ALL: [CrtParam; 5] = [
        CrtParam::Curvature,
        CrtParam::ChromaticAberration,
        CrtParam::ScanlineIntensity,
        CrtParam::PhosphorIntensity,
        CrtParam::VignetteStrength,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            CrtParam::Curvature => "Curvature",
            CrtParam::ChromaticAberration => "Chromatic Aberr.",
            CrtParam::ScanlineIntensity => "Scanlines",
            CrtParam::PhosphorIntensity => "Phosphor",
            CrtParam::VignetteStrength => "Vignette",
        }
    }

    /// Returns (min, max) range for the slider
    pub fn range(&self) -> (f32, f32) {
        match self {
            CrtParam::Curvature => (0.0, 0.5),
            CrtParam::ChromaticAberration => (0.0, 0.015),
            CrtParam::ScanlineIntensity => (0.0, 1.0),
            CrtParam::PhosphorIntensity => (0.0, 1.0),
            CrtParam::VignetteStrength => (0.0, 4.0),
        }
    }

    pub fn format_value(&self, value: f32) -> String {
        match self {
            CrtParam::ChromaticAberration => format!("{:.4}", value),
            _ => format!("{:.2}", value),
        }
    }
}

#[derive(Component)]
pub struct CrtSliderTrack(pub CrtParam);

#[derive(Component)]
pub struct CrtSliderFill(pub CrtParam);

#[derive(Component)]
pub struct CrtSliderText(pub CrtParam);

// === Music UI ===
#[derive(Component)]
pub struct MusicPlayer;

#[derive(Component)]
pub struct MusicTrackButton(pub usize);

#[derive(Component)]
pub struct MusicMuteButton;

#[derive(Component)]
pub struct MusicVolumeSliderTrack;

#[derive(Component)]
pub struct MusicVolumeSliderFill;

#[derive(Component)]
pub struct MusicVolumeText;

// === Upgrades ===
#[derive(Debug, Clone, Copy)]
pub enum UpgradeKind {
    NewWeapon(WeaponKind),
    LevelUpWeapon(WeaponKind),
    IncreaseMaxHealth,
    IncreaseMoveSpeed,
    HealPlayer,
    IncreasePickupRange,
}

impl UpgradeKind {
    pub fn name(&self) -> String {
        match self {
            UpgradeKind::NewWeapon(kind) => format!("New: {}", kind.display_name()),
            UpgradeKind::LevelUpWeapon(kind) => format!("{} Level Up", kind.display_name()),
            UpgradeKind::IncreaseMaxHealth => "Max Health +20".into(),
            UpgradeKind::IncreaseMoveSpeed => "Move Speed +15%".into(),
            UpgradeKind::HealPlayer => "Heal 30%".into(),
            UpgradeKind::IncreasePickupRange => "Pickup Range +25%".into(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            UpgradeKind::NewWeapon(kind) => format!("Gain {}", kind.display_name()),
            UpgradeKind::LevelUpWeapon(_) => "Increase weapon level".into(),
            UpgradeKind::IncreaseMaxHealth => "Increase max HP by 20".into(),
            UpgradeKind::IncreaseMoveSpeed => "Move 15% faster".into(),
            UpgradeKind::HealPlayer => "Heal 30% of max HP".into(),
            UpgradeKind::IncreasePickupRange => "Collect pickups from farther away".into(),
        }
    }
}
