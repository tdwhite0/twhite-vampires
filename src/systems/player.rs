use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

const PET_FOLLOW_SPEED: f32 = 280.0;
const PET_FOLLOW_DISTANCE: f32 = 40.0;

pub fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MoveSpeed), With<Player>>,
) {
    let Ok((mut transform, speed)) = query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        transform.translation.x += direction.x * speed.0 * time.delta_secs();
        transform.translation.y += direction.y * speed.0 * time.delta_secs();
    }
}

pub fn player_aim_at_mouse(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<HudCamera>)>,
    mut player_query: Query<
        (&Transform, &mut Sprite, &mut AnimationIndices),
        With<Player>,
    >,
) {
    let Ok(window) = window_query.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_global)) = camera_query.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_global, cursor_pos) else {
        return;
    };
    let Ok((player_transform, mut sprite, mut anim_indices)) = player_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let to_mouse = world_pos - player_pos;

    // Hero sprite sheet: 4 columns x 3 rows (40x64 per frame)
    // Row 0 (indices 0-3): facing down (toward camera)
    // Row 1 (indices 4-7): facing right (side view)
    // Row 2 (indices 8-11): facing up (away from camera)
    let row = if to_mouse.y < -to_mouse.x.abs() * 0.5 {
        0 // Facing down
    } else if to_mouse.y > to_mouse.x.abs() * 0.5 {
        2 // Facing up
    } else {
        1 // Facing side
    };

    // Update animation range when the row changes so animate_sprites
    // cycles the correct frames instead of always looping row 0
    let row_start = row * 4;
    let row_end = row_start + 3;
    if anim_indices.first != row_start {
        anim_indices.first = row_start;
        anim_indices.last = row_end;
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = row_start;
        }
    }

    // Native sprite faces right — flip when mouse is to the left
    if to_mouse.x < 0.0 {
        sprite.flip_x = true;
    } else if to_mouse.x > 0.0 {
        sprite.flip_x = false;
    }
}

pub fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>, Without<HudCamera>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    let target = player_transform.translation;
    let smooth_speed = 5.0;
    camera_transform.translation = camera_transform
        .translation
        .lerp(target, smooth_speed * time.delta_secs());
    camera_transform.translation.z = 999.0;
}

pub fn update_damage_flash(
    mut query: Query<(&mut DamageCooldown, &mut Sprite), With<Player>>,
    time: Res<Time>,
) {
    let Ok((mut cooldown, mut sprite)) = query.single_mut() else {
        return;
    };

    if cooldown.0 > 0.0 {
        cooldown.0 -= time.delta_secs();
        // Flash by toggling color tint
        let flash = (cooldown.0 * 20.0).sin() > 0.0;
        sprite.color = if flash {
            Color::srgb(1.0, 0.3, 0.3)
        } else {
            Color::WHITE
        };
    } else {
        sprite.color = Color::WHITE;
    }
}

pub fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                // Simple cycle within the animation range
                // Hero direction-switching is handled by player_movement
                atlas.index = if atlas.index >= indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

const DASH_SPEED: f32 = 800.0;
const DASH_DURATION: f32 = 0.15;

pub fn dash_ability_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut abilities: ResMut<PlayerAbilities>,
    time: Res<Time>,
    query: Query<(Entity, &Transform), (With<Player>, Without<Dashing>)>,
    mut flash_query: Query<&mut AbilityCooldownFlash>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<HudCamera>)>,
) {
    // Tick all ability cooldowns
    for ability in abilities.abilities.iter_mut() {
        ability.tick(time.delta_secs());
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(dash) = abilities.get_mut(AbilityKind::Dash) else {
        return;
    };
    if !dash.is_ready() {
        // Flash the UI slot red
        let dash_index = abilities.index_of(AbilityKind::Dash).unwrap_or(0);
        for mut flash in flash_query.iter_mut() {
            if flash.index == dash_index {
                flash.remaining = 0.3;
            }
        }
        return;
    }

    let Ok((entity, player_transform)) = query.single() else {
        return;
    };

    // Dash toward the mouse cursor
    let dir = (|| -> Option<Vec2> {
        let window = window_query.single().ok()?;
        let cursor_pos = window.cursor_position()?;
        let (camera, camera_global) = camera_query.single().ok()?;
        let world_pos = camera.viewport_to_world_2d(camera_global, cursor_pos).ok()?;
        let to_mouse = world_pos - player_transform.translation.truncate();
        if to_mouse.length_squared() > 0.0 {
            Some(to_mouse.normalize())
        } else {
            None
        }
    })()
    .unwrap_or(Vec2::X);

    dash.trigger();
    commands.entity(entity).insert(Dashing {
        direction: dir,
        remaining: DASH_DURATION,
        speed: DASH_SPEED,
        damage: 30.0,
        hit_enemies: Vec::new(),
    });
}

pub fn update_dash(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Dashing), With<Player>>,
    time: Res<Time>,
) {
    let Ok((entity, mut transform, mut dashing)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    transform.translation.x += dashing.direction.x * dashing.speed * dt;
    transform.translation.y += dashing.direction.y * dashing.speed * dt;

    dashing.remaining -= dt;
    if dashing.remaining <= 0.0 {
        commands.entity(entity).remove::<Dashing>();
    }
}

pub fn pet_follow_player(
    player_query: Query<&Transform, (With<Player>, Without<PetDog>)>,
    mut pet_query: Query<(&mut Transform, &mut Sprite), (With<PetDog>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let Ok((mut pet_transform, mut pet_sprite)) = pet_query.single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let pet_pos = pet_transform.translation.truncate();
    let to_player = player_pos - pet_pos;
    let dist = to_player.length();

    if dist > PET_FOLLOW_DISTANCE {
        let direction = to_player.normalize_or_zero();
        let move_speed = PET_FOLLOW_SPEED * time.delta_secs();
        let move_amount = (dist - PET_FOLLOW_DISTANCE).min(move_speed);
        pet_transform.translation.x += direction.x * move_amount;
        pet_transform.translation.y += direction.y * move_amount;

        // Flip sprite based on movement direction
        if direction.x < -0.1 {
            pet_sprite.flip_x = false;
        } else if direction.x > 0.1 {
            pet_sprite.flip_x = true;
        }
    }
}
