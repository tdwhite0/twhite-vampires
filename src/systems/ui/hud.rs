use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;
use crate::systems::shader_materials::{HealthBarMaterial, XpBarMaterial};

// === Arena Grid ===
pub fn draw_arena_grid(
    mut gizmos: Gizmos,
    camera_query: Query<&Transform, (With<Camera2d>, Without<HudCamera>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_transform.translation.truncate();
    let grid_size = 100.0;
    let grid_color = Color::srgba(0.15, 0.15, 0.25, 0.5);
    let extent = 800.0;

    let start_x = ((camera_pos.x - extent) / grid_size).floor() * grid_size;
    let end_x = ((camera_pos.x + extent) / grid_size).ceil() * grid_size;
    let start_y = ((camera_pos.y - extent) / grid_size).floor() * grid_size;
    let end_y = ((camera_pos.y + extent) / grid_size).ceil() * grid_size;

    let mut x = start_x;
    while x <= end_x {
        gizmos.line_2d(Vec2::new(x, start_y), Vec2::new(x, end_y), grid_color);
        x += grid_size;
    }

    let mut y = start_y;
    while y <= end_y {
        gizmos.line_2d(Vec2::new(start_x, y), Vec2::new(end_x, y), grid_color);
        y += grid_size;
    }
}

// === Weapon Visual Effects ===
// Flame aura and lightning are now rendered via shader entities (see weapons.rs)
// This function is kept for any remaining gizmo-based effects
pub fn draw_weapon_effects(
    _gizmos: Gizmos,
    _player_query: Query<&Transform, With<Player>>,
    _weapons: Res<PlayerWeapons>,
) {
    // Shader entities handle flame aura and lightning visuals now
}

// === HUD Updates (text labels only) ===
pub fn update_hud(
    player_query: Query<(&Health, &Experience), With<Player>>,
    mut kill_text: Query<&mut Text, (With<KillCountText>, Without<TimerText>, Without<LevelText>)>,
    mut timer_text: Query<&mut Text, (With<TimerText>, Without<KillCountText>, Without<LevelText>)>,
    mut level_text: Query<&mut Text, (With<LevelText>, Without<KillCountText>, Without<TimerText>)>,
    mut stats: ResMut<GameStats>,
    time: Res<Time>,
) {
    stats.time_survived += time.delta_secs();

    let Ok((_, xp)) = player_query.single() else {
        return;
    };

    // Kill count
    if let Ok(mut text) = kill_text.single_mut() {
        **text = format!("Kills: {}", stats.enemies_killed);
    }

    // Timer
    if let Ok(mut text) = timer_text.single_mut() {
        let secs = stats.time_survived as u32;
        let mins = secs / 60;
        let secs = secs % 60;
        **text = format!("{}:{:02}", mins, secs);
    }

    // Level
    if let Ok(mut text) = level_text.single_mut() {
        **text = format!("Level {}", xp.level);
    }
}

// === Shader Bar Updates (fill_percent uniforms) ===
pub fn update_shader_bars(
    player_query: Query<(&Health, &Experience), With<Player>>,
    health_query: Query<&MeshMaterial2d<HealthBarMaterial>, With<HealthBarShader>>,
    xp_query: Query<&MeshMaterial2d<XpBarMaterial>, With<XpBarShader>>,
    mut health_mats: ResMut<Assets<HealthBarMaterial>>,
    mut xp_mats: ResMut<Assets<XpBarMaterial>>,
) {
    // HUD camera is fixed at (0,0) — no positioning needed

    let Ok((health, xp)) = player_query.single() else { return; };

    // Update health bar fill_percent
    if let Ok(mat_handle) = health_query.single() {
        if let Some(mat) = health_mats.get_mut(&mat_handle.0) {
            mat.data.fill_percent = (health.current / health.max).clamp(0.0, 1.0);
        }
    }

    // Update XP bar fill_percent
    if let Ok(mat_handle) = xp_query.single() {
        if let Some(mat) = xp_mats.get_mut(&mat_handle.0) {
            mat.data.fill_percent = (xp.current / xp.next_level).clamp(0.0, 1.0);
        }
    }
}

// === Floating Text ===
pub fn update_floating_text(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut FloatingText, &mut TextColor)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut ft, mut color) in query.iter_mut() {
        ft.lifetime -= time.delta_secs();
        if ft.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.x += ft.velocity.x * time.delta_secs();
        transform.translation.y += ft.velocity.y * time.delta_secs();

        // Fade out
        let alpha = (ft.lifetime / ft.max_lifetime).clamp(0.0, 1.0);
        *color = TextColor(Color::srgba(ft.base_color.x, ft.base_color.y, ft.base_color.z, alpha));
    }
}

// === Ability Slot UI Update ===
pub fn update_ability_ui(
    abilities: Res<PlayerAbilities>,
    mut cooldown_query: Query<(&AbilityCooldownFill, &mut Node)>,
) {
    for (fill, mut node) in cooldown_query.iter_mut() {
        if let Some(ability) = abilities.abilities.get(fill.0) {
            let frac = ability.cooldown_fraction();
            node.height = Val::Percent(frac * 100.0);
        }
    }
}

pub fn update_ability_cooldown_flash(
    time: Res<Time>,
    mut query: Query<(&mut AbilityCooldownFlash, &mut BackgroundColor)>,
) {
    let dt = time.delta_secs();
    for (mut flash, mut bg) in query.iter_mut() {
        if flash.remaining > 0.0 {
            flash.remaining = (flash.remaining - dt).max(0.0);
            let alpha = flash.remaining / 0.3;
            *bg = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, alpha * 0.6));
        }
    }
}

// === Boss Hitbox Debug Drawing ===
pub fn draw_boss_hitboxes(
    mut gizmos: Gizmos,
    debug: Res<DebugSettings>,
    boss_query: Query<(&Transform, &Boss, Option<&BossLaser>)>,
    player_query: Query<&Transform, With<Player>>,
    orbit_query: Query<&Transform, (With<OrbitShield>, Without<Boss>, Without<Player>)>,
    projectile_query: Query<&Transform, (With<Projectile>, Without<Boss>, Without<Player>, Without<OrbitShield>)>,
    boomerang_query: Query<&Transform, (With<BoomerangProjectile>, Without<Boss>, Without<Player>, Without<OrbitShield>, Without<Projectile>)>,
    bone_query: Query<&Transform, (With<BoneProjectile>, Without<Boss>, Without<Player>, Without<OrbitShield>, Without<Projectile>, Without<BoomerangProjectile>)>,
    holy_water_query: Query<(&Transform, &HolyWaterZone)>,
    weapons: Res<PlayerWeapons>,
) {
    if !debug.show_boss_hitboxes {
        return;
    }

    let boss_color = Color::srgba(1.0, 0.2, 0.2, 0.8);
    let boss_contact_color = Color::srgba(1.0, 0.5, 0.2, 0.4);
    let player_color = Color::srgba(0.2, 1.0, 0.3, 0.7);
    let orbit_color = Color::srgba(0.0, 1.0, 1.0, 0.7);
    let projectile_color = Color::srgba(0.9, 0.9, 1.0, 0.7);
    let boomerang_color = Color::srgba(0.7, 0.3, 1.0, 0.7);
    let bone_color = Color::srgba(1.0, 0.95, 0.8, 0.7);
    let flame_color = Color::srgba(1.0, 0.5, 0.0, 0.4);
    let holy_water_color = Color::srgba(0.2, 0.9, 0.7, 0.5);
    let laser_color = Color::srgba(0.2, 1.0, 0.3, 0.5);

    // Boss collision circles
    for (boss_transform, boss, laser) in boss_query.iter() {
        let boss_pos = boss_transform.translation.truncate();
        let radius = crate::systems::boss::boss_radius(boss.kind);

        // Main collision radius
        gizmos.circle_2d(boss_pos, radius, boss_color);
        // Outer ring showing contact damage range (boss_radius + player_radius)
        gizmos.circle_2d(boss_pos, radius + 15.0, boss_contact_color);

        // Laser beam hitbox visualization (eyeball only)
        if let Some(laser) = laser {
            if let BossLaserPhase::Firing { angle, .. } = laser.phase {
                let iris_offset = Vec2::new(42.0, 18.0);
                let iris_pos = boss_pos + iris_offset;
                let beam_dir = Vec2::new(angle.cos(), angle.sin());
                let beam_perp = Vec2::new(-beam_dir.y, beam_dir.x);
                let beam_half_width = 25.0;
                let beam_length = 400.0;

                let start_l = iris_pos + beam_perp * beam_half_width;
                let start_r = iris_pos - beam_perp * beam_half_width;
                let end_l = start_l + beam_dir * beam_length;
                let end_r = start_r + beam_dir * beam_length;

                gizmos.line_2d(start_l, end_l, laser_color);
                gizmos.line_2d(start_r, end_r, laser_color);
                gizmos.line_2d(end_l, end_r, laser_color);
            }
        }
    }

    // Player collision circle
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation.truncate();
        gizmos.circle_2d(player_pos, 15.0, player_color);

        // Flame aura effective range (if equipped)
        if let Some(flame) = weapons.get(WeaponKind::FlameAura) {
            gizmos.circle_2d(player_pos, flame.area, flame_color);
            // Outer ring showing effective boss-hit range
            gizmos.circle_2d(player_pos, flame.area + 50.0, Color::srgba(1.0, 0.5, 0.0, 0.2));
        }
    }

    // Orbit shield hitboxes
    for orbit_transform in orbit_query.iter() {
        let pos = orbit_transform.translation.truncate();
        gizmos.circle_2d(pos, 8.0, orbit_color);
    }

    // Projectile hitboxes
    for proj_transform in projectile_query.iter() {
        let pos = proj_transform.translation.truncate();
        gizmos.circle_2d(pos, 4.0, projectile_color);
    }

    // Boomerang hitboxes
    for boom_transform in boomerang_query.iter() {
        let pos = boom_transform.translation.truncate();
        gizmos.circle_2d(pos, 6.0, boomerang_color);
    }

    // Bone hitboxes
    for bone_transform in bone_query.iter() {
        let pos = bone_transform.translation.truncate();
        gizmos.circle_2d(pos, 6.0, bone_color);
    }

    // Holy water zone hitboxes
    for (zone_transform, zone) in holy_water_query.iter() {
        let pos = zone_transform.translation.truncate();
        gizmos.circle_2d(pos, zone.radius, holy_water_color);
        // Outer ring showing effective boss-hit range
        gizmos.circle_2d(pos, zone.radius + 50.0, Color::srgba(0.2, 0.9, 0.7, 0.2));
    }
}

// === Pickup Label Drawing ===
pub fn draw_pickup_labels(
    mut gizmos: Gizmos,
    pickup_query: Query<(&Transform, &WeaponPickup)>,
    time: Res<Time>,
) {
    let pulse = (time.elapsed_secs() * 3.0).sin() * 0.3 + 0.7;

    for (transform, pickup) in pickup_query.iter() {
        let pos = transform.translation.truncate();
        let color = pickup.0.color();
        // Draw pulsing circle around pickup
        gizmos.circle_2d(pos, 14.0 * pulse, color);
        gizmos.circle_2d(pos, 18.0 * pulse, color.with_alpha(0.3));
    }
}
