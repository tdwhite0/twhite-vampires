use bevy::prelude::*;
use rand::Rng;
use crate::components::*;
use crate::resources::*;
use crate::systems::startup;

const CHAR_BOX_SIZE: f32 = 140.0;
const CHAR_BOX_HEIGHT: f32 = 170.0;

// === Title Screen ===
pub fn spawn_title_screen(
    mut commands: Commands,
    selected: Res<SelectedCharacter>,
    sprites: Option<Res<SpriteAssets>>,
    existing: Query<Entity, With<TitleScreenEntity>>,
) {
    // SpriteAssets may not exist yet on first OnEnter (commands from Startup
    // haven't flushed). This system also runs each frame in Update so it
    // will create the screen as soon as assets are ready.
    let Some(sprites) = sprites else {
        return;
    };
    // Don't respawn if title screen already exists
    if !existing.is_empty() {
        return;
    }
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.05, 0.95)),
            TitleScreenEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SURVIVOR ARENA"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 1.0)),
            ));
            parent.spawn((
                Text::new("A Vampire Survivors-like"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.6)),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));

            // "Choose your character" label
            parent.spawn((
                Text::new("Choose your character"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Character selection row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                })
                .with_children(|row| {
                    for (i, kind) in CharacterKind::ALL.iter().enumerate() {
                        let is_selected = *kind == selected.0;
                        let border_color = if is_selected {
                            kind.border_color()
                        } else {
                            Color::srgb(0.3, 0.3, 0.4)
                        };
                        let bg_alpha = if is_selected { 0.3 } else { 0.1 };

                        let (texture, layout) = kind.sprite_info(&sprites);
                        let preview_index = kind.preview_index();

                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(CHAR_BOX_SIZE),
                                height: Val::Px(CHAR_BOX_HEIGHT),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(3.0)),
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            BorderColor::all(border_color),
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.2, bg_alpha)),
                            CharacterSelectBox(i),
                        ))
                        .with_children(|card| {
                            // Character sprite preview
                            card.spawn((
                                ImageNode {
                                    image: texture,
                                    texture_atlas: Some(TextureAtlas {
                                        layout,
                                        index: preview_index,
                                    }),
                                    ..default()
                                },
                                Node {
                                    width: Val::Px(64.0),
                                    height: Val::Px(64.0),
                                    margin: UiRect::bottom(Val::Px(8.0)),
                                    ..default()
                                },
                            ));

                            // Character name
                            card.spawn((
                                Text::new(kind.display_name()),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(if is_selected {
                                    Color::WHITE
                                } else {
                                    Color::srgb(0.6, 0.6, 0.7)
                                }),
                            ));

                            // Character description
                            card.spawn((
                                Text::new(kind.description()),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.5, 0.5, 0.6)),
                                Node {
                                    margin: UiRect::top(Val::Px(4.0)),
                                    ..default()
                                },
                            ));
                        });
                    }
                });

            // Navigation hint
            parent.spawn((
                Text::new("A/D or Click to select"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.6)),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));

            // Start prompt
            parent.spawn((
                Text::new("Press SPACE to Start"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.3)),
                Node {
                    margin: UiRect::top(Val::Px(24.0)),
                    ..default()
                },
            ));
        });
}

pub fn despawn_title_screen(
    mut commands: Commands,
    query: Query<Entity, With<TitleScreenEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn handle_title_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut needs_init: ResMut<NeedsGameInit>,
    sound_assets: Res<SoundAssets>,
    mut selected: ResMut<SelectedCharacter>,
    mut box_query: Query<(&CharacterSelectBox, &mut BorderColor, &mut BackgroundColor, &Interaction, &Children)>,
    mut text_query: Query<&mut TextColor>,
) {
    let all = CharacterKind::ALL;
    let current_idx = all.iter().position(|k| *k == selected.0).unwrap_or(0);

    // Keyboard cycling
    let mut new_idx = None;
    if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) {
        new_idx = Some((current_idx + 1) % all.len());
    }
    if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) {
        new_idx = Some((current_idx + all.len() - 1) % all.len());
    }

    // Mouse click selection
    for (select_box, _, _, interaction, _) in box_query.iter() {
        if *interaction == Interaction::Pressed {
            new_idx = Some(select_box.0);
        }
    }

    if let Some(idx) = new_idx {
        selected.0 = all[idx];

        // Update visual state of all boxes
        for (select_box, mut border, mut bg, _, children) in box_query.iter_mut() {
            let kind = all[select_box.0];
            let is_sel = select_box.0 == idx;
            *border = BorderColor::all(if is_sel {
                kind.border_color()
            } else {
                Color::srgb(0.3, 0.3, 0.4)
            });
            *bg = BackgroundColor(Color::srgba(0.1, 0.1, 0.2, if is_sel { 0.3 } else { 0.1 }));

            // Update name text color (first text child with font_size >= 18)
            for child in children.iter() {
                if let Ok(mut tc) = text_query.get_mut(child) {
                    // Update all text children - name gets bright, desc stays dim
                    tc.0 = if is_sel {
                        Color::WHITE
                    } else {
                        Color::srgb(0.6, 0.6, 0.7)
                    };
                }
            }
        }
    }

    // Start game
    if keys.just_pressed(KeyCode::Space) {
        needs_init.0 = true;
        next_state.set(GameState::Playing);
        commands.spawn((
            AudioPlayer::<AudioSource>::new(sound_assets.game_start.clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.25)),
        ));
    }
}

// === Game Over Screen ===
pub fn spawn_game_over_screen(
    mut commands: Commands,
    stats: Res<GameStats>,
    player_query: Query<&Experience, With<Player>>,
    weapons: Res<PlayerWeapons>,
) {
    let level = player_query.single().map(|x| x.level).unwrap_or(1);
    let secs = stats.time_survived as u32;
    let mins = secs / 60;
    let remaining_secs = secs % 60;

    let stat_rows: Vec<(&str, String)> = vec![
        ("Time Survived", format!("{}:{:02}", mins, remaining_secs)),
        ("Enemies Slain", format!("{}", stats.enemies_killed)),
        ("Bosses Defeated", format!("{}", stats.bosses_killed)),
        ("XP Collected", format!("{}", stats.xp_collected as u32)),
        ("Damage Taken", format!("{}", stats.damage_taken as u32)),
        ("Heals Collected", format!("{}", stats.heals_collected)),
        ("Level Reached", format!("{}", level)),
    ];

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.0, 0.0, 0.95)),
            GameOverEntity,
            GameOverAnimTimer { elapsed: 0.0 },
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.15, 0.15, 0.0)),
                GameOverStatRow(0),
            ));

            // Separator spacer
            parent.spawn(Node {
                height: Val::Px(24.0),
                ..default()
            });

            // Stats panel
            for (i, (label, value)) in stat_rows.iter().enumerate() {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(420.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Row,
                            margin: UiRect::vertical(Val::Px(4.0)),
                            padding: UiRect::horizontal(Val::Px(12.0)),
                            ..default()
                        },
                        GameOverStatRow((i + 1) as u32),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(*label),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.6, 0.6, 0.7, 0.0)),
                        ));
                        row.spawn((
                            Text::new(value.clone()),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                        ));
                    });
            }

            // Per-weapon damage rows
            if !weapons.weapons.is_empty() {
                // Sort weapons by damage dealt (highest first)
                let mut weapon_damages: Vec<_> = weapons.weapons.iter().map(|w| {
                    let dmg = stats.weapon_damage.get(&w.kind).copied().unwrap_or(0.0);
                    (w.kind, w.level, dmg)
                }).collect();
                weapon_damages.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

                let base_row = stat_rows.len() as u32 + 1;
                for (i, (kind, level, dmg)) in weapon_damages.iter().enumerate() {
                    let color = kind.color().with_alpha(0.0);
                    parent
                        .spawn((
                            Node {
                                width: Val::Px(420.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Row,
                                margin: UiRect::vertical(Val::Px(3.0)),
                                padding: UiRect::horizontal(Val::Px(12.0)),
                                ..default()
                            },
                            GameOverStatRow(base_row + i as u32),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(format!("{} Lv.{}", kind.display_name(), level)),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(color),
                            ));
                            row.spawn((
                                Text::new(format!("{}", *dmg as u32)),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0)),
                            ));
                        });
                }
            }

            // Restart prompt
            parent.spawn((
                Text::new("Press SPACE to Continue"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 0.3, 0.0)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
                GameOverRestartPrompt,
            ));
        });
}

pub fn animate_game_over_screen(
    time: Res<Time>,
    mut timer_query: Query<&mut GameOverAnimTimer>,
    stat_rows_with_children: Query<(Entity, &GameOverStatRow, Option<&Children>)>,
    mut texts: Query<&mut TextColor>,
    prompt_query: Query<Entity, With<GameOverRestartPrompt>>,
) {
    let Ok(mut timer) = timer_query.single_mut() else {
        return;
    };
    timer.elapsed += time.delta_secs();
    let elapsed = timer.elapsed;

    for (entity, stat_row, children) in stat_rows_with_children.iter() {
        let index = stat_row.0;
        let alpha = if index == 0 {
            // Title: fade in from 0.0 to 0.6
            (elapsed / 0.6).clamp(0.0, 1.0)
        } else {
            // Stat rows 1-8: stagger in
            let start_time = 0.6 + (index - 1) as f32 * 0.15;
            ((elapsed - start_time) / 0.3).clamp(0.0, 1.0)
        };

        // Apply alpha to the entity itself (for title text which has TextColor directly)
        if let Ok(mut text_color) = texts.get_mut(entity) {
            text_color.0 = text_color.0.with_alpha(alpha);
        }

        // Apply alpha to all text children in this row
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut text_color) = texts.get_mut(child) {
                    text_color.0 = text_color.0.with_alpha(alpha);
                }
            }
        }
    }

    // Restart prompt: visible after 2.8s, pulsing
    if elapsed > 2.8 {
        let pulse_alpha = (elapsed * 2.5).sin() * 0.3 + 0.7;
        if let Ok(entity) = prompt_query.single() {
            if let Ok(mut text_color) = texts.get_mut(entity) {
                text_color.0 = text_color.0.with_alpha(pulse_alpha);
            }
        }
    }
}

pub fn handle_game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Title);
    }
}

// === Level Up Screen ===
pub fn spawn_level_up_screen(
    mut commands: Commands,
    weapons: Res<PlayerWeapons>,
    player_query: Query<&Experience, With<Player>>,
    pickup_range: Res<PickupRange>,
) {
    let level = player_query.single().map(|x| x.level).unwrap_or(1);
    let upgrades = generate_upgrades(&weapons, level, &pickup_range);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.1, 0.85)),
            LevelUpEntity,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("LEVEL UP! (Lv. {})", level)),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Choose an upgrade:"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Upgrade buttons
            for upgrade in &upgrades {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(350.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::all(Val::Px(6.0)),
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
                        UpgradeButton(*upgrade),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(upgrade.name()),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 1.0)),
                        ));
                        btn.spawn((
                            Text::new(upgrade.description()),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.6)),
                        ));
                    });
            }
        });
}

fn generate_upgrades(weapons: &PlayerWeapons, _level: u32, pickup_range: &PickupRange) -> Vec<UpgradeKind> {
    let mut rng = rand::thread_rng();
    let mut options: Vec<UpgradeKind> = Vec::new();

    // Collect possible upgrades
    let mut pool: Vec<UpgradeKind> = Vec::new();

    // New weapons the player doesn't have
    for kind in WeaponKind::all() {
        if !weapons.has(*kind) {
            pool.push(UpgradeKind::NewWeapon(*kind));
        }
    }

    // Level up existing weapons (if below max level)
    for weapon in &weapons.weapons {
        if weapon.level < ActiveWeapon::MAX_LEVEL {
            pool.push(UpgradeKind::LevelUpWeapon(weapon.kind));
        }
    }

    // Player upgrades
    pool.push(UpgradeKind::IncreaseMaxHealth);
    pool.push(UpgradeKind::IncreaseMoveSpeed);
    pool.push(UpgradeKind::HealPlayer);
    if pickup_range.level < PickupRange::MAX_LEVEL {
        pool.push(UpgradeKind::IncreasePickupRange);
    }

    // Pick 3 random unique upgrades
    while options.len() < 3 && !pool.is_empty() {
        let idx = rng.gen_range(0..pool.len());
        options.push(pool.remove(idx));
    }

    options
}

pub fn handle_level_up_selection(
    mut commands: Commands,
    query: Query<(&Interaction, &UpgradeButton), Changed<Interaction>>,
    mut weapons: ResMut<PlayerWeapons>,
    mut player_query: Query<(&mut Health, &mut MoveSpeed), With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
    weapon_shaders: Res<WeaponShaderHandles>,
    orbit_query: Query<Entity, With<OrbitShield>>,
    sound_assets: Res<SoundAssets>,
    mut notif_events: MessageWriter<NotificationEvent>,
    mut pickup_range: ResMut<PickupRange>,
) {
    for (interaction, upgrade_btn) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let upgrade = upgrade_btn.0;

        // Send notification for the chosen upgrade
        let (msg, kind) = match &upgrade {
            UpgradeKind::NewWeapon(wk) => (
                format!("Gained Weapon: {}", wk.display_name()),
                NotificationType::Weapon,
            ),
            UpgradeKind::IncreaseMaxHealth | UpgradeKind::IncreaseMoveSpeed | UpgradeKind::HealPlayer | UpgradeKind::IncreasePickupRange => (
                upgrade.name(),
                NotificationType::Buff,
            ),
            UpgradeKind::LevelUpWeapon(wk) => (
                format!("{} Level Up!", wk.display_name()),
                NotificationType::Weapon,
            ),
        };
        notif_events.write(NotificationEvent { message: msg, kind });

        apply_upgrade(
            &mut commands,
            &upgrade,
            &mut weapons,
            &mut player_query,
            &weapon_shaders,
            &orbit_query,
            &mut pickup_range,
        );
        commands.spawn((
            AudioPlayer::<AudioSource>::new(sound_assets.upgrade_selected.clone()),
            PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(0.20)),
        ));
        next_state.set(GameState::Playing);
        return;
    }
}

fn apply_upgrade(
    commands: &mut Commands,
    upgrade: &UpgradeKind,
    weapons: &mut PlayerWeapons,
    player_query: &mut Query<(&mut Health, &mut MoveSpeed), With<Player>>,
    weapon_shaders: &WeaponShaderHandles,
    orbit_query: &Query<Entity, With<OrbitShield>>,
    pickup_range: &mut PickupRange,
) {
    match upgrade {
        UpgradeKind::NewWeapon(kind) => {
            weapons.weapons.push(ActiveWeapon::new(*kind));
            if *kind == WeaponKind::OrbitShield {
                startup::spawn_orbit_shield_ball(
                    commands,
                    weapon_shaders,
                    0,
                );
            }
        }
        UpgradeKind::LevelUpWeapon(kind) => {
            if let Some(w) = weapons.get_mut(*kind) {
                if w.level < ActiveWeapon::MAX_LEVEL {
                    w.set_level(w.level + 1);
                    if *kind == WeaponKind::OrbitShield {
                        startup::sync_orbit_shield_entities(commands, weapon_shaders, orbit_query, w.count);
                    }
                }
            }
        }
        UpgradeKind::IncreaseMaxHealth => {
            if let Ok((mut health, _)) = player_query.single_mut() {
                health.max += 20.0;
                health.current += 20.0;
            }
        }
        UpgradeKind::IncreaseMoveSpeed => {
            if let Ok((_, mut speed)) = player_query.single_mut() {
                speed.0 *= 1.15;
            }
        }
        UpgradeKind::HealPlayer => {
            if let Ok((mut health, _)) = player_query.single_mut() {
                health.current = (health.current + health.max * 0.3).min(health.max);
            }
        }
        UpgradeKind::IncreasePickupRange => {
            pickup_range.level += 1;
            pickup_range.multiplier += 0.25;
        }
    }
}

pub fn despawn_level_up_screen(
    mut commands: Commands,
    query: Query<Entity, With<LevelUpEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

// === Button hover effects ===
pub fn button_hover_visuals(
    mut query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<UpgradeButton>)>,
) {
    for (interaction, mut bg) in query.iter_mut() {
        *bg = match interaction {
            Interaction::Pressed => BackgroundColor(Color::srgb(0.3, 0.3, 0.5)),
            Interaction::Hovered => BackgroundColor(Color::srgb(0.2, 0.2, 0.35)),
            Interaction::None => BackgroundColor(Color::srgb(0.15, 0.15, 0.25)),
        };
    }
}
