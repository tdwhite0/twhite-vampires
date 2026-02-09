use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;
use crate::systems::shader_materials::*;
use crate::systems::startup;

fn play_sound(commands: &mut Commands, handle: &Handle<AudioSource>, volume: f32) {
    commands.spawn((
        AudioPlayer::<AudioSource>::new(handle.clone()),
        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume)),
    ));
}

const ORBIT_SHIELD_SPEED: f32 = 3.0;
const PROJECTILE_SPEED: f32 = 400.0;
const PROJECTILE_LIFETIME: f32 = 1.5;
const BOOMERANG_TOTAL_TIME: f32 = 2.0;
const PHIERA_SPEED: f32 = 350.0;
const PHIERA_LIFETIME: f32 = 2.0;

// === Orbit Shield ===
pub fn orbit_shield_system(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    mut orbit_query: Query<(Entity, &mut Transform, &mut OrbitShield), (Without<Player>, Without<Enemy>)>,
    time: Res<Time>,
    weapons: Res<PlayerWeapons>,
    weapon_shaders: Res<WeaponShaderHandles>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Some(weapon) = weapons.get(WeaponKind::OrbitShield) else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let base_angle = time.elapsed_secs() * ORBIT_SHIELD_SPEED;
    let count = weapon.count.max(1);
    let angle_offset = std::f32::consts::TAU / count as f32;

    let level_idx = (weapon.level.clamp(1, 8) - 1) as usize;

    let mut current_count = 0u32;
    for (orbit_entity, mut transform, mut orbit) in orbit_query.iter_mut() {
        // Despawn extras when count decreased (e.g. debug level downgrade)
        if orbit.orbit_index >= count {
            commands.entity(orbit_entity).despawn();
            continue;
        }
        current_count += 1;

        // Update hit cooldown
        if orbit.hit_cooldown > 0.0 {
            orbit.hit_cooldown -= time.delta_secs();
        }

        let angle = base_angle + orbit.orbit_index as f32 * angle_offset;
        let pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * weapon.area;
        transform.translation = pos.extend(8.0);
        transform.rotation = Quat::from_rotation_z(time.elapsed_secs() * 5.0);
        commands.entity(orbit_entity).insert(MeshMaterial2d(weapon_shaders.orbit_shield[level_idx].clone()));
    }

    // Spawn missing entities
    for idx in current_count..count {
        startup::spawn_orbit_shield_ball(&mut commands, &weapon_shaders, idx);
    }
}

// === Projectile Burst ===
pub fn projectile_burst_system(
    mut commands: Commands,
    mut weapons: ResMut<PlayerWeapons>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    sound_assets: Res<SoundAssets>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Some(weapon) = weapons.get_mut(WeaponKind::ProjectileBurst) else {
        return;
    };

    if !weapon.tick_cooldown(time.delta_secs()) {
        return;
    }

    play_sound(&mut commands, &sound_assets.weapon_projectile, 0.10);

    let player_pos = player_transform.translation.truncate();
    let count = weapon.count.max(1);
    let damage = weapon.damage;
    let level = weapon.level;
    let angle_step = std::f32::consts::TAU / count as f32;

    for i in 0..count {
        let angle = i as f32 * angle_step;
        let direction = Vec2::new(angle.cos(), angle.sin());

        commands.spawn((
            Mesh2d(weapon_shaders.projectile_quad.clone()),
            MeshMaterial2d(weapon_shaders.projectile[(level.clamp(1, 8) - 1) as usize].clone()),
            Transform::from_translation((player_pos + direction * 20.0).extend(9.0)),
            Projectile {
                damage,
                lifetime: PROJECTILE_LIFETIME,
                speed: PROJECTILE_SPEED,
                direction,
                piercing: level >= 5,
                hit_enemies: Vec::new(),
            },
            GameEntity,
        ));
    }
}

pub fn update_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut projectile) in query.iter_mut() {
        projectile.lifetime -= time.delta_secs();
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let movement = projectile.direction * projectile.speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;
    }
}

// === Lightning Zap ===
pub fn lightning_zap_system(
    mut commands: Commands,
    mut weapons: ResMut<PlayerWeapons>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyHealth), (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    mut lightning_mats: ResMut<Assets<LightningMaterial>>,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Some(weapon) = weapons.get_mut(WeaponKind::LightningZap) else {
        return;
    };

    if !weapon.tick_cooldown(time.delta_secs()) {
        return;
    }

    let player_pos = player_transform.translation.truncate();
    let range = weapon.area;
    let damage = weapon.damage * debug.damage_multiplier;
    let chain_count = weapon.count.max(1);
    let vis = WeaponKind::LightningZap.visual_def(weapon.level);

    // Build chain targets iteratively
    let mut chain_targets: Vec<(Entity, Vec2)> = Vec::new();
    let mut current_origin = player_pos;

    for _ in 0..chain_count {
        let mut best_dist = f32::MAX;
        let mut best_target: Option<(Entity, Vec2)> = None;

        for (entity, transform, _) in enemy_query.iter() {
            // Skip enemies already in the chain
            if chain_targets.iter().any(|(e, _)| *e == entity) {
                continue;
            }
            let pos = transform.translation.truncate();
            let dist = current_origin.distance(pos);
            if dist < range && dist < best_dist {
                best_dist = dist;
                best_target = Some((entity, pos));
            }
        }

        if let Some(target) = best_target {
            current_origin = target.1;
            chain_targets.push(target);
        } else {
            break;
        }
    }

    if chain_targets.is_empty() {
        return;
    }

    // Apply damage to all chain targets
    for (entity, _) in &chain_targets {
        if let Ok((_, _, mut health)) = enemy_query.get_mut(*entity) {
            if health.0 > 0.0 {
                health.0 -= damage;
            }
        }
    }

    play_sound(&mut commands, &sound_assets.weapon_lightning, 0.10);

    // Spawn bolt visuals for each chain link
    let mut rng = rand::thread_rng();
    let bolt_width = 40.0;
    let mut bolt_origin = player_pos;

    for (_, target_pos) in &chain_targets {
        let delta = *target_pos - bolt_origin;
        let bolt_length = delta.length();
        let bolt_angle = delta.y.atan2(delta.x);
        let midpoint = (bolt_origin + *target_pos) / 2.0;
        let seed = rng.gen_range(0.0f32..100.0);

        let bolt_mat = lightning_mats.add(LightningMaterial {
            data: LightningData {
                color: Vec4::from_array(vis.color),
                intensity: vis.intensity,
                lifetime: 0.2,
                seed,
                _pad: 0.0,
            },
        });

        commands.spawn((
            Mesh2d(weapon_shaders.lightning_quad.clone()),
            MeshMaterial2d(bolt_mat),
            Transform::from_translation(midpoint.extend(11.0))
                .with_rotation(Quat::from_rotation_z(bolt_angle))
                .with_scale(Vec3::new(bolt_length, bolt_width, 1.0)),
            LightningBoltEntity { lifetime: 0.2 },
            GameEntity,
        ));

        bolt_origin = *target_pos;
    }
}

// === Flame Aura ===
pub fn flame_aura_system(
    mut commands: Commands,
    mut weapons: ResMut<PlayerWeapons>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Transform, &mut EnemyHealth), (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Some(weapon) = weapons.get_mut(WeaponKind::FlameAura) else {
        return;
    };

    if !weapon.tick_cooldown(time.delta_secs()) {
        return;
    }

    play_sound(&mut commands, &sound_assets.weapon_flame, 0.07);

    let player_pos = player_transform.translation.truncate();
    let range = weapon.area;
    let damage = weapon.damage * debug.damage_multiplier;

    for (transform, mut health) in enemy_query.iter_mut() {
        let pos = transform.translation.truncate();
        if player_pos.distance(pos) < range {
            health.0 -= damage;
        }
    }
}

// === Boomerang ===
pub fn boomerang_system(
    mut commands: Commands,
    mut weapons: ResMut<PlayerWeapons>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    sound_assets: Res<SoundAssets>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Some(weapon) = weapons.get_mut(WeaponKind::Boomerang) else {
        return;
    };

    if !weapon.tick_cooldown(time.delta_secs()) {
        return;
    }

    play_sound(&mut commands, &sound_assets.weapon_boomerang, 0.10);

    let player_pos = player_transform.translation.truncate();
    let damage = weapon.damage;
    let max_range = weapon.area;
    let count = weapon.count;
    let level = weapon.level;

    // Find nearest enemy direction, or default forward
    let mut best_dir = Vec2::Y;
    let mut best_dist = f32::MAX;
    for transform in enemy_query.iter() {
        let dist = player_pos.distance(transform.translation.truncate());
        if dist < best_dist {
            best_dist = dist;
            best_dir = (transform.translation.truncate() - player_pos).normalize_or_zero();
        }
    }

    let angle_spread = if count > 1 {
        std::f32::consts::FRAC_PI_4
    } else {
        0.0
    };

    for i in 0..count {
        let offset_angle = if count > 1 {
            let t = i as f32 / (count - 1) as f32 - 0.5;
            t * angle_spread
        } else {
            0.0
        };
        let dir = Vec2::new(
            best_dir.x * offset_angle.cos() - best_dir.y * offset_angle.sin(),
            best_dir.x * offset_angle.sin() + best_dir.y * offset_angle.cos(),
        );

        commands.spawn((
            Mesh2d(weapon_shaders.boomerang_quad.clone()),
            MeshMaterial2d(weapon_shaders.boomerang[(level.clamp(1, 8) - 1) as usize].clone()),
            Transform::from_translation(player_pos.extend(9.0)),
            BoomerangProjectile {
                damage,
                elapsed: 0.0,
                total_time: BOOMERANG_TOTAL_TIME,
                target_dir: dir,
                origin: player_pos,
                max_range,
                hit_enemies: Vec::new(),
            },
            GameEntity,
        ));
    }
}

pub fn update_boomerangs(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BoomerangProjectile)>,
    player_query: Query<&Transform, (With<Player>, Without<BoomerangProjectile>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, mut transform, mut boomerang) in query.iter_mut() {
        boomerang.elapsed += time.delta_secs();
        let progress = boomerang.elapsed / boomerang.total_time;

        if progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let speed = 350.0;
        if progress < 0.5 {
            // Move outward
            let vel = boomerang.target_dir * speed;
            transform.translation.x += vel.x * time.delta_secs();
            transform.translation.y += vel.y * time.delta_secs();
        } else {
            // Return toward player
            let current = transform.translation.truncate();
            let to_player = (player_pos - current).normalize_or_zero();
            let vel = to_player * speed * 1.5;
            transform.translation.x += vel.x * time.delta_secs();
            transform.translation.y += vel.y * time.delta_secs();

            // Despawn if close to player
            if current.distance(player_pos) < 20.0 {
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Spin (the shader also spins internally, but mesh rotation adds to it)
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 12.0);
    }
}

// === Flame Aura Entity Management ===
pub fn manage_flame_aura_entity(
    mut commands: Commands,
    weapons: Res<PlayerWeapons>,
    player_query: Query<&Transform, With<Player>>,
    mut aura_query: Query<(Entity, &mut Transform), (With<FlameAuraEntity>, Without<Player>)>,
    weapon_shaders: Res<WeaponShaderHandles>,
) {
    let has_flame = weapons.has(WeaponKind::FlameAura);
    let aura_area = weapons.get(WeaponKind::FlameAura).map(|w| w.area).unwrap_or(60.0);

    if has_flame {
        let Ok(player_transform) = player_query.single() else {
            return;
        };
        let player_pos = player_transform.translation.truncate();

        let level_idx = (weapons.get(WeaponKind::FlameAura).map(|w| w.level).unwrap_or(1).clamp(1, 8) - 1) as usize;

        if let Ok((entity, mut aura_transform)) = aura_query.single_mut() {
            // Update position and scale to match weapon area
            aura_transform.translation = player_pos.extend(7.0);
            let scale = aura_area * 3.0; // Scale quad to cover the aura radius
            aura_transform.scale = Vec3::new(scale, scale, 1.0);
            commands.entity(entity).insert(MeshMaterial2d(weapon_shaders.flame_aura[level_idx].clone()));
        } else {
            // Spawn new flame aura entity
            let scale = aura_area * 3.0;
            commands.spawn((
                Mesh2d(weapon_shaders.flame_aura_quad.clone()),
                MeshMaterial2d(weapon_shaders.flame_aura[level_idx].clone()),
                Transform::from_translation(player_pos.extend(7.0))
                    .with_scale(Vec3::new(scale, scale, 1.0)),
                FlameAuraEntity,
                GameEntity,
            ));
        }
    } else {
        // Remove aura entity if weapon no longer equipped
        for (entity, _) in aura_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// === Pet Bone Attack ===
const BONE_TOTAL_TIME: f32 = 1.8;

pub fn bone_attack_system(
    mut commands: Commands,
    mut pet_query: Query<(&Transform, &mut PetDog)>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<PetDog>)>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    sound_assets: Res<SoundAssets>,
) {
    let Ok((pet_transform, mut pet)) = pet_query.single_mut() else {
        return;
    };

    pet.bone_timer -= time.delta_secs();
    if pet.bone_timer > 0.0 {
        return;
    }
    pet.bone_timer = pet.bone_cooldown;

    play_sound(&mut commands, &sound_assets.weapon_boomerang, 0.10);

    let pet_pos = pet_transform.translation.truncate();
    let damage = pet.bone_damage;

    // Find nearest enemy direction
    let mut best_dir = Vec2::Y;
    let mut best_dist = f32::MAX;
    for transform in enemy_query.iter() {
        let dist = pet_pos.distance(transform.translation.truncate());
        if dist < best_dist {
            best_dist = dist;
            best_dir = (transform.translation.truncate() - pet_pos).normalize_or_zero();
        }
    }

    commands.spawn((
        Mesh2d(weapon_shaders.bone_quad.clone()),
        MeshMaterial2d(weapon_shaders.bone.clone()),
        Transform::from_translation(pet_pos.extend(9.0)),
        BoneProjectile {
            damage,
            elapsed: 0.0,
            total_time: BONE_TOTAL_TIME,
            target_dir: best_dir,
            owner_pos: pet_pos,
            hit_enemies: Vec::new(),
        },
        GameEntity,
    ));
}

pub fn update_bone_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BoneProjectile)>,
    pet_query: Query<&Transform, (With<PetDog>, Without<BoneProjectile>)>,
    time: Res<Time>,
) {
    let pet_pos = pet_query
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (entity, mut transform, mut bone) in query.iter_mut() {
        bone.elapsed += time.delta_secs();
        let progress = bone.elapsed / bone.total_time;

        if progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let speed = 300.0;
        if progress < 0.5 {
            // Move outward
            let vel = bone.target_dir * speed;
            transform.translation.x += vel.x * time.delta_secs();
            transform.translation.y += vel.y * time.delta_secs();
        } else {
            // Return toward pet
            let current = transform.translation.truncate();
            let to_pet = (pet_pos - current).normalize_or_zero();
            let vel = to_pet * speed * 1.5;
            transform.translation.x += vel.x * time.delta_secs();
            transform.translation.y += vel.y * time.delta_secs();

            if current.distance(pet_pos) < 20.0 {
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Spin
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * 14.0);
    }
}

// === Holy Water ===
const HOLY_WATER_TICK_RATE: f32 = 0.4;
const HOLY_WATER_DURATION: f32 = 3.0;
const HOLY_WATER_SPAWN_RADIUS: f32 = 120.0;

pub fn holy_water_system(
    mut commands: Commands,
    mut weapons: ResMut<PlayerWeapons>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    mut holy_water_mats: ResMut<Assets<HolyWaterMaterial>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Some(weapon) = weapons.get_mut(WeaponKind::HolyWater) else {
        return;
    };

    if !weapon.tick_cooldown(time.delta_secs()) {
        return;
    }

    let player_pos = player_transform.translation.truncate();
    let count = weapon.count.max(1);
    let damage = weapon.damage;
    let area = weapon.area;
    let vis = WeaponKind::HolyWater.visual_def(weapon.level);
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let dist = rng.gen_range(20.0..HOLY_WATER_SPAWN_RADIUS);
        let pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * dist;

        // Create per-zone material so we can update lifetime_frac individually
        let mat = holy_water_mats.add(HolyWaterMaterial {
            data: HolyWaterData {
                color: Vec4::from_array(vis.color),
                intensity: vis.intensity,
                lifetime_frac: 1.0,
                _pad1: 0.0,
                _pad2: 0.0,
            },
        });

        let scale = area * 2.5;
        let stretch = 1.6; // oblong oval: wider than tall
        let zone_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        commands.spawn((
            Mesh2d(weapon_shaders.holy_water_quad.clone()),
            MeshMaterial2d(mat),
            Transform::from_translation(pos.extend(0.5))
                .with_scale(Vec3::new(scale * stretch, scale, 1.0))
                .with_rotation(Quat::from_rotation_z(zone_angle)),
            HolyWaterZone {
                damage,
                tick_timer: 0.0,
                tick_rate: HOLY_WATER_TICK_RATE,
                lifetime: HOLY_WATER_DURATION,
                max_lifetime: HOLY_WATER_DURATION,
                radius: area * stretch, // use the wider dimension for collision
            },
            GameEntity,
        ));
    }
}

pub fn update_holy_water_zones(
    mut commands: Commands,
    mut zone_query: Query<(Entity, &mut HolyWaterZone, &Transform, &MeshMaterial2d<HolyWaterMaterial>)>,
    mut enemy_query: Query<(&Transform, &mut EnemyHealth), (With<Enemy>, Without<HolyWaterZone>)>,
    mut holy_water_mats: ResMut<Assets<HolyWaterMaterial>>,
    time: Res<Time>,
    debug: Res<DebugSettings>,
) {
    let dt = time.delta_secs();
    let dmg_mult = debug.damage_multiplier;

    for (entity, mut zone, zone_transform, mat_handle) in zone_query.iter_mut() {
        zone.lifetime -= dt;
        if zone.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Update shader lifetime_frac for fade-out
        let frac = (zone.lifetime / zone.max_lifetime).clamp(0.0, 1.0);
        if let Some(mat) = holy_water_mats.get_mut(mat_handle.id()) {
            mat.data.lifetime_frac = frac;
        }

        // Tick damage
        zone.tick_timer -= dt;
        if zone.tick_timer > 0.0 {
            continue;
        }
        zone.tick_timer = zone.tick_rate;

        let zone_pos = zone_transform.translation.truncate();
        let damage = zone.damage * dmg_mult;
        let radius = zone.radius;

        for (enemy_transform, mut health) in enemy_query.iter_mut() {
            if health.0 <= 0.0 {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            if zone_pos.distance(enemy_pos) < radius {
                health.0 -= damage;
            }
        }
    }
}

// === UpDown Wave ===
const UPDOWN_SPEED: f32 = 250.0;
const UPDOWN_LIFETIME: f32 = 1.5;

pub fn updown_system(
    mut commands: Commands,
    mut weapons: ResMut<PlayerWeapons>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    mut updown_mats: ResMut<Assets<UpDownMaterial>>,
    sound_assets: Res<SoundAssets>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Some(weapon) = weapons.get_mut(WeaponKind::UpDown) else {
        return;
    };

    if !weapon.tick_cooldown(time.delta_secs()) {
        return;
    }

    play_sound(&mut commands, &sound_assets.weapon_projectile, 0.10);

    let player_pos = player_transform.translation.truncate();
    let count = weapon.count.max(1);
    let damage = weapon.damage;
    let area = weapon.area;
    let vis = WeaponKind::UpDown.visual_def(weapon.level);

    // Spawn count pairs of waves (one up, one down per pair)
    // Pairs are spread horizontally
    let spread = 40.0;
    for i in 0..count {
        let x_offset = if count > 1 {
            let t = i as f32 / (count - 1) as f32 - 0.5;
            t * spread * (count as f32 - 1.0)
        } else {
            0.0
        };

        for dir in [-1.0f32, 1.0f32] {
            let mat = updown_mats.add(UpDownMaterial {
                data: UpDownData {
                    color: Vec4::from_array(vis.color),
                    intensity: vis.intensity,
                    lifetime_frac: 1.0,
                    _pad1: 0.0,
                    _pad2: 0.0,
                },
            });

            let wave_height = area * 2.5;
            let wave_width = 40.0;
            let pos = player_pos + Vec2::new(x_offset, 0.0);

            commands.spawn((
                Mesh2d(weapon_shaders.updown_quad.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(pos.extend(9.0))
                    .with_scale(Vec3::new(wave_width, wave_height, 1.0)),
                UpDownWave {
                    damage,
                    lifetime: UPDOWN_LIFETIME,
                    max_lifetime: UPDOWN_LIFETIME,
                    direction: dir,
                    speed: UPDOWN_SPEED,
                    hit_enemies: Vec::new(),
                },
                GameEntity,
            ));
        }
    }
}

pub fn update_updown_waves(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut UpDownWave, &MeshMaterial2d<UpDownMaterial>)>,
    mut updown_mats: ResMut<Assets<UpDownMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut wave, mat_handle) in query.iter_mut() {
        wave.lifetime -= dt;
        if wave.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Update shader lifetime for fade-out
        let frac = (wave.lifetime / wave.max_lifetime).clamp(0.0, 1.0);
        if let Some(mat) = updown_mats.get_mut(mat_handle.id()) {
            mat.data.lifetime_frac = frac;
        }

        // Move vertically
        transform.translation.y += wave.direction * wave.speed * dt;
    }
}

// === Phiera Der Tuphello ===
pub fn phiera_system(
    mut commands: Commands,
    mut weapons: ResMut<PlayerWeapons>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    sound_assets: Res<SoundAssets>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Some(weapon) = weapons.get_mut(WeaponKind::Phiera) else {
        return;
    };

    if !weapon.tick_cooldown(time.delta_secs()) {
        return;
    }

    play_sound(&mut commands, &sound_assets.weapon_projectile, 0.08);

    let player_pos = player_transform.translation.truncate();
    let count = weapon.count.max(1);
    let damage = weapon.damage;
    let level = weapon.level;
    let level_idx = (level.clamp(1, 8) - 1) as usize;

    // 4 diagonal directions: NE, NW, SE, SW
    let diagonals = [
        Vec2::new(1.0, 1.0).normalize(),
        Vec2::new(-1.0, 1.0).normalize(),
        Vec2::new(1.0, -1.0).normalize(),
        Vec2::new(-1.0, -1.0).normalize(),
    ];

    let spread_angle = if count > 1 { 0.15 } else { 0.0 }; // slight angular spread for extra bullets

    for base_dir in &diagonals {
        for i in 0..count {
            let offset = if count > 1 {
                let t = i as f32 / (count - 1) as f32 - 0.5;
                t * spread_angle
            } else {
                0.0
            };

            let direction = Vec2::new(
                base_dir.x * offset.cos() - base_dir.y * offset.sin(),
                base_dir.x * offset.sin() + base_dir.y * offset.cos(),
            );

            commands.spawn((
                Mesh2d(weapon_shaders.phiera_quad.clone()),
                MeshMaterial2d(weapon_shaders.phiera[level_idx].clone()),
                Transform::from_translation((player_pos + direction * 15.0).extend(9.0)),
                Projectile {
                    damage,
                    lifetime: PHIERA_LIFETIME,
                    speed: PHIERA_SPEED,
                    direction,
                    piercing: level >= 5,
                    hit_enemies: Vec::new(),
                },
                GameEntity,
            ));
        }
    }
}

// === Lightning Bolt Entity Lifetime ===
pub fn update_lightning_bolt_entities(
    mut commands: Commands,
    mut query: Query<(Entity, &mut LightningBoltEntity, &MeshMaterial2d<LightningMaterial>)>,
    mut lightning_mats: ResMut<Assets<LightningMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut bolt, mat_handle) in query.iter_mut() {
        bolt.lifetime -= time.delta_secs();
        if bolt.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Update the material's lifetime uniform for fade effect
        if let Some(mat) = lightning_mats.get_mut(mat_handle.id()) {
            mat.data.lifetime = bolt.lifetime;
        }
    }
}
