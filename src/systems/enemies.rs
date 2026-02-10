use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;

const MAX_ENEMIES: u32 = 200;
const SPAWN_DISTANCE: f32 = 600.0;

// Flock boid parameters
const FLOCK_COHESION: f32 = 50.0;
const FLOCK_SEPARATION: f32 = 80.0;
const FLOCK_SEPARATION_DIST: f32 = 30.0;
const FLOCK_ALIGNMENT: f32 = 40.0;
const FLOCK_PLAYER_CHASE: f32 = 60.0;
const FLOCK_GROUP_SIZE_MIN: u32 = 5;
const FLOCK_GROUP_SIZE_MAX: u32 = 8;
const FLOCK_SPAWN_INTERVAL: f32 = 12.0;
const FLOCK_MIN_ELAPSED: f32 = 90.0;

/// Counter for assigning unique flock IDs
#[derive(Resource, Default)]
pub struct FlockIdCounter(pub u32);

pub fn enemy_spawning(
    mut commands: Commands,
    mut wave: ResMut<WaveManager>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    sprites: Res<SpriteAssets>,
    debug: Res<DebugSettings>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    wave.elapsed += time.delta_secs();
    wave.spawn_timer -= time.delta_secs();

    if wave.spawn_timer > 0.0 || wave.enemies_alive >= MAX_ENEMIES {
        return;
    }

    // Difficulty scales over time
    let difficulty = 1.0 + wave.elapsed / 30.0;

    // Spawn interval decreases over time (more enemies)
    // Apply spawn rate multiplier: higher = faster spawning (shorter interval)
    let base_interval = (1.2 - wave.elapsed * 0.005).max(0.15);
    wave.spawn_interval = if debug.spawn_rate_multiplier > 0.0 {
        base_interval / debug.spawn_rate_multiplier
    } else {
        f32::MAX // effectively stops spawning at 0x
    };
    wave.spawn_timer = wave.spawn_interval;

    // Determine how many to spawn this tick
    let batch_size = (difficulty as u32).min(5).max(1);

    let mut rng = rand::thread_rng();

    for _ in 0..batch_size {
        if wave.enemies_alive >= MAX_ENEMIES {
            break;
        }

        let kind = pick_enemy_kind(wave.elapsed, &mut rng);
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let spawn_pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * SPAWN_DISTANCE;

        spawn_enemy(&mut commands, &sprites, kind, spawn_pos, None);
        wave.enemies_alive += 1;
    }

    // Every 60 seconds, spawn a burst of enemies
    let burst_interval = 60.0;
    let bursts_passed = (wave.elapsed / burst_interval) as u32;
    let prev_bursts = ((wave.elapsed - time.delta_secs()) / burst_interval) as u32;
    if bursts_passed > prev_bursts {
        let burst_count = (10 + bursts_passed * 5).min(40);
        for _ in 0..burst_count {
            if wave.enemies_alive >= MAX_ENEMIES {
                break;
            }
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let spawn_pos = player_pos + Vec2::new(angle.cos(), angle.sin()) * SPAWN_DISTANCE;
            let kind = pick_enemy_kind(wave.elapsed, &mut rng);
            spawn_enemy(&mut commands, &sprites, kind, spawn_pos, None);
            wave.enemies_alive += 1;
        }
    }
}

/// Separate spawning system for flock groups
pub fn flock_spawning(
    mut commands: Commands,
    mut wave: ResMut<WaveManager>,
    mut flock_counter: ResMut<FlockIdCounter>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    sprites: Res<SpriteAssets>,
) {
    if wave.elapsed < FLOCK_MIN_ELAPSED {
        return;
    }

    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    // Use a simple time-based check: spawn a flock every FLOCK_SPAWN_INTERVAL seconds
    let interval = FLOCK_SPAWN_INTERVAL;
    let current = wave.elapsed;
    let previous = current - time.delta_secs();
    let spawns_now = (current / interval) as u32;
    let spawns_prev = (previous / interval) as u32;

    if spawns_now <= spawns_prev {
        return;
    }

    let mut rng = rand::thread_rng();
    let group_size = rng.gen_range(FLOCK_GROUP_SIZE_MIN..=FLOCK_GROUP_SIZE_MAX);

    // Pick a random direction to spawn the flock from
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let center = player_pos + Vec2::new(angle.cos(), angle.sin()) * SPAWN_DISTANCE;

    let flock_id = flock_counter.0;
    flock_counter.0 += 1;

    for _ in 0..group_size {
        if wave.enemies_alive >= MAX_ENEMIES {
            break;
        }
        // Scatter slightly around the center point
        let offset = Vec2::new(
            rng.gen_range(-40.0..40.0),
            rng.gen_range(-40.0..40.0),
        );
        spawn_enemy(&mut commands, &sprites, EnemyKind::Flock, center + offset, Some(flock_id));
        wave.enemies_alive += 1;
    }
}

fn pick_enemy_kind(elapsed: f32, rng: &mut impl Rng) -> EnemyKind {
    let roll: f32 = rng.gen_range(0.0f32..1.0);
    if elapsed < 30.0 {
        EnemyKind::Basic
    } else if elapsed < 60.0 {
        if roll < 0.6 {
            EnemyKind::Basic
        } else {
            EnemyKind::Fast
        }
    } else if elapsed < 120.0 {
        if roll < 0.4 {
            EnemyKind::Basic
        } else if roll < 0.7 {
            EnemyKind::Fast
        } else {
            EnemyKind::Tank
        }
    } else {
        if roll < 0.25 {
            EnemyKind::Basic
        } else if roll < 0.45 {
            EnemyKind::Fast
        } else if roll < 0.65 {
            EnemyKind::Tank
        } else {
            EnemyKind::Swarm
        }
    }
}

// Creature tilemap: 10 cols x 18 rows, each tile is a UNIQUE creature (no animation frames).
// Pick a single static sprite index per enemy, with a few variants for visual variety.
// Row 0: Demons/horned - indices 0-9
// Row 1: Humanoid monsters - indices 10-19
// Row 2: Undead/ghosts - indices 20-29
// Row 3: Small flying/floating - indices 30-39
// Row 5: Elemental/armored - indices 50-59
// Row 8-9: Large beasts - indices 80-99

fn pick_sprite_index(kind: EnemyKind, rng: &mut impl Rng) -> (usize, f32) {
    // Returns (sprite_index, scale)
    match kind {
        // Demons from row 0 - pick one of a few
        EnemyKind::Basic => {
            let options = [0, 1, 2, 3];
            (options[rng.gen_range(0..options.len())], 2.0)
        }
        // Undead from row 2 - smaller, faster looking
        EnemyKind::Fast => {
            let options = [20, 21, 22, 23];
            (options[rng.gen_range(0..options.len())], 1.8)
        }
        // Large armored from row 5 - big and tough
        EnemyKind::Tank => {
            let options = [50, 51, 52];
            (options[rng.gen_range(0..options.len())], 2.5)
        }
        // Small creatures from row 3 - tiny swarm
        EnemyKind::Swarm => {
            let options = [30, 31, 32, 33, 34];
            (options[rng.gen_range(0..options.len())], 1.3)
        }
        // Flying creatures from row 3 (second half) - flock birds/bats
        EnemyKind::Flock => {
            let options = [35, 36, 37, 38, 39];
            (options[rng.gen_range(0..options.len())], 1.5)
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    kind: EnemyKind,
    position: Vec2,
    flock_id: Option<u32>,
) {
    let (health, speed, damage, z) = match kind {
        EnemyKind::Basic => (10.0, 80.0, 10.0, 5.0),
        EnemyKind::Fast => (5.0, 150.0, 8.0, 5.0),
        EnemyKind::Tank => (30.0, 50.0, 20.0, 4.0),
        EnemyKind::Swarm => (3.0, 100.0, 5.0, 5.0),
        EnemyKind::Flock => (4.0, 110.0, 6.0, 5.0),
    };

    let mut rng = rand::thread_rng();
    let (sprite_index, scale) = pick_sprite_index(kind, &mut rng);

    let mut entity = commands.spawn((
        Sprite::from_atlas_image(
            sprites.creatures_texture.clone(),
            TextureAtlas {
                layout: sprites.creatures_layout.clone(),
                index: sprite_index,
            },
        ),
        Transform::from_translation(position.extend(z))
            .with_scale(Vec3::splat(scale)),
        Enemy,
        EnemyType(kind),
        EnemyHealth(health),
        ContactDamage(damage),
        EnemySpeed(speed),
        GameEntity,
    ));

    if let Some(id) = flock_id {
        entity.insert(FlockMember { flock_id: id });
    }
}

pub fn enemy_movement(
    mut enemy_query: Query<(&mut Transform, &EnemySpeed), (With<Enemy>, Without<FlockMember>)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (mut transform, speed) in enemy_query.iter_mut() {
        let enemy_pos = transform.translation.truncate();
        let direction = (player_pos - enemy_pos).normalize_or_zero();
        transform.translation.x += direction.x * speed.0 * time.delta_secs();
        transform.translation.y += direction.y * speed.0 * time.delta_secs();
    }
}

/// Boid-style flocking movement for FlockMember enemies
pub fn flock_movement(
    mut flock_query: Query<(Entity, &mut Transform, &EnemySpeed, &FlockMember), With<Enemy>>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    // Collect all flock member positions first
    let flock_data: Vec<(Entity, Vec2, u32)> = flock_query
        .iter()
        .map(|(e, t, _, f)| (e, t.translation.truncate(), f.flock_id))
        .collect();

    for (entity, mut transform, speed, flock) in flock_query.iter_mut() {
        let my_pos = transform.translation.truncate();
        let my_flock = flock.flock_id;

        // Gather same-flock neighbors
        let mut cohesion_center = Vec2::ZERO;
        let mut separation_force = Vec2::ZERO;
        let mut alignment_dir = Vec2::ZERO;
        let mut neighbor_count = 0u32;

        for &(other_entity, other_pos, other_flock) in &flock_data {
            if other_entity == entity || other_flock != my_flock {
                continue;
            }
            let diff = other_pos - my_pos;
            let dist = diff.length();

            cohesion_center += other_pos;
            neighbor_count += 1;

            // Separation: push away from close neighbors
            if dist < FLOCK_SEPARATION_DIST && dist > 0.1 {
                separation_force -= diff / dist;
            }

            // Alignment: move in roughly the same direction (toward player)
            let other_to_player = (player_pos - other_pos).normalize_or_zero();
            alignment_dir += other_to_player;
        }

        let mut velocity = Vec2::ZERO;

        if neighbor_count > 0 {
            // Cohesion: steer toward flock center
            cohesion_center /= neighbor_count as f32;
            let cohesion_dir = (cohesion_center - my_pos).normalize_or_zero();
            velocity += cohesion_dir * FLOCK_COHESION;

            // Separation
            velocity += separation_force.normalize_or_zero() * FLOCK_SEPARATION;

            // Alignment
            velocity += (alignment_dir / neighbor_count as f32).normalize_or_zero() * FLOCK_ALIGNMENT;
        }

        // Chase player
        let to_player = (player_pos - my_pos).normalize_or_zero();
        velocity += to_player * FLOCK_PLAYER_CHASE;

        // Normalize and apply speed
        let final_dir = velocity.normalize_or_zero();
        let dt = time.delta_secs();
        transform.translation.x += final_dir.x * speed.0 * dt;
        transform.translation.y += final_dir.y * speed.0 * dt;
    }
}
