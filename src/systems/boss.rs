use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;
use crate::systems::shader_materials::*;
use crate::systems::collision::weapon_radii;

fn spawn_boss_damage_number(commands: &mut Commands, pos: Vec2, damage: f32) {
    let mut rng = rand::thread_rng();
    let offset_x = rng.gen_range(-15.0f32..15.0);
    commands.spawn((
        Text2d::new(format!("{:.0}", damage)),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.2, 0.2)),
        Transform::from_translation((pos + Vec2::new(offset_x, 20.0)).extend(50.0)),
        FloatingText {
            velocity: Vec2::new(0.0, 60.0),
            lifetime: 0.7,
            max_lifetime: 0.7,
            base_color: Vec3::new(1.0, 0.2, 0.2),
        },
        GameEntity,
    ));
}

fn play_damage_sound(commands: &mut Commands, sound_assets: &SoundAssets) {
    commands.spawn((
        AudioPlayer::<AudioSource>::new(sound_assets.player_damage.clone()),
        PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.35)),
    ));
}

// === Per-Boss Constants ===

// Eyeball
const EYEBALL_MAX_HEALTH: f32 = 500.0;
const EYEBALL_SPEED: f32 = 40.0;
const EYEBALL_CONTACT_DAMAGE: f32 = 30.0;
const EYEBALL_RADIUS: f32 = 50.0;
const EYEBALL_LASER_RANGE: f32 = 300.0;
const EYEBALL_LASER_DAMAGE: f32 = 20.0;
const EYEBALL_LASER_HIT_COOLDOWN: f32 = 0.3;
const BOSS_IRIS_OFFSET: Vec2 = Vec2::new(42.0, 18.0);
const CHARGE_DURATION: f32 = 1.5;
const FIRE_DURATION: f32 = 2.0;
const COOLDOWN_DURATION: f32 = 2.5;
const SWEEP_ANGLE: f32 = std::f32::consts::FRAC_PI_3;

// Dragon Lord
const DRAGON_MAX_HEALTH: f32 = 600.0;
const DRAGON_SPEED: f32 = 50.0;
const DRAGON_CONTACT_DAMAGE: f32 = 25.0;
const DRAGON_RADIUS: f32 = 45.0;
const DRAGON_SPRITE_INDEX: usize = 60;
const DRAGON_SCALE: f32 = 5.0;
const DRAGON_FIREBALL_SPEED: f32 = 200.0;
const DRAGON_FIREBALL_DAMAGE: f32 = 15.0;
const DRAGON_FIREBALL_LIFETIME: f32 = 3.0;
const DRAGON_FIRE_ZONE_DAMAGE: f32 = 8.0;
const DRAGON_FIRE_ZONE_LIFETIME: f32 = 3.0;
const DRAGON_FIRE_ZONE_RADIUS: f32 = 40.0;

// Necromancer
const NECRO_MAX_HEALTH: f32 = 450.0;
const NECRO_SPEED: f32 = 35.0;
const NECRO_CONTACT_DAMAGE: f32 = 15.0;
const NECRO_RADIUS: f32 = 40.0;
const NECRO_SPRITE_INDEX: usize = 34;
const NECRO_SCALE: f32 = 4.5;
const NECRO_ORB_SPEED: f32 = 120.0;
const NECRO_ORB_TURN_RATE: f32 = 1.5;
const NECRO_ORB_DAMAGE: f32 = 20.0;
const NECRO_ORB_LIFETIME: f32 = 4.0;
const NECRO_SUMMON_COUNT_MIN: u32 = 4;
const NECRO_SUMMON_COUNT_MAX: u32 = 6;

// Slime King
const SLIME_MAX_HEALTH: f32 = 700.0;
const SLIME_SPEED: f32 = 30.0;
const SLIME_CONTACT_DAMAGE: f32 = 35.0;
const SLIME_RADIUS: f32 = 55.0;
const SLIME_SPRITE_INDEX: usize = 0;
const SLIME_SCALE: f32 = 5.0;
const SLIME_SHOCKWAVE_DAMAGE: f32 = 25.0;
const SLIME_SHOCKWAVE_MAX_RADIUS: f32 = 150.0;
const SLIME_SPLIT_HEALTH: f32 = 100.0;
const SLIME_SPLIT_SPEED: f32 = 60.0;
const SLIME_SPLIT_DAMAGE: f32 = 15.0;
const SLIME_SPLIT_SCALE: f32 = 2.5;
const SLIME_SPLIT_RADIUS: f32 = 20.0;

// General
const BOSS_SPAWN_DISTANCE: f32 = 800.0;
const BOSS_XP_DROP: f32 = 50.0;

pub fn boss_radius(kind: BossKind) -> f32 {
    match kind {
        BossKind::Eyeball => EYEBALL_RADIUS,
        BossKind::DragonLord => DRAGON_RADIUS,
        BossKind::Necromancer => NECRO_RADIUS,
        BossKind::SlimeKing => SLIME_RADIUS,
    }
}

fn boss_kind_from_counter(counter: u32) -> BossKind {
    match counter % 4 {
        0 => BossKind::Eyeball,
        1 => BossKind::DragonLord,
        2 => BossKind::Necromancer,
        3 => BossKind::SlimeKing,
        _ => unreachable!(),
    }
}

// === Boss Spawning ===

pub fn boss_spawning(
    mut commands: Commands,
    mut boss_timer: ResMut<BossSpawnTimer>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    boss_assets: Res<BossAssets>,
    sprite_assets: Res<SpriteAssets>,
    existing_bosses: Query<&Boss>,
    mut notif_events: MessageWriter<NotificationEvent>,
    force_spawn: Option<Res<ForceSpawnBoss>>,
) {
    let forced_kind = force_spawn.as_ref().map(|f| f.kind);
    let forced = force_spawn.is_some();
    if forced {
        commands.remove_resource::<ForceSpawnBoss>();
    }

    if !forced {
        boss_timer.timer -= time.delta_secs();
        if boss_timer.timer > 0.0 {
            return;
        }
        boss_timer.timer = boss_timer.interval;

        if existing_bosses.iter().count() >= 2 {
            return;
        }
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let spawn_pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * BOSS_SPAWN_DISTANCE;

    let kind = forced_kind.flatten().unwrap_or_else(|| {
        boss_kind_from_counter(boss_timer.spawn_counter)
    });
    boss_timer.spawn_counter += 1;

    match kind {
        BossKind::Eyeball => spawn_eyeball(&mut commands, spawn_pos, &boss_assets),
        BossKind::DragonLord => spawn_dragon(&mut commands, spawn_pos, &sprite_assets),
        BossKind::Necromancer => spawn_necromancer(&mut commands, spawn_pos, &sprite_assets),
        BossKind::SlimeKing => spawn_slime_king(&mut commands, spawn_pos, &sprite_assets),
    }

    notif_events.write(NotificationEvent {
        message: format!("{} Spawned!", kind.display_name()),
        kind: NotificationType::Boss,
    });
}

fn spawn_eyeball(commands: &mut Commands, pos: Vec2, boss_assets: &BossAssets) {
    commands.spawn((
        Sprite::from_atlas_image(
            boss_assets.eye_texture_1.clone(),
            TextureAtlas {
                layout: boss_assets.eye_layout.clone(),
                index: 0,
            },
        ),
        Transform::from_translation(pos.extend(5.0))
            .with_scale(Vec3::splat(1.5)),
        Boss { kind: BossKind::Eyeball },
        BossHealth { current: EYEBALL_MAX_HEALTH, max: EYEBALL_MAX_HEALTH },
        ContactDamage(EYEBALL_CONTACT_DAMAGE),
        BossLaser {
            phase: BossLaserPhase::Idle,
            range: EYEBALL_LASER_RANGE,
            damage_per_tick: EYEBALL_LASER_DAMAGE,
            hit_cooldown: 0.0,
        },
        GameEntity,
    ));
}

fn spawn_dragon(commands: &mut Commands, pos: Vec2, sprite_assets: &SpriteAssets) {
    commands.spawn((
        Sprite::from_atlas_image(
            sprite_assets.creatures_texture.clone(),
            TextureAtlas {
                layout: sprite_assets.creatures_layout.clone(),
                index: DRAGON_SPRITE_INDEX,
            },
        ),
        Transform::from_translation(pos.extend(5.0))
            .with_scale(Vec3::splat(DRAGON_SCALE)),
        Boss { kind: BossKind::DragonLord },
        BossHealth { current: DRAGON_MAX_HEALTH, max: DRAGON_MAX_HEALTH },
        ContactDamage(DRAGON_CONTACT_DAMAGE),
        DragonBreath { phase: DragonBreathPhase::Idle },
        GameEntity,
    ));
}

fn spawn_necromancer(commands: &mut Commands, pos: Vec2, sprite_assets: &SpriteAssets) {
    commands.spawn((
        Sprite::from_atlas_image(
            sprite_assets.creatures_texture.clone(),
            TextureAtlas {
                layout: sprite_assets.creatures_layout.clone(),
                index: NECRO_SPRITE_INDEX,
            },
        ),
        Transform::from_translation(pos.extend(5.0))
            .with_scale(Vec3::splat(NECRO_SCALE)),
        Boss { kind: BossKind::Necromancer },
        BossHealth { current: NECRO_MAX_HEALTH, max: NECRO_MAX_HEALTH },
        ContactDamage(NECRO_CONTACT_DAMAGE),
        NecromancerMagic {
            phase: NecroPhase::Idle { timer: 2.0 },
            attack_index: 0,
        },
        GameEntity,
    ));
}

fn spawn_slime_king(commands: &mut Commands, pos: Vec2, sprite_assets: &SpriteAssets) {
    commands.spawn((
        Sprite {
            image: sprite_assets.creatures_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sprite_assets.creatures_layout.clone(),
                index: SLIME_SPRITE_INDEX,
            }),
            color: Color::srgb(0.5, 1.0, 0.5),
            ..default()
        },
        Transform::from_translation(pos.extend(5.0))
            .with_scale(Vec3::splat(SLIME_SCALE)),
        Boss { kind: BossKind::SlimeKing },
        BossHealth { current: SLIME_MAX_HEALTH, max: SLIME_MAX_HEALTH },
        ContactDamage(SLIME_CONTACT_DAMAGE),
        SlimeKingAbility {
            phase: SlimePhase::Idle { timer: 1.5 },
        },
        GameEntity,
    ));
}

// === Boss Movement ===

pub fn boss_movement(
    mut boss_query: Query<(&mut Transform, &Boss, Option<&BossLaser>, Option<&DragonBreath>, Option<&SlimeKingAbility>), With<Boss>>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (mut boss_transform, boss, laser, dragon, slime) in boss_query.iter_mut() {
        let (speed, can_move) = match boss.kind {
            BossKind::Eyeball => {
                let moving = if let Some(laser) = laser {
                    !matches!(laser.phase, BossLaserPhase::Charging { .. } | BossLaserPhase::Firing { .. })
                } else { true };
                (EYEBALL_SPEED, moving)
            }
            BossKind::DragonLord => {
                let moving = if let Some(d) = dragon {
                    !matches!(d.phase, DragonBreathPhase::Windup { .. } | DragonBreathPhase::Firing { .. })
                } else { true };
                (DRAGON_SPEED, moving)
            }
            BossKind::Necromancer => {
                (NECRO_SPEED, true)
            }
            BossKind::SlimeKing => {
                let moving = if let Some(s) = slime {
                    matches!(s.phase, SlimePhase::Idle { .. } | SlimePhase::Cooldown { .. })
                } else { true };
                (SLIME_SPEED, moving)
            }
        };

        if !can_move {
            continue;
        }

        let boss_pos = boss_transform.translation.truncate();
        let to_player = player_pos - boss_pos;
        let dist = to_player.length();

        // Approach radius varies by type
        let approach_dist = match boss.kind {
            BossKind::Eyeball => EYEBALL_LASER_RANGE * 0.8,
            BossKind::DragonLord => 250.0,
            BossKind::Necromancer => 200.0,
            BossKind::SlimeKing => 50.0, // Slime wants to get close
        };

        if dist > approach_dist {
            let direction = to_player.normalize_or_zero();
            boss_transform.translation.x += direction.x * speed * time.delta_secs();
            boss_transform.translation.y += direction.y * speed * time.delta_secs();
        }
    }
}

// === Boss Laser Attack (Eyeball only) ===

pub fn boss_laser_attack(
    mut commands: Commands,
    mut boss_query: Query<(Entity, &Transform, &mut BossLaser), With<Boss>>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    mut laser_mats: ResMut<Assets<BossLaserMaterial>>,
    beam_query: Query<(Entity, &BossLaserBeam, &MeshMaterial2d<BossLaserMaterial>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (boss_entity, boss_transform, mut laser) in boss_query.iter_mut() {
        let boss_pos = boss_transform.translation.truncate();
        let iris_pos = boss_pos + BOSS_IRIS_OFFSET;
        let to_player = player_pos - iris_pos;
        let dist = to_player.length();

        match laser.phase {
            BossLaserPhase::Idle => {
                if dist < laser.range {
                    let angle = to_player.y.atan2(to_player.x);
                    laser.phase = BossLaserPhase::Charging { timer: CHARGE_DURATION, angle };
                    spawn_laser_beam(&mut commands, &weapon_shaders, &mut laser_mats, boss_entity, angle);
                }
            }
            BossLaserPhase::Charging { ref mut timer, ref mut angle } => {
                *timer -= time.delta_secs();
                // Track the player during charge so the indicator follows them
                *angle = to_player.y.atan2(to_player.x);
                let progress = 1.0 - (*timer / CHARGE_DURATION);
                update_beam_charge_for_boss(boss_entity, &beam_query, &mut laser_mats, progress * 0.3);
                if *timer <= 0.0 {
                    // Start sweep from the angle we've been tracking
                    let mut rng = rand::thread_rng();
                    let sweep_dir = if rng.gen_range(0.0f32..1.0) > 0.5 { 1.0 } else { -1.0 };
                    laser.phase = BossLaserPhase::Firing {
                        timer: FIRE_DURATION,
                        angle: *angle - SWEEP_ANGLE * 0.5 * sweep_dir,
                        sweep_dir,
                    };
                }
            }
            BossLaserPhase::Firing { ref mut timer, ref mut angle, sweep_dir } => {
                *timer -= time.delta_secs();
                let sweep_speed = SWEEP_ANGLE / FIRE_DURATION;
                *angle += sweep_dir * sweep_speed * time.delta_secs();
                update_beam_charge_for_boss(boss_entity, &beam_query, &mut laser_mats, 1.0);
                if *timer <= 0.0 {
                    laser.phase = BossLaserPhase::Cooldown { timer: COOLDOWN_DURATION };
                    for (entity, beam, _) in beam_query.iter() {
                        if beam.owner == boss_entity {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
            BossLaserPhase::Cooldown { ref mut timer } => {
                *timer -= time.delta_secs();
                if *timer <= 0.0 {
                    laser.phase = BossLaserPhase::Idle;
                }
            }
        }

        if laser.hit_cooldown > 0.0 {
            laser.hit_cooldown = (laser.hit_cooldown - time.delta_secs()).max(0.0);
        }
    }
}

fn spawn_laser_beam(
    commands: &mut Commands,
    weapon_shaders: &WeaponShaderHandles,
    laser_mats: &mut Assets<BossLaserMaterial>,
    boss_entity: Entity,
    angle: f32,
) {
    let mat = laser_mats.add(BossLaserMaterial {
        data: BossLaserData {
            color: Vec4::new(0.2, 1.0, 0.3, 0.9),
            intensity: 2.0,
            charge_progress: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        },
    });

    commands.spawn((
        Mesh2d(weapon_shaders.boss_laser_quad.clone()),
        MeshMaterial2d(mat.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 12.0))
            .with_rotation(Quat::from_rotation_z(angle)),
        BossLaserBeam { owner: boss_entity },
        GameEntity,
    ));
}

fn update_beam_charge_for_boss(
    boss_entity: Entity,
    beam_query: &Query<(Entity, &BossLaserBeam, &MeshMaterial2d<BossLaserMaterial>)>,
    laser_mats: &mut Assets<BossLaserMaterial>,
    charge: f32,
) {
    for (_, beam, mat_handle) in beam_query.iter() {
        if beam.owner == boss_entity {
            if let Some(mat) = laser_mats.get_mut(&mat_handle.0) {
                mat.data.charge_progress = charge;
            }
        }
    }
}

pub fn update_boss_laser_beam(
    boss_query: Query<(Entity, &Transform, &BossLaser), With<Boss>>,
    mut beam_query: Query<(&BossLaserBeam, &mut Transform), Without<Boss>>,
) {
    let beam_half_len = 400.0;

    for (boss_entity, boss_transform, laser) in boss_query.iter() {
        let boss_pos = boss_transform.translation.truncate();
        let iris_pos = boss_pos + BOSS_IRIS_OFFSET;

        match laser.phase {
            BossLaserPhase::Charging { angle, .. } => {
                for (beam, mut beam_transform) in beam_query.iter_mut() {
                    if beam.owner != boss_entity { continue; }
                    let offset = Vec2::new(angle.cos(), angle.sin()) * beam_half_len;
                    beam_transform.translation = (iris_pos + offset).extend(12.0);
                    beam_transform.rotation = Quat::from_rotation_z(angle);
                }
            }
            BossLaserPhase::Firing { angle, .. } => {
                for (beam, mut beam_transform) in beam_query.iter_mut() {
                    if beam.owner != boss_entity { continue; }
                    let offset = Vec2::new(angle.cos(), angle.sin()) * beam_half_len;
                    beam_transform.translation = (iris_pos + offset).extend(12.0);
                    beam_transform.rotation = Quat::from_rotation_z(angle);
                }
            }
            _ => {}
        }
    }
}

pub fn boss_laser_damage_player(
    mut player_query: Query<(&Transform, &mut Health, &mut DamageCooldown), (With<Player>, Without<Boss>)>,
    boss_query: Query<(&Transform, &BossLaser), With<Boss>>,
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
) {
    let Ok((player_transform, mut health, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    if debug.invincible || cooldown.0 > 0.0 {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (boss_transform, laser) in boss_query.iter() {
        let BossLaserPhase::Firing { angle, .. } = laser.phase else {
            continue;
        };

        let boss_pos = boss_transform.translation.truncate();
        let iris_pos = boss_pos + BOSS_IRIS_OFFSET;
        let to_player = player_pos - iris_pos;
        let dist = to_player.length();

        if dist > 400.0 || dist < 30.0 {
            continue;
        }

        let beam_dir = Vec2::new(angle.cos(), angle.sin());
        let along_beam = to_player.dot(beam_dir);
        if along_beam < 0.0 {
            continue;
        }

        let perp_dist = (to_player - beam_dir * along_beam).length();
        let beam_width = 25.0;

        if perp_dist < beam_width {
            health.current -= laser.damage_per_tick;
            cooldown.0 = EYEBALL_LASER_HIT_COOLDOWN;
            play_damage_sound(&mut commands, &sound_assets);
            break;
        }
    }
}

// === Dragon Lord Attack ===

pub fn dragon_breath_attack(
    mut commands: Commands,
    mut boss_query: Query<(Entity, &Transform, &mut DragonBreath), With<Boss>>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    mut proj_mats: ResMut<Assets<ProjectileMaterial>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (_boss_entity, boss_transform, mut dragon) in boss_query.iter_mut() {
        let boss_pos = boss_transform.translation.truncate();
        let to_player = player_pos - boss_pos;
        let dist = to_player.length();

        match dragon.phase {
            DragonBreathPhase::Idle => {
                if dist < 400.0 {
                    dragon.phase = DragonBreathPhase::Windup { timer: 1.0 };
                }
            }
            DragonBreathPhase::Windup { ref mut timer } => {
                *timer -= time.delta_secs();
                if *timer <= 0.0 {
                    dragon.phase = DragonBreathPhase::Firing {
                        timer: 2.0,
                        shots_fired: 0,
                        shot_timer: 0.0,
                    };
                }
            }
            DragonBreathPhase::Firing { ref mut timer, ref mut shots_fired, ref mut shot_timer } => {
                *timer -= time.delta_secs();
                *shot_timer -= time.delta_secs();

                if *shot_timer <= 0.0 && *shots_fired < 5 {
                    // Fire a fireball at player
                    let dir = to_player.normalize_or_zero();
                    let mat = proj_mats.add(ProjectileMaterial {
                        data: ProjectileData {
                            color: Vec4::new(1.0, 0.5, 0.0, 1.0),
                            intensity: 2.5,
                            _pad1: 0.0,
                            _pad2: 0.0,
                            _pad3: 0.0,
                        },
                    });

                    commands.spawn((
                        Mesh2d(weapon_shaders.projectile_quad.clone()),
                        MeshMaterial2d(mat),
                        Transform::from_translation(boss_pos.extend(10.0))
                            .with_scale(Vec3::splat(2.5)),
                        DragonFireball {
                            damage: DRAGON_FIREBALL_DAMAGE,
                            speed: DRAGON_FIREBALL_SPEED,
                            direction: dir,
                            lifetime: DRAGON_FIREBALL_LIFETIME,
                        },
                        GameEntity,
                    ));

                    *shots_fired += 1;
                    *shot_timer = 0.4;
                }

                if *timer <= 0.0 {
                    dragon.phase = DragonBreathPhase::Cooldown { timer: 3.0 };
                }
            }
            DragonBreathPhase::Cooldown { ref mut timer } => {
                *timer -= time.delta_secs();
                if *timer <= 0.0 {
                    dragon.phase = DragonBreathPhase::Idle;
                }
            }
        }
    }
}

pub fn update_dragon_fireballs(
    mut commands: Commands,
    mut fireball_query: Query<(Entity, &mut Transform, &mut DragonFireball)>,
    time: Res<Time>,
    weapon_shaders: Res<WeaponShaderHandles>,
    mut flame_mats: ResMut<Assets<FlameAuraMaterial>>,
) {
    for (entity, mut transform, mut fireball) in fireball_query.iter_mut() {
        fireball.lifetime -= time.delta_secs();
        transform.translation.x += fireball.direction.x * fireball.speed * time.delta_secs();
        transform.translation.y += fireball.direction.y * fireball.speed * time.delta_secs();

        if fireball.lifetime <= 0.0 {
            let pos = transform.translation.truncate();
            commands.entity(entity).despawn();

            // Spawn lingering fire zone
            let mat = flame_mats.add(FlameAuraMaterial {
                data: FlameAuraData {
                    color: Vec4::new(1.0, 0.4, 0.0, 0.7),
                    intensity: 2.0,
                    inner_radius: 0.0,
                    outer_radius: DRAGON_FIRE_ZONE_RADIUS,
                    _pad: 0.0,
                },
            });

            commands.spawn((
                Mesh2d(weapon_shaders.flame_aura_quad.clone()),
                MeshMaterial2d(mat),
                Transform::from_translation(pos.extend(2.0))
                    .with_scale(Vec3::splat(DRAGON_FIRE_ZONE_RADIUS / 10.0)),
                DragonFireZone {
                    damage: DRAGON_FIRE_ZONE_DAMAGE,
                    tick_timer: 0.0,
                    lifetime: DRAGON_FIRE_ZONE_LIFETIME,
                    radius: DRAGON_FIRE_ZONE_RADIUS,
                },
                GameEntity,
            ));
        }
    }
}

pub fn dragon_fire_damage_player(
    mut player_query: Query<(&Transform, &mut Health, &mut DamageCooldown), (With<Player>, Without<Boss>)>,
    mut fireball_query: Query<(Entity, &Transform, &DragonFireball), Without<Player>>,
    mut zone_query: Query<(&Transform, &mut DragonFireZone), (Without<Player>, Without<DragonFireball>)>,
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
    time: Res<Time>,
) {
    let Ok((player_transform, mut health, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    // Fireball direct hits
    if !debug.invincible && cooldown.0 <= 0.0 {
        for (entity, fb_transform, fireball) in fireball_query.iter_mut() {
            let fb_pos = fb_transform.translation.truncate();
            if player_pos.distance(fb_pos) < crate::systems::startup::PLAYER_RADIUS + 10.0 {
                health.current -= fireball.damage;
                cooldown.0 = 0.3;
                play_damage_sound(&mut commands, &sound_assets);
                commands.entity(entity).despawn();
                break;
            }
        }
    }

    // Fire zone tick damage
    for (zone_transform, mut zone) in zone_query.iter_mut() {
        zone.lifetime -= time.delta_secs();
        zone.tick_timer -= time.delta_secs();

        if zone.lifetime <= 0.0 {
            continue; // Will be cleaned up
        }

        if debug.invincible || cooldown.0 > 0.0 {
            continue;
        }

        let zone_pos = zone_transform.translation.truncate();
        if player_pos.distance(zone_pos) < zone.radius + crate::systems::startup::PLAYER_RADIUS {
            if zone.tick_timer <= 0.0 {
                health.current -= zone.damage;
                cooldown.0 = 0.3;
                zone.tick_timer = 0.5;
                play_damage_sound(&mut commands, &sound_assets);
            }
        }
    }
}

pub fn cleanup_dragon_fire_zones(
    mut commands: Commands,
    zone_query: Query<(Entity, &DragonFireZone)>,
) {
    for (entity, zone) in zone_query.iter() {
        if zone.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// === Necromancer Attack ===

pub fn necromancer_attack(
    mut commands: Commands,
    mut boss_query: Query<(Entity, &Transform, &mut NecromancerMagic), With<Boss>>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    time: Res<Time>,
    sprite_assets: Res<SpriteAssets>,
    weapon_shaders: Res<WeaponShaderHandles>,
    mut proj_mats: ResMut<Assets<ProjectileMaterial>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (boss_entity, boss_transform, mut necro) in boss_query.iter_mut() {
        let boss_pos = boss_transform.translation.truncate();

        match necro.phase {
            NecroPhase::Idle { ref mut timer } => {
                *timer -= time.delta_secs();
                if *timer <= 0.0 {
                    necro.phase = NecroPhase::Casting { timer: 0.8 };
                }
            }
            NecroPhase::Casting { ref mut timer } => {
                *timer -= time.delta_secs();
                if *timer <= 0.0 {
                    let attack = necro.attack_index % 3;
                    necro.attack_index += 1;

                    match attack {
                        0 => {
                            // Summon skeleton minions
                            let mut rng = rand::thread_rng();
                            let count = rng.gen_range(NECRO_SUMMON_COUNT_MIN..=NECRO_SUMMON_COUNT_MAX);
                            for _ in 0..count {
                                let offset_angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                let offset_dist = rng.gen_range(40.0..80.0);
                                let spawn_pos = boss_pos + Vec2::new(offset_angle.cos(), offset_angle.sin()) * offset_dist;

                                // Skeleton sprite (index 30 from creatures.png)
                                commands.spawn((
                                    Sprite::from_atlas_image(
                                        sprite_assets.creatures_texture.clone(),
                                        TextureAtlas {
                                            layout: sprite_assets.creatures_layout.clone(),
                                            index: 30,
                                        },
                                    ),
                                    Transform::from_translation(spawn_pos.extend(4.0))
                                        .with_scale(Vec3::splat(2.0)),
                                    Enemy,
                                    EnemyType(EnemyKind::Basic),
                                    EnemyHealth(30.0),
                                    EnemySpeed(80.0),
                                    ContactDamage(8.0),
                                    NecroSummon { owner: boss_entity },
                                    GameEntity,
                                ));
                            }
                        }
                        1 => {
                            // Fire 3 homing dark orbs
                            let to_player = (player_pos - boss_pos).normalize_or_zero();
                            for i in 0..3 {
                                let spread = (i as f32 - 1.0) * 0.4;
                                let angle = to_player.y.atan2(to_player.x) + spread;
                                let dir = Vec2::new(angle.cos(), angle.sin());

                                let mat = proj_mats.add(ProjectileMaterial {
                                    data: ProjectileData {
                                        color: Vec4::new(0.6, 0.1, 0.9, 1.0),
                                        intensity: 2.5,
                                        _pad1: 0.0,
                                        _pad2: 0.0,
                                        _pad3: 0.0,
                                    },
                                });

                                commands.spawn((
                                    Mesh2d(weapon_shaders.projectile_quad.clone()),
                                    MeshMaterial2d(mat),
                                    Transform::from_translation(boss_pos.extend(10.0))
                                        .with_scale(Vec3::splat(2.0)),
                                    NecroOrb {
                                        damage: NECRO_ORB_DAMAGE,
                                        speed: NECRO_ORB_SPEED,
                                        turn_rate: NECRO_ORB_TURN_RATE,
                                        lifetime: NECRO_ORB_LIFETIME,
                                        direction: dir,
                                    },
                                    GameEntity,
                                ));
                            }
                        }
                        2 => {
                            // Teleport to random position near player
                            let mut rng = rand::thread_rng();
                            let tp_angle = rng.gen_range(0.0..std::f32::consts::TAU);
                            let tp_dist = rng.gen_range(200.0..350.0);
                            let tp_pos = player_pos + Vec2::new(tp_angle.cos(), tp_angle.sin()) * tp_dist;

                            // Spawn particles at old position
                            for _ in 0..8 {
                                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                let speed = rng.gen_range(60.0..150.0);
                                commands.spawn((
                                    Sprite {
                                        color: Color::srgba(0.6, 0.2, 0.9, 0.8),
                                        custom_size: Some(Vec2::splat(4.0)),
                                        ..default()
                                    },
                                    Transform::from_translation(boss_pos.extend(15.0)),
                                    Particle {
                                        velocity: Vec2::new(angle.cos(), angle.sin()) * speed,
                                        lifetime: 0.6,
                                    },
                                    GameEntity,
                                ));
                            }

                            // We can't mutate Transform in this query directly since it's immutable
                            // Instead, use commands to teleport
                            commands.entity(boss_entity).insert(
                                Transform::from_translation(tp_pos.extend(5.0))
                                    .with_scale(Vec3::splat(NECRO_SCALE))
                            );

                            // Spawn particles at new position
                            for _ in 0..8 {
                                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                                let speed = rng.gen_range(60.0..150.0);
                                commands.spawn((
                                    Sprite {
                                        color: Color::srgba(0.6, 0.2, 0.9, 0.8),
                                        custom_size: Some(Vec2::splat(4.0)),
                                        ..default()
                                    },
                                    Transform::from_translation(tp_pos.extend(15.0)),
                                    Particle {
                                        velocity: Vec2::new(angle.cos(), angle.sin()) * speed,
                                        lifetime: 0.6,
                                    },
                                    GameEntity,
                                ));
                            }
                        }
                        _ => {}
                    }

                    necro.phase = NecroPhase::Idle { timer: 3.0 };
                }
            }
        }
    }
}

pub fn update_necro_orbs(
    mut commands: Commands,
    mut orb_query: Query<(Entity, &mut Transform, &mut NecroOrb)>,
    player_query: Query<&Transform, (With<Player>, Without<NecroOrb>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, mut transform, mut orb) in orb_query.iter_mut() {
        orb.lifetime -= time.delta_secs();
        if orb.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Homing: steer toward player
        let orb_pos = transform.translation.truncate();
        let to_player = (player_pos - orb_pos).normalize_or_zero();
        let current_angle = orb.direction.y.atan2(orb.direction.x);
        let target_angle = to_player.y.atan2(to_player.x);
        let mut angle_diff = target_angle - current_angle;
        // Normalize to [-PI, PI]
        while angle_diff > std::f32::consts::PI { angle_diff -= std::f32::consts::TAU; }
        while angle_diff < -std::f32::consts::PI { angle_diff += std::f32::consts::TAU; }
        let max_turn = orb.turn_rate * time.delta_secs();
        let turn = angle_diff.clamp(-max_turn, max_turn);
        let new_angle = current_angle + turn;
        orb.direction = Vec2::new(new_angle.cos(), new_angle.sin());

        transform.translation.x += orb.direction.x * orb.speed * time.delta_secs();
        transform.translation.y += orb.direction.y * orb.speed * time.delta_secs();
    }
}

pub fn necro_orb_damage_player(
    mut player_query: Query<(&Transform, &mut Health, &mut DamageCooldown), (With<Player>, Without<NecroOrb>)>,
    mut commands: Commands,
    orb_query: Query<(Entity, &Transform, &NecroOrb), Without<Player>>,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
) {
    let Ok((player_transform, mut health, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    if debug.invincible || cooldown.0 > 0.0 {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (entity, orb_transform, orb) in orb_query.iter() {
        let orb_pos = orb_transform.translation.truncate();
        if player_pos.distance(orb_pos) < crate::systems::startup::PLAYER_RADIUS + 8.0 {
            health.current -= orb.damage;
            cooldown.0 = 0.3;
            play_damage_sound(&mut commands, &sound_assets);
            commands.entity(entity).despawn();
            break;
        }
    }
}

// === Slime King Attack ===

pub fn slime_king_attack_system(
    mut commands: Commands,
    mut boss_query: Query<(Entity, &mut Transform, &mut SlimeKingAbility), With<Boss>>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (_boss_entity, mut boss_transform, mut slime) in boss_query.iter_mut() {
        let boss_pos = boss_transform.translation.truncate();

        match slime.phase {
            SlimePhase::Idle { ref mut timer } => {
                *timer -= time.delta_secs();
                if *timer <= 0.0 {
                    slime.phase = SlimePhase::Windup { timer: 0.8 };
                }
            }
            SlimePhase::Windup { ref mut timer } => {
                *timer -= time.delta_secs();
                // Squish effect: flatten vertically during windup
                let progress = 1.0 - (*timer / 0.8);
                let squish_y = SLIME_SCALE * (1.0 - 0.3 * progress);
                let squish_x = SLIME_SCALE * (1.0 + 0.2 * progress);
                boss_transform.scale = Vec3::new(squish_x, squish_y, 1.0);

                if *timer <= 0.0 {
                    slime.phase = SlimePhase::Jumping {
                        timer: 0.6,
                        target: player_pos,
                    };
                    // Restore scale for jump
                    boss_transform.scale = Vec3::splat(SLIME_SCALE);
                }
            }
            SlimePhase::Jumping { ref mut timer, target } => {
                *timer -= time.delta_secs();
                // Move toward target position
                let progress = 1.0 - (*timer / 0.6);
                let start_pos = boss_pos;
                let lerped = start_pos.lerp(target, (progress * 3.0).min(1.0));
                boss_transform.translation.x = lerped.x;
                boss_transform.translation.y = lerped.y;
                // Slight scale up during jump (airborne feel)
                let jump_scale = SLIME_SCALE * (1.0 + 0.3 * (1.0 - (progress - 0.5).abs() * 2.0).max(0.0));
                boss_transform.scale = Vec3::splat(jump_scale);

                if *timer <= 0.0 {
                    slime.phase = SlimePhase::Landing { timer: 0.3 };
                    boss_transform.scale = Vec3::splat(SLIME_SCALE);

                    // Spawn shockwave at landing position
                    let land_pos = boss_transform.translation.truncate();
                    let mesh = meshes.add(Circle::new(1.0));
                    let mat = materials.add(ColorMaterial::from(Color::srgba(0.3, 0.9, 0.3, 0.5)));

                    commands.spawn((
                        Mesh2d(mesh),
                        MeshMaterial2d(mat),
                        Transform::from_translation(land_pos.extend(3.0))
                            .with_scale(Vec3::splat(10.0)),
                        SlimeShockwave {
                            damage: SLIME_SHOCKWAVE_DAMAGE,
                            current_radius: 10.0,
                            max_radius: SLIME_SHOCKWAVE_MAX_RADIUS,
                            lifetime: 0.5,
                            hit_player: false,
                        },
                        GameEntity,
                    ));
                }
            }
            SlimePhase::Landing { ref mut timer } => {
                *timer -= time.delta_secs();
                // Impact squish
                let progress = 1.0 - (*timer / 0.3);
                let squish = 1.0 - 0.2 * (1.0 - progress);
                boss_transform.scale = Vec3::new(
                    SLIME_SCALE * (2.0 - squish),
                    SLIME_SCALE * squish,
                    1.0,
                );
                if *timer <= 0.0 {
                    boss_transform.scale = Vec3::splat(SLIME_SCALE);
                    slime.phase = SlimePhase::Cooldown { timer: 2.0 };
                }
            }
            SlimePhase::Cooldown { ref mut timer } => {
                *timer -= time.delta_secs();
                if *timer <= 0.0 {
                    slime.phase = SlimePhase::Idle { timer: 0.5 };
                }
            }
        }
    }
}

pub fn update_slime_shockwaves(
    mut commands: Commands,
    mut shock_query: Query<(Entity, &mut Transform, &mut SlimeShockwave)>,
    mut player_query: Query<(&Transform, &mut Health, &mut DamageCooldown), (With<Player>, Without<SlimeShockwave>)>,
    time: Res<Time>,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
) {
    // First pass: update shockwaves and despawn expired
    for (entity, mut transform, mut shock) in shock_query.iter_mut() {
        shock.lifetime -= time.delta_secs();
        if shock.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let expand_speed = SLIME_SHOCKWAVE_MAX_RADIUS / 0.5;
        shock.current_radius = (shock.current_radius + expand_speed * time.delta_secs()).min(shock.max_radius);
        transform.scale = Vec3::splat(shock.current_radius);
    }

    // Second pass: check player collision
    let Ok((player_transform, mut health, mut cooldown)) = player_query.single_mut() else {
        return;
    };
    if debug.invincible || cooldown.0 > 0.0 {
        return;
    }
    let player_pos = player_transform.translation.truncate();

    for (_entity, transform, mut shock) in shock_query.iter_mut() {
        if shock.hit_player { continue; }
        let shock_pos = transform.translation.truncate();
        let dist = player_pos.distance(shock_pos);
        if dist < shock.current_radius + crate::systems::startup::PLAYER_RADIUS {
            health.current -= shock.damage;
            cooldown.0 = 0.5;
            shock.hit_player = true;
            play_damage_sound(&mut commands, &sound_assets);
            break;
        }
    }
}

pub fn update_slime_splits(
    mut split_query: Query<(&mut Transform, &SlimeSplit), Without<Player>>,
    player_query: Query<&Transform, (With<Player>, Without<SlimeSplit>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (mut transform, split) in split_query.iter_mut() {
        let pos = transform.translation.truncate();
        let to_player = (player_pos - pos).normalize_or_zero();
        transform.translation.x += to_player.x * split.speed * time.delta_secs();
        transform.translation.y += to_player.y * split.speed * time.delta_secs();
    }
}

pub fn slime_split_damage_player(
    mut player_query: Query<(&Transform, &mut Health, &mut DamageCooldown), (With<Player>, Without<SlimeSplit>)>,
    split_query: Query<(&Transform, &SlimeSplit), Without<Player>>,
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
) {
    let Ok((player_transform, mut health, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    if debug.invincible || cooldown.0 > 0.0 {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (split_transform, split) in split_query.iter() {
        let split_pos = split_transform.translation.truncate();
        if player_pos.distance(split_pos) < crate::systems::startup::PLAYER_RADIUS + SLIME_SPLIT_RADIUS {
            health.current -= split.damage;
            cooldown.0 = 0.3;
            play_damage_sound(&mut commands, &sound_assets);
            break;
        }
    }
}

pub fn weapon_slime_split_collision(
    mut commands: Commands,
    mut projectile_query: Query<
        (Entity, &Transform, &mut Projectile),
        (Without<SlimeSplit>, Without<Player>),
    >,
    mut boomerang_query: Query<
        (&Transform, &mut BoomerangProjectile),
        (Without<SlimeSplit>, Without<Player>, Without<Projectile>),
    >,
    mut split_query: Query<(Entity, &Transform, &mut SlimeSplit)>,
    debug: Res<DebugSettings>,
    game_meshes: Res<GameMeshes>,
    game_materials: Res<GameMaterials>,
) {
    let dmg_mult = debug.damage_multiplier;

    for (proj_entity, proj_transform, mut projectile) in projectile_query.iter_mut() {
        let proj_pos = proj_transform.translation.truncate();
        for (split_entity, split_transform, mut split) in split_query.iter_mut() {
            if split.health <= 0.0 { continue; }
            if projectile.hit_enemies.contains(&split_entity) { continue; }
            let split_pos = split_transform.translation.truncate();
            if proj_pos.distance(split_pos) < weapon_radii::PROJECTILE + SLIME_SPLIT_RADIUS {
                let dmg = projectile.damage * dmg_mult;
                split.health -= dmg;
                spawn_boss_damage_number(&mut commands, split_pos, dmg);
                if projectile.piercing {
                    projectile.hit_enemies.push(split_entity);
                } else {
                    commands.entity(proj_entity).despawn();
                    break;
                }
            }
        }
    }

    for (boom_transform, mut boomerang) in boomerang_query.iter_mut() {
        let boom_pos = boom_transform.translation.truncate();
        for (split_entity, split_transform, mut split) in split_query.iter_mut() {
            if split.health <= 0.0 { continue; }
            if boomerang.hit_enemies.contains(&split_entity) { continue; }
            let split_pos = split_transform.translation.truncate();
            if boom_pos.distance(split_pos) < weapon_radii::BOOMERANG + SLIME_SPLIT_RADIUS {
                let dmg = boomerang.damage * dmg_mult;
                split.health -= dmg;
                spawn_boss_damage_number(&mut commands, split_pos, dmg);
                boomerang.hit_enemies.push(split_entity);
            }
        }
    }

    // Death check for splits
    for (entity, transform, split) in split_query.iter() {
        if split.health <= 0.0 {
            let pos = transform.translation.truncate();
            commands.entity(entity).despawn();

            // Small death particles
            let mut rng = rand::thread_rng();
            for _ in 0..6 {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let speed = rng.gen_range(60.0..150.0);
                commands.spawn((
                    Mesh2d(game_meshes.projectile.clone()),
                    MeshMaterial2d(game_materials.xp_gem.clone()),
                    Transform::from_translation(pos.extend(15.0))
                        .with_scale(Vec3::splat(0.5)),
                    Particle {
                        velocity: Vec2::new(angle.cos(), angle.sin()) * speed,
                        lifetime: 0.5,
                    },
                    GameEntity,
                ));
            }

            // Drop small XP
            commands.spawn((
                Mesh2d(game_meshes.xp_gem.clone()),
                MeshMaterial2d(game_materials.xp_gem.clone()),
                Transform::from_translation(pos.extend(1.0))
                    .with_scale(Vec3::splat(1.5))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                XpGem(10.0),
                GameEntity,
            ));
        }
    }
}

// === Boss Contact Damage to Player ===

pub fn boss_contact_damage(
    mut player_query: Query<(&Transform, &mut Health, &mut DamageCooldown), (With<Player>, Without<Boss>, Without<Dashing>)>,
    boss_query: Query<(&Transform, &ContactDamage, &Boss)>,
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    debug: Res<DebugSettings>,
) {
    let Ok((player_transform, mut health, mut cooldown)) = player_query.single_mut() else {
        return;
    };

    if debug.invincible || cooldown.0 > 0.0 {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (boss_transform, damage, boss) in boss_query.iter() {
        let boss_pos = boss_transform.translation.truncate();
        let dist = player_pos.distance(boss_pos);
        let radius = boss_radius(boss.kind);

        if dist < crate::systems::startup::PLAYER_RADIUS + radius {
            health.current -= damage.0;
            cooldown.0 = 0.5;
            play_damage_sound(&mut commands, &sound_assets);
            break;
        }
    }
}

// === Weapons Damage Boss ===

pub fn weapon_boss_collision(
    mut commands: Commands,
    mut orbit_query: Query<(&Transform, &mut OrbitShield), (Without<Boss>, Without<Player>)>,
    mut projectile_query: Query<
        (Entity, &Transform, &mut Projectile),
        (Without<Boss>, Without<Player>, Without<OrbitShield>),
    >,
    mut boomerang_query: Query<
        (&Transform, &mut BoomerangProjectile),
        (Without<Boss>, Without<Player>, Without<OrbitShield>, Without<Projectile>, Without<BoneProjectile>),
    >,
    mut bone_query: Query<
        (&Transform, &mut BoneProjectile),
        (Without<Boss>, Without<Player>, Without<OrbitShield>, Without<Projectile>, Without<BoomerangProjectile>),
    >,
    mut updown_query: Query<
        (&Transform, &mut UpDownWave),
        (Without<Boss>, Without<Player>, Without<OrbitShield>, Without<Projectile>, Without<BoomerangProjectile>, Without<BoneProjectile>),
    >,
    mut boss_query: Query<(Entity, &Transform, &mut BossHealth, &Boss)>,
    weapons: Res<PlayerWeapons>,
    debug: Res<DebugSettings>,
) {
    let dmg_mult = debug.damage_multiplier;
    let orbit_damage = weapons
        .get(WeaponKind::OrbitShield)
        .map(|w| w.damage * dmg_mult)
        .unwrap_or(0.0);

    for (orbit_transform, mut orbit) in orbit_query.iter_mut() {
        if orbit.hit_cooldown > 0.0 {
            continue;
        }
        let orbit_pos = orbit_transform.translation.truncate();

        for (_entity, boss_transform, mut health, boss) in boss_query.iter_mut() {
            if health.current <= 0.0 { continue; }
            let boss_pos = boss_transform.translation.truncate();
            let radius = boss_radius(boss.kind);
            if orbit_pos.distance(boss_pos) < weapon_radii::ORBIT_SHIELD + radius {
                health.current -= orbit_damage;
                orbit.hit_cooldown = 0.3;
                spawn_boss_damage_number(&mut commands, boss_pos, orbit_damage);
                break;
            }
        }
    }

    for (proj_entity, proj_transform, mut projectile) in projectile_query.iter_mut() {
        let proj_pos = proj_transform.translation.truncate();
        for (boss_entity, boss_transform, mut health, boss) in boss_query.iter_mut() {
            if health.current <= 0.0 { continue; }
            if projectile.hit_enemies.contains(&boss_entity) { continue; }
            let boss_pos = boss_transform.translation.truncate();
            let radius = boss_radius(boss.kind);
            if proj_pos.distance(boss_pos) < weapon_radii::PROJECTILE + radius {
                let dmg = projectile.damage * dmg_mult;
                health.current -= dmg;
                spawn_boss_damage_number(&mut commands, boss_pos, dmg);
                if projectile.piercing {
                    projectile.hit_enemies.push(boss_entity);
                } else {
                    commands.entity(proj_entity).despawn();
                    break;
                }
            }
        }
    }

    for (boom_transform, mut boomerang) in boomerang_query.iter_mut() {
        let boom_pos = boom_transform.translation.truncate();
        for (boss_entity, boss_transform, mut health, boss) in boss_query.iter_mut() {
            if health.current <= 0.0 { continue; }
            if boomerang.hit_enemies.contains(&boss_entity) { continue; }
            let boss_pos = boss_transform.translation.truncate();
            let radius = boss_radius(boss.kind);
            if boom_pos.distance(boss_pos) < weapon_radii::BOOMERANG + radius {
                let dmg = boomerang.damage * dmg_mult;
                health.current -= dmg;
                spawn_boss_damage_number(&mut commands, boss_pos, dmg);
                boomerang.hit_enemies.push(boss_entity);
            }
        }
    }

    for (bone_transform, mut bone) in bone_query.iter_mut() {
        let bone_pos = bone_transform.translation.truncate();
        for (boss_entity, boss_transform, mut health, boss) in boss_query.iter_mut() {
            if health.current <= 0.0 { continue; }
            if bone.hit_enemies.contains(&boss_entity) { continue; }
            let boss_pos = boss_transform.translation.truncate();
            let radius = boss_radius(boss.kind);
            if bone_pos.distance(boss_pos) < weapon_radii::BONE + radius {
                let dmg = bone.damage * dmg_mult;
                health.current -= dmg;
                spawn_boss_damage_number(&mut commands, boss_pos, dmg);
                bone.hit_enemies.push(boss_entity);
            }
        }
    }

    for (wave_transform, mut wave) in updown_query.iter_mut() {
        let wave_pos = wave_transform.translation.truncate();
        for (boss_entity, boss_transform, mut health, boss) in boss_query.iter_mut() {
            if health.current <= 0.0 { continue; }
            if wave.hit_enemies.contains(&boss_entity) { continue; }
            let boss_pos = boss_transform.translation.truncate();
            let radius = boss_radius(boss.kind);
            if wave_pos.distance(boss_pos) < weapon_radii::UPDOWN + radius {
                let dmg = wave.damage * dmg_mult;
                health.current -= dmg;
                spawn_boss_damage_number(&mut commands, boss_pos, dmg);
                wave.hit_enemies.push(boss_entity);
            }
        }
    }
}

// === Dash Damages Boss ===

pub fn dash_boss_collision(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Dashing), With<Player>>,
    mut boss_query: Query<(Entity, &Transform, &mut BossHealth, &Boss)>,
    debug: Res<DebugSettings>,
) {
    let Ok((player_transform, mut dashing)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (boss_entity, boss_transform, mut health, boss) in boss_query.iter_mut() {
        if health.current <= 0.0 { continue; }
        if dashing.hit_enemies.contains(&boss_entity) { continue; }
        let boss_pos = boss_transform.translation.truncate();
        let radius = boss_radius(boss.kind);
        if player_pos.distance(boss_pos) < crate::systems::startup::PLAYER_RADIUS + radius {
            let dmg = dashing.damage * debug.damage_multiplier;
            health.current -= dmg;
            spawn_boss_damage_number(&mut commands, boss_pos, dmg);
            dashing.hit_enemies.push(boss_entity);
        }
    }
}

// === Boss Death ===

pub fn boss_death(
    mut commands: Commands,
    boss_query: Query<(Entity, &Transform, &BossHealth, &Boss)>,
    game_meshes: Res<GameMeshes>,
    game_materials: Res<GameMaterials>,
    sound_assets: Res<SoundAssets>,
    beam_query: Query<(Entity, &BossLaserBeam)>,
    healthbar_query: Query<(Entity, &BossHealthBarOwner)>,
    summon_query: Query<(Entity, &NecroSummon)>,
    sprite_assets: Res<SpriteAssets>,
    mut notif_events: MessageWriter<NotificationEvent>,
    mut stats: ResMut<GameStats>,
) {
    for (entity, transform, health, boss) in boss_query.iter() {
        if health.current > 0.0 {
            continue;
        }

        let pos = transform.translation.truncate();
        let kind = boss.kind;
        commands.entity(entity).despawn();
        stats.bosses_killed += 1;

        // Per-type cleanup
        match kind {
            BossKind::Eyeball => {
                for (beam_entity, beam) in beam_query.iter() {
                    if beam.owner == entity {
                        commands.entity(beam_entity).despawn();
                    }
                }
            }
            BossKind::Necromancer => {
                // Despawn all summoned minions
                for (summon_entity, summon) in summon_query.iter() {
                    if summon.owner == entity {
                        commands.entity(summon_entity).despawn();
                    }
                }
            }
            BossKind::SlimeKing => {
                // Spawn 2 mini-slimes
                for i in 0..2 {
                    let offset = if i == 0 { Vec2::new(-30.0, 0.0) } else { Vec2::new(30.0, 0.0) };
                    let split_pos = pos + offset;
                    commands.spawn((
                        Sprite {
                            image: sprite_assets.creatures_texture.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: sprite_assets.creatures_layout.clone(),
                                index: SLIME_SPRITE_INDEX,
                            }),
                            color: Color::srgb(0.5, 1.0, 0.5),
                            ..default()
                        },
                        Transform::from_translation(split_pos.extend(4.0))
                            .with_scale(Vec3::splat(SLIME_SPLIT_SCALE)),
                        SlimeSplit {
                            health: SLIME_SPLIT_HEALTH,
                            speed: SLIME_SPLIT_SPEED,
                            damage: SLIME_SPLIT_DAMAGE,
                        },
                        GameEntity,
                    ));
                }
            }
            BossKind::DragonLord => {} // No special cleanup
        }

        // Despawn health bar entities
        for (bar_entity, owner) in healthbar_query.iter() {
            if owner.0 == entity {
                commands.entity(bar_entity).despawn();
            }
        }

        // Play death sound
        let idx = rand::thread_rng().gen_range(0..5);
        commands.spawn((
            AudioPlayer::<AudioSource>::new(sound_assets.enemy_death[idx].clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.4)),
        ));

        // Drop big XP gem
        commands.spawn((
            Mesh2d(game_meshes.xp_gem.clone()),
            MeshMaterial2d(game_materials.xp_gem.clone()),
            Transform::from_translation(pos.extend(1.0))
                .with_scale(Vec3::splat(3.0))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            XpGem(BOSS_XP_DROP),
            GameEntity,
        ));

        // Big death particle explosion
        let mut rng = rand::thread_rng();
        for _ in 0..16 {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(120.0..350.0);
            let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
            commands.spawn((
                Mesh2d(game_meshes.projectile.clone()),
                MeshMaterial2d(game_materials.xp_gem.clone()),
                Transform::from_translation(pos.extend(15.0))
                    .with_scale(Vec3::splat(rng.gen_range(0.5..1.5))),
                Particle {
                    velocity: vel,
                    lifetime: rng.gen_range(0.5..1.0),
                },
                GameEntity,
            ));
        }

        notif_events.write(NotificationEvent {
            message: format!("{} Defeated!", kind.display_name()),
            kind: NotificationType::Boss,
        });
    }
}

// === Boss Sprite Phase (swap texture based on HP) ===

pub fn boss_sprite_phase(
    mut boss_query: Query<(&BossHealth, &Boss, &mut Sprite)>,
    boss_assets: Res<BossAssets>,
) {
    for (health, boss, mut sprite) in boss_query.iter_mut() {
        let pct = health.current / health.max;
        match boss.kind {
            BossKind::Eyeball => {
                let texture = if pct > 0.66 {
                    boss_assets.eye_texture_1.clone()
                } else if pct > 0.33 {
                    boss_assets.eye_texture_2.clone()
                } else {
                    boss_assets.eye_texture_3.clone()
                };
                sprite.image = texture;
            }
            _ => {
                // Color tint for non-eyeball bosses at HP thresholds
                let tint = if pct > 0.66 {
                    Color::WHITE
                } else if pct > 0.33 {
                    Color::srgb(1.0, 0.7, 0.7)
                } else {
                    Color::srgb(1.0, 0.4, 0.4)
                };
                // Preserve green tint for slime king
                if boss.kind == BossKind::SlimeKing {
                    let r = if pct > 0.66 { 0.5 } else if pct > 0.33 { 0.7 } else { 0.9 };
                    sprite.color = Color::srgb(r, 1.0, 0.5);
                } else {
                    sprite.color = tint;
                }
            }
        }
    }
}

// === Boss Sprite Animation ===

pub fn boss_animate(
    mut boss_query: Query<(&mut Transform, &mut Sprite, &Boss)>,
    time: Res<Time>,
) {
    for (mut transform, mut sprite, boss) in boss_query.iter_mut() {
        match boss.kind {
            BossKind::Eyeball => {
                let frame = ((time.elapsed_secs() * 4.0) as usize) % 6;
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = frame;
                }
            }
            _ => {
                // Gentle scale pulse for non-eyeball bosses
                let base_scale = match boss.kind {
                    BossKind::DragonLord => DRAGON_SCALE,
                    BossKind::Necromancer => NECRO_SCALE,
                    BossKind::SlimeKing => SLIME_SCALE,
                    BossKind::Eyeball => unreachable!(),
                };
                // Only pulse if not being animated by attack systems
                // (SlimeKing handles its own scale in attack system)
                if boss.kind != BossKind::SlimeKing {
                    let pulse = 1.0 + 0.03 * (time.elapsed_secs() * 2.0).sin();
                    transform.scale = Vec3::splat(base_scale * pulse);
                }
            }
        }
    }
}

// === Boss Health Bar (World-Space) ===

const BOSS_BAR_WIDTH: f32 = 80.0;
const BOSS_BAR_HEIGHT: f32 = 6.0;
const BOSS_BAR_Y_OFFSET: f32 = 85.0;

pub fn spawn_boss_health_bar(
    mut commands: Commands,
    boss_query: Query<(Entity, &Boss), Added<Boss>>,
) {
    for (boss_entity, boss) in boss_query.iter() {
        let name = boss.kind.display_name();
        let name_color = boss.kind.name_color();
        let bar_color = boss.kind.bar_color();

        commands.spawn((
            Text2d::new(name),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(name_color),
            Transform::from_translation(Vec3::new(0.0, 0.0, 20.0)),
            BossNameText,
            BossHealthBarOwner(boss_entity),
            GameEntity,
        ));

        commands.spawn((
            Sprite {
                color: Color::srgba(0.15, 0.0, 0.0, 0.85),
                custom_size: Some(Vec2::new(BOSS_BAR_WIDTH, BOSS_BAR_HEIGHT)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 19.0)),
            BossHealthBarBg,
            BossHealthBarOwner(boss_entity),
            GameEntity,
        ));

        commands.spawn((
            Sprite {
                color: bar_color,
                custom_size: Some(Vec2::new(BOSS_BAR_WIDTH, BOSS_BAR_HEIGHT)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 19.5)),
            BossHealthBarFill,
            BossHealthBarOwner(boss_entity),
            GameEntity,
        ));
    }
}

pub fn update_boss_health_bar(
    boss_query: Query<(Entity, &Transform, &BossHealth), With<Boss>>,
    mut bar_bg_query: Query<
        (&mut Transform, &BossHealthBarOwner),
        (With<BossHealthBarBg>, Without<Boss>, Without<BossHealthBarFill>, Without<BossNameText>),
    >,
    mut bar_fill_query: Query<
        (&mut Transform, &mut Sprite, &BossHealthBarOwner),
        (With<BossHealthBarFill>, Without<Boss>, Without<BossHealthBarBg>, Without<BossNameText>),
    >,
    mut name_query: Query<
        (&mut Transform, &BossHealthBarOwner),
        (With<BossNameText>, Without<Boss>, Without<BossHealthBarBg>, Without<BossHealthBarFill>),
    >,
) {
    let bosses: Vec<(Entity, Vec2, f32)> = boss_query
        .iter()
        .map(|(e, t, h)| (e, t.translation.truncate(), (h.current / h.max).clamp(0.0, 1.0)))
        .collect();

    for (mut transform, owner) in bar_bg_query.iter_mut() {
        if let Some((_, pos, _)) = bosses.iter().find(|(e, _, _)| *e == owner.0) {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y + BOSS_BAR_Y_OFFSET;
        }
    }

    for (mut transform, mut sprite, owner) in bar_fill_query.iter_mut() {
        if let Some((_, pos, pct)) = bosses.iter().find(|(e, _, _)| *e == owner.0) {
            let fill_width = BOSS_BAR_WIDTH * pct;
            sprite.custom_size = Some(Vec2::new(fill_width, BOSS_BAR_HEIGHT));
            transform.translation.x = pos.x - (BOSS_BAR_WIDTH - fill_width) * 0.5;
            transform.translation.y = pos.y + BOSS_BAR_Y_OFFSET;
        }
    }

    for (mut transform, owner) in name_query.iter_mut() {
        if let Some((_, pos, _)) = bosses.iter().find(|(e, _, _)| *e == owner.0) {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y + BOSS_BAR_Y_OFFSET + 10.0;
        }
    }
}

// === Holy Water damages Boss ===

pub fn holy_water_damages_boss(
    mut commands: Commands,
    mut boss_query: Query<(&Transform, &mut BossHealth, &Boss)>,
    zone_query: Query<(&Transform, &HolyWaterZone)>,
    debug: Res<DebugSettings>,
) {
    for (zone_transform, zone) in zone_query.iter() {
        let zone_pos = zone_transform.translation.truncate();
        for (boss_transform, mut health, boss) in boss_query.iter_mut() {
            if health.current <= 0.0 { continue; }
            let boss_pos = boss_transform.translation.truncate();
            let radius = boss_radius(boss.kind);
            let dist = zone_pos.distance(boss_pos);
            if dist < zone.radius + radius {
                let dmg = zone.damage * debug.damage_multiplier;
                health.current -= dmg;
                spawn_boss_damage_number(&mut commands, boss_pos, dmg);
            }
        }
    }
}

// === Flame Aura damages Boss ===

pub fn flame_aura_damages_boss(
    mut commands: Commands,
    mut boss_query: Query<(&Transform, &mut BossHealth, &Boss)>,
    player_query: Query<&Transform, With<Player>>,
    weapons: Res<PlayerWeapons>,
    debug: Res<DebugSettings>,
) {
    let Some(flame) = weapons.get(WeaponKind::FlameAura) else {
        return;
    };

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (boss_transform, mut health, boss) in boss_query.iter_mut() {
        if health.current <= 0.0 { continue; }
        let boss_pos = boss_transform.translation.truncate();
        let radius = boss_radius(boss.kind);
        let dist = player_pos.distance(boss_pos);
        if dist < flame.area + radius {
            let dmg = flame.damage * debug.damage_multiplier;
            health.current -= dmg;
            spawn_boss_damage_number(&mut commands, boss_pos, dmg);
        }
    }
}
