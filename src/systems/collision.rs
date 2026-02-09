use crate::components::EnemyKind;

pub fn enemy_radius_for(kind: EnemyKind) -> f32 {
    match kind {
        EnemyKind::Basic => 12.0,
        EnemyKind::Fast => 8.0,
        EnemyKind::Tank => 14.0,
        EnemyKind::Swarm => 5.0,
    }
}

pub mod weapon_radii {
    pub const ORBIT_SHIELD: f32 = 8.0;
    pub const PROJECTILE: f32 = 4.0;
    pub const BOOMERANG: f32 = 6.0;
    pub const BONE: f32 = 6.0;
    pub const UPDOWN: f32 = 20.0;
}
