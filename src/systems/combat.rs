use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;
use crate::systems::startup;
use crate::systems::collision::{enemy_radius_for, weapon_radii};

const PLAYER_RADIUS: f32 = 15.0;
const INVINCIBILITY_TIME: f32 = 0.5;
const XP_GEM_RADIUS: f32 = 4.0;
const XP_MAGNET_RADIUS: f32 = 100.0;
const XP_MAGNET_SPEED: f32 = 300.0;
const PICKUP_RADIUS: f32 = 10.0;
const HEALING_DOT_RADIUS: f32 = 5.0;
const HEALING_DOT_MAGNET_RADIUS: f32 = 80.0;
const HEALING_DOT_MAGNET_SPEED: f32 = 250.0;
const WEAPON_PICKUP_MAGNET_RADIUS: f32 = 90.0;
const WEAPON_PICKUP_MAGNET_SPEED: f32 = 275.0;
const MAGNET_SNAP_TIME: f32 = 1.5; // seconds chasing before instant snap
const HEALING_DOT_DROP_CHANCE: f64 = 0.08; // 8% chance

pub fn player_enemy_collision(
    mut commands: Commands,
    mut player_query: Query<
        (&Transform, &mut Health, &mut DamageCooldown),
        (With<Player>, Without<Dashing>),
    >,
    enemy_query: Query<(&Transform, &ContactDamage, &EnemyType), With<Enemy>>,
    _time: Res<Time>,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
    mut stats: ResMut<GameStats>,
) {
    let Ok((player_transform, mut health, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    if debug.invincible || cooldown.0 > 0.0 {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (enemy_transform, damage, enemy_type) in enemy_query.iter() {
        let enemy_pos = enemy_transform.translation.truncate();
        let enemy_radius = match enemy_type.0 {
            EnemyKind::Basic => 12.0,
            EnemyKind::Fast => 8.0,
            EnemyKind::Tank => 14.0,
            EnemyKind::Swarm => 5.0,
            EnemyKind::Flock => 6.0,
        };

        let dist = player_pos.distance(enemy_pos);
        if dist < PLAYER_RADIUS + enemy_radius {
            health.current -= damage.0;
            stats.damage_taken += damage.0;
            cooldown.0 = INVINCIBILITY_TIME;
            commands.spawn((
                AudioPlayer::<AudioSource>::new(sound_assets.player_damage.clone()),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.35)),
            ));
            break; // Only take damage once per frame
        }
    }
}

pub fn dash_enemy_collision(
    mut player_query: Query<(&Transform, &mut Dashing), With<Player>>,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyHealth, &EnemyType), With<Enemy>>,
    debug: Res<DebugSettings>,
) {
    let Ok((player_transform, mut dashing)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (enemy_entity, enemy_transform, mut health, enemy_type) in enemy_query.iter_mut() {
        if health.0 <= 0.0 {
            continue;
        }
        if dashing.hit_enemies.contains(&enemy_entity) {
            continue;
        }

        let enemy_pos = enemy_transform.translation.truncate();
        let enemy_radius = enemy_radius_for(enemy_type.0);
        let dist = player_pos.distance(enemy_pos);

        if dist < PLAYER_RADIUS + enemy_radius {
            health.0 -= dashing.damage * debug.damage_multiplier;
            dashing.hit_enemies.push(enemy_entity);
        }
    }
}

pub fn weapon_enemy_collision(
    mut commands: Commands,
    // Orbit shields
    mut orbit_query: Query<
        (&Transform, &mut OrbitShield),
        (Without<Enemy>, Without<Player>),
    >,
    // Projectiles
    mut projectile_query: Query<
        (Entity, &Transform, &mut Projectile),
        (Without<Enemy>, Without<Player>, Without<OrbitShield>),
    >,
    // Boomerangs
    mut boomerang_query: Query<
        (&Transform, &mut BoomerangProjectile),
        (Without<Enemy>, Without<Player>, Without<OrbitShield>, Without<Projectile>, Without<BoneProjectile>),
    >,
    // Bones (pet weapon)
    mut bone_query: Query<
        (&Transform, &mut BoneProjectile),
        (Without<Enemy>, Without<Player>, Without<OrbitShield>, Without<Projectile>, Without<BoomerangProjectile>),
    >,
    // UpDown waves
    mut updown_query: Query<
        (&Transform, &mut UpDownWave),
        (Without<Enemy>, Without<Player>, Without<OrbitShield>, Without<Projectile>, Without<BoomerangProjectile>, Without<BoneProjectile>),
    >,
    // Whip slashes
    mut whip_query: Query<
        (&Transform, &mut WhipSlash),
        (Without<Enemy>, Without<Player>, Without<OrbitShield>, Without<Projectile>, Without<BoomerangProjectile>, Without<BoneProjectile>, Without<UpDownWave>),
    >,
    mut enemy_query: Query<
        (Entity, &Transform, &mut EnemyHealth, &EnemyType),
        With<Enemy>,
    >,
    weapons: Res<PlayerWeapons>,
    debug: Res<DebugSettings>,
    mut stats: ResMut<GameStats>,
) {
    let dmg_mult = debug.damage_multiplier;
    let orbit_damage = weapons
        .get(WeaponKind::OrbitShield)
        .map(|w| w.damage * dmg_mult)
        .unwrap_or(0.0);

    // Orbit shield collisions
    for (orbit_transform, mut orbit) in orbit_query.iter_mut() {
        if orbit.hit_cooldown > 0.0 {
            continue;
        }
        let orbit_pos = orbit_transform.translation.truncate();
        let orbit_radius = weapon_radii::ORBIT_SHIELD;

        for (_entity, enemy_transform, mut health, enemy_type) in enemy_query.iter_mut() {
            if health.0 <= 0.0 {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let enemy_radius = enemy_radius_for(enemy_type.0);
            let dist = orbit_pos.distance(enemy_pos);

            if dist < orbit_radius + enemy_radius {
                health.0 -= orbit_damage;
                stats.record_weapon_damage(WeaponKind::OrbitShield, orbit_damage);
                orbit.hit_cooldown = 0.3;
                break;
            }
        }
    }

    // Projectile collisions
    for (proj_entity, proj_transform, mut projectile) in projectile_query.iter_mut() {
        let proj_pos = proj_transform.translation.truncate();
        let proj_radius = weapon_radii::PROJECTILE;

        for (enemy_entity, enemy_transform, mut health, enemy_type) in enemy_query.iter_mut() {
            if health.0 <= 0.0 {
                continue;
            }
            if projectile.hit_enemies.contains(&enemy_entity) {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let enemy_radius = enemy_radius_for(enemy_type.0);
            let dist = proj_pos.distance(enemy_pos);

            if dist < proj_radius + enemy_radius {
                let dmg = projectile.damage * dmg_mult;
                health.0 -= dmg;
                stats.record_weapon_damage(projectile.source, dmg);
                if projectile.piercing {
                    projectile.hit_enemies.push(enemy_entity);
                } else {
                    commands.entity(proj_entity).despawn();
                    break;
                }
            }
        }
    }

    // Boomerang collisions
    for (boom_transform, mut boomerang) in boomerang_query.iter_mut() {
        let boom_pos = boom_transform.translation.truncate();
        let boom_radius = weapon_radii::BOOMERANG;

        for (enemy_entity, enemy_transform, mut health, enemy_type) in enemy_query.iter_mut() {
            if health.0 <= 0.0 {
                continue;
            }
            if boomerang.hit_enemies.contains(&enemy_entity) {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let enemy_radius = enemy_radius_for(enemy_type.0);
            let dist = boom_pos.distance(enemy_pos);

            if dist < boom_radius + enemy_radius {
                let dmg = boomerang.damage * dmg_mult;
                health.0 -= dmg;
                stats.record_weapon_damage(WeaponKind::Boomerang, dmg);
                boomerang.hit_enemies.push(enemy_entity);
            }
        }
    }

    // Bone collisions (pet weapon - same pattern as boomerang)
    for (bone_transform, mut bone) in bone_query.iter_mut() {
        let bone_pos = bone_transform.translation.truncate();
        let bone_radius = weapon_radii::BONE;

        for (enemy_entity, enemy_transform, mut health, enemy_type) in enemy_query.iter_mut() {
            if health.0 <= 0.0 {
                continue;
            }
            if bone.hit_enemies.contains(&enemy_entity) {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let enemy_radius = enemy_radius_for(enemy_type.0);
            let dist = bone_pos.distance(enemy_pos);

            if dist < bone_radius + enemy_radius {
                let dmg = bone.damage * dmg_mult;
                health.0 -= dmg;
                // Bone is a pet weapon, not tracked per-weapon
                bone.hit_enemies.push(enemy_entity);
            }
        }
    }

    // UpDown wave collisions (piercing, tracks hit enemies)
    for (wave_transform, mut wave) in updown_query.iter_mut() {
        let wave_pos = wave_transform.translation.truncate();
        let wave_radius = weapon_radii::UPDOWN;

        for (enemy_entity, enemy_transform, mut health, enemy_type) in enemy_query.iter_mut() {
            if health.0 <= 0.0 {
                continue;
            }
            if wave.hit_enemies.contains(&enemy_entity) {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();
            let enemy_radius = enemy_radius_for(enemy_type.0);
            let dist = wave_pos.distance(enemy_pos);

            if dist < wave_radius + enemy_radius {
                let dmg = wave.damage * dmg_mult;
                health.0 -= dmg;
                stats.record_weapon_damage(WeaponKind::UpDown, dmg);
                wave.hit_enemies.push(enemy_entity);
            }
        }
    }

    // Whip slash collisions (oriented rectangle, piercing, tracks hit enemies)
    for (slash_transform, mut slash) in whip_query.iter_mut() {
        if slash.delay > 0.0 {
            continue;
        }
        let slash_pos = slash_transform.translation.truncate();

        for (enemy_entity, enemy_transform, mut health, _enemy_type) in enemy_query.iter_mut() {
            if health.0 <= 0.0 {
                continue;
            }
            if slash.hit_enemies.contains(&enemy_entity) {
                continue;
            }
            let enemy_pos = enemy_transform.translation.truncate();

            // Oriented rectangle check: transform enemy position into slash's local space
            let to_enemy = enemy_pos - slash_pos;
            let local_x = to_enemy.dot(slash.direction);
            let perp = Vec2::new(-slash.direction.y, slash.direction.x);
            let local_y = to_enemy.dot(perp);

            if local_x.abs() < slash.half_length && local_y.abs() < slash.half_width {
                let dmg = slash.damage * dmg_mult;
                health.0 -= dmg;
                stats.record_weapon_damage(WeaponKind::Whip, dmg);
                slash.hit_enemies.push(enemy_entity);
            }
        }
    }
}

pub fn enemy_death(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Transform, &EnemyHealth, &EnemyType), With<Enemy>>,
    mut wave: ResMut<WaveManager>,
    mut stats: ResMut<GameStats>,
    game_meshes: Res<GameMeshes>,
    game_materials: Res<GameMaterials>,
    sound_assets: Res<SoundAssets>,
) {
    for (entity, transform, health, enemy_type) in enemy_query.iter() {
        if health.0 > 0.0 {
            continue;
        }

        let pos = transform.translation.truncate();
        commands.entity(entity).despawn();
        wave.enemies_alive = wave.enemies_alive.saturating_sub(1);
        stats.enemies_killed += 1;

        // Play random pentatonic death tone
        let idx = rand::thread_rng().gen_range(0..5);
        commands.spawn((
            AudioPlayer::<AudioSource>::new(sound_assets.enemy_death[idx].clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.15)),
        ));

        // Spawn XP gem
        let xp_value = match enemy_type.0 {
            EnemyKind::Basic => 1.0,
            EnemyKind::Fast => 1.0,
            EnemyKind::Tank => 3.0,
            EnemyKind::Swarm => 0.5,
            EnemyKind::Flock => 0.8,
        };

        commands.spawn((
            Mesh2d(game_meshes.xp_gem.clone()),
            MeshMaterial2d(game_materials.xp_gem.clone()),
            Transform::from_translation(pos.extend(1.0))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            XpGem(xp_value),
            GameEntity,
        ));

        // Chance to spawn healing dot
        let mut rng = rand::thread_rng();
        if rng.gen_bool(HEALING_DOT_DROP_CHANCE) {
            let heal_amount = match enemy_type.0 {
                EnemyKind::Basic | EnemyKind::Fast => 5.0,
                EnemyKind::Tank => 10.0,
                EnemyKind::Swarm => 3.0,
                EnemyKind::Flock => 4.0,
            };
            commands.spawn((
                Mesh2d(game_meshes.healing_dot.clone()),
                MeshMaterial2d(game_materials.healing_dot.clone()),
                Transform::from_translation(pos.extend(1.5)),
                HealingDot(heal_amount),
                GameEntity,
            ));
        }

        // Spawn death particles
        for _ in 0..4 {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(80.0..200.0);
            let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
            commands.spawn((
                Mesh2d(game_meshes.projectile.clone()),
                MeshMaterial2d(game_materials.enemy_basic.clone()),
                Transform::from_translation(pos.extend(15.0))
                    .with_scale(Vec3::splat(0.5)),
                Particle {
                    velocity: vel,
                    lifetime: 0.4,
                },
                GameEntity,
            ));
        }
    }
}

pub fn xp_gem_collection(
    mut commands: Commands,
    mut gem_query: Query<(Entity, &mut Transform, &XpGem, Option<&mut MagnetLocked>), Without<Player>>,
    mut player_query: Query<(&Transform, &mut Experience), With<Player>>,
    time: Res<Time>,
    sound_assets: Res<SoundAssets>,
    pickup_range: Res<PickupRange>,
    mut stats: ResMut<GameStats>,
) {
    let Ok((player_transform, mut xp)) = player_query.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let magnet_radius = XP_MAGNET_RADIUS * pickup_range.multiplier;
    let dt = time.delta_secs();

    for (entity, mut gem_transform, gem, mut locked) in gem_query.iter_mut() {
        let gem_pos = gem_transform.translation.truncate();
        let dist = player_pos.distance(gem_pos);

        // Magnetic attraction — once in range, stay locked on and accelerate
        if dist < magnet_radius && locked.is_none() {
            commands.entity(entity).insert(MagnetLocked(0.0));
        }
        if let Some(ref mut lock) = locked {
            lock.0 += dt;
            if lock.0 >= MAGNET_SNAP_TIME {
                // Snap directly to player
                gem_transform.translation = player_transform.translation;
            } else {
                let direction = (player_pos - gem_pos).normalize_or_zero();
                let accel = 1.0 + (lock.0 / MAGNET_SNAP_TIME) * 4.0;
                let speed = XP_MAGNET_SPEED * accel;
                gem_transform.translation.x += direction.x * speed * dt;
                gem_transform.translation.y += direction.y * speed * dt;
            }
        } else if dist < magnet_radius {
            let direction = (player_pos - gem_pos).normalize_or_zero();
            gem_transform.translation.x += direction.x * XP_MAGNET_SPEED * dt;
            gem_transform.translation.y += direction.y * XP_MAGNET_SPEED * dt;
        }

        // Collection
        if dist < PLAYER_RADIUS + XP_GEM_RADIUS {
            xp.current += gem.0;
            stats.xp_collected += gem.0;
            commands.entity(entity).despawn();
            let idx = rand::thread_rng().gen_range(0..3);
            commands.spawn((
                AudioPlayer::<AudioSource>::new(sound_assets.xp_gem[idx].clone()),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.12)),
            ));
        }
    }
}

pub fn healing_dot_collection(
    mut commands: Commands,
    mut dot_query: Query<(Entity, &mut Transform, &HealingDot, Option<&mut MagnetLocked>), Without<Player>>,
    mut player_query: Query<(&Transform, &mut Health), With<Player>>,
    time: Res<Time>,
    sound_assets: Res<SoundAssets>,
    pickup_range: Res<PickupRange>,
    mut stats: ResMut<GameStats>,
) {
    let Ok((player_transform, mut health)) = player_query.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let magnet_radius = HEALING_DOT_MAGNET_RADIUS * pickup_range.multiplier;
    let dt = time.delta_secs();

    for (entity, mut dot_transform, dot, mut locked) in dot_query.iter_mut() {
        let dot_pos = dot_transform.translation.truncate();
        let dist = player_pos.distance(dot_pos);

        // Magnetic attraction — once in range, stay locked on and accelerate
        if dist < magnet_radius && locked.is_none() {
            commands.entity(entity).insert(MagnetLocked(0.0));
        }
        if let Some(ref mut lock) = locked {
            lock.0 += dt;
            if lock.0 >= MAGNET_SNAP_TIME {
                dot_transform.translation = player_transform.translation;
            } else {
                let direction = (player_pos - dot_pos).normalize_or_zero();
                let accel = 1.0 + (lock.0 / MAGNET_SNAP_TIME) * 4.0;
                let speed = HEALING_DOT_MAGNET_SPEED * accel;
                dot_transform.translation.x += direction.x * speed * dt;
                dot_transform.translation.y += direction.y * speed * dt;
            }
        } else if dist < magnet_radius {
            let direction = (player_pos - dot_pos).normalize_or_zero();
            dot_transform.translation.x += direction.x * HEALING_DOT_MAGNET_SPEED * dt;
            dot_transform.translation.y += direction.y * HEALING_DOT_MAGNET_SPEED * dt;
        }

        // Collection
        if dist < PLAYER_RADIUS + HEALING_DOT_RADIUS {
            health.current = (health.current + dot.0).min(health.max);
            stats.heals_collected += 1;
            commands.entity(entity).despawn();
            commands.spawn((
                AudioPlayer::<AudioSource>::new(sound_assets.xp_gem[0].clone()),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.15)),
            ));
        }
    }
}

pub fn check_level_up(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&mut Experience, With<Player>>,
    sound_assets: Res<SoundAssets>,
) {
    let Ok(mut xp) = player_query.single_mut() else {
        return;
    };

    if xp.current >= xp.next_level {
        xp.current -= xp.next_level;
        xp.level += 1;
        xp.next_level = xp.level as f32 * 12.0;
        next_state.set(GameState::LevelUp);
        commands.spawn((
            AudioPlayer::<AudioSource>::new(sound_assets.level_up.clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.30)),
        ));
    }
}

pub fn check_game_over(
    mut commands: Commands,
    player_query: Query<&Health, With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
    sound_assets: Res<SoundAssets>,
) {
    let Ok(health) = player_query.single() else {
        return;
    };

    if health.current <= 0.0 {
        next_state.set(GameState::GameOver);
        commands.spawn((
            AudioPlayer::<AudioSource>::new(sound_assets.game_over.clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.30)),
        ));
    }
}

pub fn weapon_pickup_collection(
    mut commands: Commands,
    mut pickup_query: Query<(Entity, &mut Transform, &WeaponPickup, Option<&mut MagnetLocked>), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut weapons: ResMut<PlayerWeapons>,
    weapon_shaders: Res<WeaponShaderHandles>,
    sound_assets: Res<SoundAssets>,
    mut notif_events: MessageWriter<NotificationEvent>,
    orbit_query: Query<Entity, With<OrbitShield>>,
    pickup_range: Res<PickupRange>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let magnet_radius = WEAPON_PICKUP_MAGNET_RADIUS * pickup_range.multiplier;
    let dt = time.delta_secs();

    for (entity, mut pickup_transform, pickup, mut locked) in pickup_query.iter_mut() {
        let pickup_pos = pickup_transform.translation.truncate();
        let dist = player_pos.distance(pickup_pos);

        // Magnetic attraction — once in range, stay locked on and accelerate
        if dist < magnet_radius && locked.is_none() {
            commands.entity(entity).insert(MagnetLocked(0.0));
        }
        if let Some(ref mut lock) = locked {
            lock.0 += dt;
            if lock.0 >= MAGNET_SNAP_TIME {
                pickup_transform.translation = player_transform.translation;
            } else {
                let direction = (player_pos - pickup_pos).normalize_or_zero();
                let accel = 1.0 + (lock.0 / MAGNET_SNAP_TIME) * 4.0;
                let speed = WEAPON_PICKUP_MAGNET_SPEED * accel;
                pickup_transform.translation.x += direction.x * speed * dt;
                pickup_transform.translation.y += direction.y * speed * dt;
            }
        } else if dist < magnet_radius {
            let direction = (player_pos - pickup_pos).normalize_or_zero();
            pickup_transform.translation.x += direction.x * WEAPON_PICKUP_MAGNET_SPEED * dt;
            pickup_transform.translation.y += direction.y * WEAPON_PICKUP_MAGNET_SPEED * dt;
        }

        if dist < PLAYER_RADIUS + PICKUP_RADIUS {
            let kind = pickup.0;
            commands.entity(entity).despawn();
            commands.spawn((
                AudioPlayer::<AudioSource>::new(sound_assets.weapon_pickup.clone()),
                PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.25)),
            ));

            if weapons.has(kind) {
                // Upgrade existing weapon
                if let Some(weapon) = weapons.get_mut(kind) {
                    if weapon.level < ActiveWeapon::MAX_LEVEL {
                        weapon.set_level(weapon.level + 1);
                    }
                }
                // Sync orbit shield entities if needed
                if kind == WeaponKind::OrbitShield {
                    if let Some(weapon) = weapons.get(kind) {
                        startup::sync_orbit_shield_entities(&mut commands, &weapon_shaders, &orbit_query, weapon.count);
                    }
                }
                notif_events.write(NotificationEvent {
                    message: format!("Weapon Upgraded: {}", kind.display_name()),
                    kind: NotificationType::Weapon,
                });
            } else {
                // Add new weapon
                weapons.weapons.push(ActiveWeapon::new(kind));

                // Spawn orbiter entities for orbit shield
                if kind == WeaponKind::OrbitShield {
                    startup::spawn_orbit_shield_ball(&mut commands, &weapon_shaders, 0);
                }
                notif_events.write(NotificationEvent {
                    message: format!("Gained Weapon: {}", kind.display_name()),
                    kind: NotificationType::Weapon,
                });
            }
        }
    }
}

pub fn spawn_weapon_pickups(
    mut commands: Commands,
    mut timer: ResMut<PickupSpawnTimer>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    existing_pickups: Query<&WeaponPickup>,
    game_meshes: Res<GameMeshes>,
    game_materials: Res<GameMaterials>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    timer.0 -= time.delta_secs();
    if timer.0 > 0.0 {
        return;
    }
    timer.0 = 15.0;

    // Max 3 pickups on ground
    if existing_pickups.iter().count() >= 3 {
        return;
    }

    let player_pos = player_transform.translation.truncate();
    let mut rng = rand::thread_rng();

    // Pick a random weapon type
    let kinds = WeaponKind::all();
    let kind = kinds[rng.gen_range(0..kinds.len())];

    // Spawn at random position near player
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let dist = rng.gen_range(150.0..300.0);
    let pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * dist;

    let material = match kind {
        WeaponKind::OrbitShield => game_materials.pickup_orbit.clone(),
        WeaponKind::ProjectileBurst => game_materials.pickup_burst.clone(),
        WeaponKind::LightningZap => game_materials.pickup_lightning.clone(),
        WeaponKind::FlameAura => game_materials.pickup_flame.clone(),
        WeaponKind::Boomerang => game_materials.pickup_boomerang.clone(),
        WeaponKind::HolyWater => game_materials.pickup_holy_water.clone(),
        WeaponKind::UpDown => game_materials.pickup_updown.clone(),
        WeaponKind::Phiera => game_materials.pickup_phiera.clone(),
        WeaponKind::Whip => game_materials.pickup_whip.clone(),
    };

    commands.spawn((
        Mesh2d(game_meshes.pickup.clone()),
        MeshMaterial2d(material),
        Transform::from_translation(pos.extend(2.0)),
        WeaponPickup(kind),
        GameEntity,
    ));
}

pub fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in query.iter_mut() {
        particle.lifetime -= time.delta_secs();
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();

        // Fade out by shrinking
        let alpha = particle.lifetime / 0.4;
        transform.scale = Vec3::splat(alpha.max(0.1));
    }
}
