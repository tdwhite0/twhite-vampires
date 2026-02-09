use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;

const MAX_ENEMIES: u32 = 200;
const SPAWN_DISTANCE: f32 = 600.0;

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

        spawn_enemy(&mut commands, &sprites, kind, spawn_pos);
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
            spawn_enemy(&mut commands, &sprites, kind, spawn_pos);
            wave.enemies_alive += 1;
        }
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
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    sprites: &SpriteAssets,
    kind: EnemyKind,
    position: Vec2,
) {
    let (health, speed, damage, z) = match kind {
        EnemyKind::Basic => (10.0, 80.0, 10.0, 5.0),
        EnemyKind::Fast => (5.0, 150.0, 8.0, 5.0),
        EnemyKind::Tank => (30.0, 50.0, 20.0, 4.0),
        EnemyKind::Swarm => (3.0, 100.0, 5.0, 5.0),
    };

    let mut rng = rand::thread_rng();
    let (sprite_index, scale) = pick_sprite_index(kind, &mut rng);

    commands.spawn((
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
}

pub fn enemy_movement(
    mut enemy_query: Query<(&mut Transform, &EnemySpeed), With<Enemy>>,
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
