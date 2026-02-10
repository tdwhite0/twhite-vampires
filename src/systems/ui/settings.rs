use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use crate::components::*;
use crate::resources::*;
use crate::systems::startup;
use crate::systems::shader_materials::CrtEffect;

// === Settings Menu ===

pub const PANEL_BG: Color = Color::srgba(0.05, 0.05, 0.12, 0.95);
pub const SECTION_BG: Color = Color::srgba(0.1, 0.1, 0.2, 0.8);
pub const BTN_DEFAULT: Color = Color::srgb(0.15, 0.15, 0.28);
pub const BTN_HOVER: Color = Color::srgb(0.22, 0.22, 0.38);
pub const BTN_ACTIVE: Color = Color::srgb(0.12, 0.4, 0.15);
pub const BTN_ACTIVE_HOVER: Color = Color::srgb(0.15, 0.5, 0.2);
pub const ACCENT: Color = Color::srgb(0.3, 0.8, 1.0);
pub const MUTED: Color = Color::srgb(0.5, 0.5, 0.6);

pub const TAB_ACTIVE_BG: Color = Color::srgb(0.2, 0.2, 0.35);
pub const TAB_INACTIVE_BG: Color = Color::srgba(0.08, 0.08, 0.15, 0.6);

pub fn spawn_settings_gear(mut commands: Commands) {
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            right: Val::Px(16.0),
            width: Val::Px(36.0),
            height: Val::Px(36.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.85)),
        BorderColor::all(ACCENT),
        SettingsGearButton,
        SettingsButtonKind::Gear,
        GameEntity,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("*"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(ACCENT),
        ));
    });
}

pub fn toggle_settings_panel(
    mut commands: Commands,
    gear_query: Query<&Interaction, (Changed<Interaction>, With<SettingsGearButton>)>,
    panel_query: Query<Entity, With<SettingsPanel>>,
    debug: Res<DebugSettings>,
    weapons: Res<PlayerWeapons>,
    music_state: Res<MusicState>,
    font_state: Res<GameFontState>,
    mut paused: ResMut<GamePaused>,
) {
    for interaction in gear_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Toggle: if panel exists, despawn it; otherwise spawn it
        if let Some(entity) = panel_query.iter().next() {
            commands.entity(entity).despawn();
            paused.0 = false;
            return;
        }

        paused.0 = true;
        spawn_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
    }
}

pub fn rebuild_settings_panel(
    commands: &mut Commands,
    debug: &DebugSettings,
    weapons: &PlayerWeapons,
    music_state: &MusicState,
    font_state: &GameFontState,
) {
    spawn_settings_panel(commands, debug, weapons, music_state, font_state);
}

fn spawn_settings_panel(
    commands: &mut Commands,
    debug: &DebugSettings,
    weapons: &PlayerWeapons,
    music_state: &MusicState,
    font_state: &GameFontState,
) {
    let active_tab = debug.settings_tab;
    let invincible = debug.invincible;
    let dmg_mult = debug.damage_multiplier;
    let weapon_states: Vec<(WeaponKind, bool, u32)> = WeaponKind::all()
        .iter()
        .map(|k| (*k, weapons.has(*k), weapons.get(*k).map(|w| w.level).unwrap_or(0)))
        .collect();

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(60.0),
            right: Val::Px(16.0),
            width: Val::Px(300.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            row_gap: Val::Px(8.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(PANEL_BG),
        BorderColor::all(Color::srgba(0.3, 0.8, 1.0, 0.3)),
        SettingsPanel,
        GameEntity,
    )).with_children(|panel| {
        // Header
        panel.spawn((
            Text::new("SETTINGS"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(ACCENT),
            Node { margin: UiRect::bottom(Val::Px(2.0)), ..default() },
        ));

        // Tab bar
        panel.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(4.0),
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        }).with_children(|row| {
            for tab in SettingsTab::ALL {
                let is_active = tab == active_tab;
                row.spawn((
                    Button,
                    Node {
                        flex_grow: 1.0,
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::bottom(Val::Px(if is_active { 2.0 } else { 0.0 })),
                        ..default()
                    },
                    BackgroundColor(if is_active { TAB_ACTIVE_BG } else { TAB_INACTIVE_BG }),
                    BorderColor::all(if is_active { ACCENT } else { Color::NONE }),
                    SettingsTabButton(tab),
                    SettingsButtonKind::Tab(tab),
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new(tab.label()),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(if is_active { ACCENT } else { MUTED }),
                    ));
                });
            }
        });

        // Tab content
        match active_tab {
            SettingsTab::Cheats => {
                // === PLAYER section ===
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SECTION_BG),
                )).with_children(|section| {
                    section.spawn((
                        Text::new("PLAYER"),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(MUTED),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));
                    section.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(if invincible { BTN_ACTIVE } else { BTN_DEFAULT }),
                        SettingsInvincibleToggle,
                        SettingsButtonKind::Invincible,
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new("Invincible"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        btn.spawn((
                            Text::new(if invincible { "[ON]" } else { "[OFF]" }),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(if invincible { Color::srgb(0.3, 1.0, 0.4) } else { MUTED }),
                            SettingsInvincibleCheck,
                        ));
                    });
                });

                // === DAMAGE MULTIPLIER section ===
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SECTION_BG),
                )).with_children(|section| {
                    section.spawn((
                        Text::new("DAMAGE MULTIPLIER"),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(MUTED),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));
                    section.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        width: Val::Percent(100.0),
                        ..default()
                    }).with_children(|row| {
                        for mult in &[1.0f32, 1.5, 2.0, 2.5, 3.0] {
                            let is_active = (dmg_mult - mult).abs() < 0.01;
                            row.spawn((
                                Button,
                                Node {
                                    flex_grow: 1.0,
                                    height: Val::Px(32.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(if is_active { BTN_ACTIVE } else { BTN_DEFAULT }),
                                SettingsDamageButton(*mult),
                                SettingsButtonKind::Damage(*mult),
                            )).with_children(|btn| {
                                btn.spawn((
                                    Text::new(format!("{}x", mult)),
                                    TextFont { font_size: 13.0, ..default() },
                                    TextColor(if is_active { Color::srgb(0.3, 1.0, 0.4) } else { Color::WHITE }),
                                    SettingsDamageCheck(*mult),
                                ));
                            });
                        }
                    });
                });

                // === SPAWN BOSS section ===
                let spawn_mult = debug.spawn_rate_multiplier;
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SECTION_BG),
                )).with_children(|section| {
                    section.spawn((
                        Text::new("SPAWNING"),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(MUTED),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));

                    // Spawn rate slider
                    section.spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("Spawn Rate"),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        row.spawn((
                            SpawnRateText,
                            Text::new(format!("{:.1}x", spawn_mult)),
                            TextFont { font_size: 12.0, ..default() },
                            TextColor(ACCENT),
                        ));
                    });

                    section.spawn((
                        SpawnRateSliderTrack,
                        Button,
                        RelativeCursorPosition::default(),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(16.0),
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                    )).with_children(|track| {
                        // Map 0.0..5.0 to 0%..100%
                        let fill_pct = (spawn_mult / 5.0) * 100.0;
                        track.spawn((
                            SpawnRateSliderFill,
                            Node {
                                width: Val::Percent(fill_pct),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.8, 0.3, 0.1)),
                        ));
                    });

                    section.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                        SettingsSpawnBossButton,
                        SettingsButtonKind::SpawnBoss,
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new("Spawn Boss"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::srgb(1.0, 0.4, 0.4)),
                        ));
                    });
                });

                // === DEBUG VISUALS section ===
                let hitboxes_on = debug.show_boss_hitboxes;
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SECTION_BG),
                )).with_children(|section| {
                    section.spawn((
                        Text::new("DEBUG VISUALS"),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(MUTED),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));
                    section.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(if hitboxes_on { BTN_ACTIVE } else { BTN_DEFAULT }),
                        SettingsBossHitboxToggle,
                        SettingsButtonKind::BossHitbox,
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new("Boss Hitboxes"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        btn.spawn((
                            Text::new(if hitboxes_on { "[ON]" } else { "[OFF]" }),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(if hitboxes_on { Color::srgb(0.3, 1.0, 0.4) } else { MUTED }),
                            SettingsBossHitboxCheck,
                        ));
                    });
                });
            }
            SettingsTab::Weapons => {
                // === EQUIP WEAPONS section ===
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SECTION_BG),
                )).with_children(|section| {
                    section.spawn((
                        Text::new("EQUIP WEAPONS"),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(MUTED),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));
                    for (kind, owned, current_level) in &weapon_states {
                        // Weapon row container
                        section.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            margin: UiRect::bottom(Val::Px(3.0)),
                            ..default()
                        }).with_children(|weapon_col| {
                            // Equip/unequip button
                            weapon_col.spawn((
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(32.0),
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    ..default()
                                },
                                BackgroundColor(if *owned { BTN_ACTIVE } else { BTN_DEFAULT }),
                                SettingsWeaponButton(*kind),
                                SettingsButtonKind::Weapon(*kind),
                            )).with_children(|btn| {
                                btn.spawn((
                                    Text::new(kind.display_name()),
                                    TextFont { font_size: 13.0, ..default() },
                                    TextColor(kind.color()),
                                ));
                                btn.spawn((
                                    Text::new(if *owned { "Equipped" } else { "Equip" }),
                                    TextFont { font_size: 12.0, ..default() },
                                    TextColor(if *owned { Color::srgb(0.3, 1.0, 0.4) } else { MUTED }),
                                ));
                            });

                            // Level selector buttons (only if weapon is equipped)
                            if *owned {
                                weapon_col.spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(2.0),
                                    width: Val::Percent(100.0),
                                    padding: UiRect::new(Val::Px(10.0), Val::Px(10.0), Val::Px(2.0), Val::Px(0.0)),
                                    ..default()
                                }).with_children(|row| {
                                    row.spawn((
                                        Text::new("Lv"),
                                        TextFont { font_size: 10.0, ..default() },
                                        TextColor(MUTED),
                                        Node { align_self: AlignSelf::Center, margin: UiRect::right(Val::Px(4.0)), ..default() },
                                    ));
                                    for lvl in 1..=ActiveWeapon::MAX_LEVEL {
                                        let is_active = *current_level == lvl;
                                        row.spawn((
                                            Button,
                                            Node {
                                                flex_grow: 1.0,
                                                height: Val::Px(22.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            BackgroundColor(if is_active { BTN_ACTIVE } else { BTN_DEFAULT }),
                                            SettingsWeaponLevelButton(*kind, lvl),
                                            SettingsButtonKind::WeaponLevel(*kind, lvl),
                                        )).with_children(|btn| {
                                            btn.spawn((
                                                Text::new(format!("{}", lvl)),
                                                TextFont { font_size: 11.0, ..default() },
                                                TextColor(if is_active { Color::srgb(0.3, 1.0, 0.4) } else { Color::WHITE }),
                                            ));
                                        });
                                    }
                                });
                            }
                        });
                    }
                });
            }
            SettingsTab::AudioVideo => {
                // === DISPLAY section ===
                let crt_on = debug.crt_enabled;
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SECTION_BG),
                )).with_children(|section| {
                    section.spawn((
                        Text::new("DISPLAY"),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(MUTED),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));
                    section.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(if crt_on { BTN_ACTIVE } else { BTN_DEFAULT }),
                        SettingsCrtToggle,
                        SettingsButtonKind::Crt,
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new("CRT Filter"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        btn.spawn((
                            Text::new(if crt_on { "[ON]" } else { "[OFF]" }),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(if crt_on { Color::srgb(0.3, 1.0, 0.4) } else { MUTED }),
                            SettingsCrtCheck,
                        ));
                    });

                    // CRT parameter sliders (only when CRT is enabled)
                    if crt_on {
                        for param in CrtParam::ALL {
                            let value = match param {
                                CrtParam::Curvature => debug.crt_curvature,
                                CrtParam::ChromaticAberration => debug.crt_chromatic_aberration,
                                CrtParam::ScanlineIntensity => debug.crt_scanline_intensity,
                                CrtParam::PhosphorIntensity => debug.crt_phosphor_intensity,
                                CrtParam::VignetteStrength => debug.crt_vignette_strength,
                            };
                            let (min, max) = param.range();
                            let fill_pct = ((value - min) / (max - min)).clamp(0.0, 1.0) * 100.0;

                            section.spawn(Node {
                                width: Val::Percent(100.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(2.0)),
                                ..default()
                            }).with_children(|row| {
                                row.spawn((
                                    Text::new(param.label()),
                                    TextFont { font_size: 11.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                                row.spawn((
                                    CrtSliderText(param),
                                    Text::new(param.format_value(value)),
                                    TextFont { font_size: 11.0, ..default() },
                                    TextColor(ACCENT),
                                ));
                            });

                            section.spawn((
                                CrtSliderTrack(param),
                                Button,
                                RelativeCursorPosition::default(),
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(12.0),
                                    margin: UiRect::bottom(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                            )).with_children(|track| {
                                track.spawn((
                                    CrtSliderFill(param),
                                    Node {
                                        width: Val::Percent(fill_pct),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.5, 0.7)),
                                ));
                            });
                        }
                    }
                });

                // === FONT section ===
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SECTION_BG),
                )).with_children(|section| {
                    section.spawn((
                        Text::new("FONT"),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(MUTED),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));
                    section.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(36.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(BTN_DEFAULT),
                        SettingsFontCycleButton,
                        SettingsButtonKind::FontCycle,
                    )).with_children(|btn| {
                        btn.spawn((
                            Text::new("Game Font"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::WHITE),
                        ));
                        btn.spawn((
                            Text::new(format!("[{}]", font_state.current_name())),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(ACCENT),
                        ));
                    });
                });

                // === AUDIO section ===
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(SECTION_BG),
                )).with_children(|section| {
                    section.spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    }).with_children(|row| {
                        row.spawn((
                            Text::new("MUSIC"),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(MUTED),
                        ));
                        row.spawn((
                            MusicVolumeText,
                            Text::new(format!("{:.0}%", music_state.volume * 100.0)),
                            TextFont { font_size: 11.0, ..default() },
                            TextColor(ACCENT),
                        ));
                    });

                    section.spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(4.0),
                        width: Val::Percent(100.0),
                        flex_wrap: FlexWrap::Wrap,
                        row_gap: Val::Px(4.0),
                        ..default()
                    }).with_children(|row| {
                        for (i, track) in music_state.tracks.iter().enumerate() {
                            let is_selected = i == music_state.selected && !music_state.muted;
                            let border_color = if is_selected {
                                ACCENT
                            } else {
                                Color::srgba(0.3, 0.3, 0.3, 0.5)
                            };

                            row.spawn((
                                MusicTrackButton(i),
                                Button,
                                Node {
                                    flex_grow: 1.0,
                                    height: Val::Px(28.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(if is_selected { BTN_ACTIVE } else { BTN_DEFAULT }),
                                BorderColor::all(border_color),
                                SettingsButtonKind::MusicTrack(i),
                            )).with_children(|btn| {
                                btn.spawn((
                                    Text::new(track.name),
                                    TextFont { font_size: 11.0, ..default() },
                                    TextColor(if is_selected { ACCENT } else { Color::WHITE }),
                                ));
                            });
                        }

                        let off_selected = music_state.muted;
                        let off_border = if off_selected {
                            Color::srgb(1.0, 0.4, 0.4)
                        } else {
                            Color::srgba(0.3, 0.3, 0.3, 0.5)
                        };

                        row.spawn((
                            MusicMuteButton,
                            Button,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Px(28.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(if off_selected { Color::srgb(0.3, 0.1, 0.1) } else { BTN_DEFAULT }),
                            BorderColor::all(off_border),
                            SettingsButtonKind::MusicMute,
                        )).with_children(|btn| {
                            btn.spawn((
                                Text::new("Off"),
                                TextFont { font_size: 11.0, ..default() },
                                TextColor(if off_selected { Color::srgb(1.0, 0.4, 0.4) } else { Color::WHITE }),
                            ));
                        });
                    });

                    section.spawn((
                        MusicVolumeSliderTrack,
                        Button,
                        RelativeCursorPosition::default(),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(16.0),
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                    )).with_children(|track| {
                        track.spawn((
                            MusicVolumeSliderFill,
                            Node {
                                width: Val::Percent(music_state.volume * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.0, 0.6, 0.7)),
                        ));
                    });
                });
            }
        }
    });
}

pub fn handle_settings_invincible(
    query: Query<&Interaction, (Changed<Interaction>, With<SettingsInvincibleToggle>)>,
    mut debug: ResMut<DebugSettings>,
    mut check_query: Query<&mut TextColor, With<SettingsInvincibleCheck>>,
    mut text_query: Query<&mut Text, With<SettingsInvincibleCheck>>,
    mut bg_query: Query<&mut BackgroundColor, With<SettingsInvincibleToggle>>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        debug.invincible = !debug.invincible;

        for mut color in check_query.iter_mut() {
            *color = TextColor(if debug.invincible { Color::srgb(0.3, 1.0, 0.4) } else { MUTED });
        }
        for mut text in text_query.iter_mut() {
            **text = if debug.invincible { "[ON]".into() } else { "[OFF]".into() };
        }
        for mut bg in bg_query.iter_mut() {
            *bg = BackgroundColor(if debug.invincible { BTN_ACTIVE } else { BTN_DEFAULT });
        }
    }
}

pub fn handle_settings_crt(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<SettingsCrtToggle>)>,
    mut debug: ResMut<DebugSettings>,
    mut check_query: Query<&mut TextColor, With<SettingsCrtCheck>>,
    mut text_query: Query<&mut Text, With<SettingsCrtCheck>>,
    mut bg_query: Query<&mut BackgroundColor, With<SettingsCrtToggle>>,
    mut camera_query: Query<Entity, (With<Camera2d>, Without<HudCamera>)>,
    panel_query: Query<Entity, With<SettingsPanel>>,
    weapons: Res<PlayerWeapons>,
    music_state: Res<MusicState>,
    font_state: Res<GameFontState>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        debug.crt_enabled = !debug.crt_enabled;

        for mut color in check_query.iter_mut() {
            *color = TextColor(if debug.crt_enabled { Color::srgb(0.3, 1.0, 0.4) } else { MUTED });
        }
        for mut text in text_query.iter_mut() {
            **text = if debug.crt_enabled { "[ON]".into() } else { "[OFF]".into() };
        }
        for mut bg in bg_query.iter_mut() {
            *bg = BackgroundColor(if debug.crt_enabled { BTN_ACTIVE } else { BTN_DEFAULT });
        }

        // Insert or remove CrtEffect component on camera
        if let Ok(camera_entity) = camera_query.single_mut() {
            if debug.crt_enabled {
                commands.entity(camera_entity).insert(CrtEffect {
                    time: 0.0,
                    curvature: debug.crt_curvature,
                    chromatic_aberration: debug.crt_chromatic_aberration,
                    scanline_intensity: debug.crt_scanline_intensity,
                    phosphor_intensity: debug.crt_phosphor_intensity,
                    vignette_strength: debug.crt_vignette_strength,
                    _padding1: 0.0,
                    _padding2: 0.0,
                });
            } else {
                commands.entity(camera_entity).remove::<CrtEffect>();
            }
        }

        // Rebuild panel to show/hide CRT sliders
        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
        return;
    }
}

pub fn handle_settings_weapon_equip(
    mut commands: Commands,
    query: Query<(&Interaction, &SettingsWeaponButton), Changed<Interaction>>,
    mut weapons: ResMut<PlayerWeapons>,
    weapon_shaders: Res<WeaponShaderHandles>,
    mut notif_events: MessageWriter<NotificationEvent>,
    panel_query: Query<Entity, With<SettingsPanel>>,
    debug: Res<DebugSettings>,
    orbit_query: Query<Entity, With<OrbitShield>>,
    flame_query: Query<Entity, With<FlameAuraEntity>>,
    holy_water_query: Query<Entity, With<HolyWaterZone>>,
    music_state: Res<MusicState>,
    font_state: Res<GameFontState>,
) {
    for (interaction, btn) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let kind = btn.0;

        if weapons.has(kind) {
            // Unequip: remove weapon from inventory
            weapons.weapons.retain(|w| w.kind != kind);

            // Despawn associated entities
            if kind == WeaponKind::OrbitShield {
                for entity in orbit_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
            if kind == WeaponKind::FlameAura {
                for entity in flame_query.iter() {
                    commands.entity(entity).despawn();
                }
            }
            if kind == WeaponKind::HolyWater {
                for entity in holy_water_query.iter() {
                    commands.entity(entity).despawn();
                }
            }

            notif_events.write(NotificationEvent {
                message: format!("Debug: Unequipped {}", kind.display_name()),
                kind: NotificationType::Info,
            });
        } else {
            // Equip: add weapon to inventory
            weapons.weapons.push(ActiveWeapon::new(kind));
            if kind == WeaponKind::OrbitShield {
                startup::spawn_orbit_shield_ball(&mut commands, &weapon_shaders, 0);
            }

            notif_events.write(NotificationEvent {
                message: format!("Debug: Equipped {}", kind.display_name()),
                kind: NotificationType::Weapon,
            });
        }

        // Refresh the panel
        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
        return;
    }
}

pub fn handle_settings_weapon_level(
    mut commands: Commands,
    query: Query<(&Interaction, &SettingsWeaponLevelButton), Changed<Interaction>>,
    mut weapons: ResMut<PlayerWeapons>,
    mut notif_events: MessageWriter<NotificationEvent>,
    panel_query: Query<Entity, With<SettingsPanel>>,
    debug: Res<DebugSettings>,
    music_state: Res<MusicState>,
    font_state: Res<GameFontState>,
) {
    for (interaction, btn) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let kind = btn.0;
        let level = btn.1;

        if let Some(weapon) = weapons.get_mut(kind) {
            weapon.set_level(level);

            notif_events.write(NotificationEvent {
                message: format!("{} set to Lv{}", kind.display_name(), level),
                kind: NotificationType::Info,
            });
        }

        // Refresh the panel
        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
        return;
    }
}

pub fn handle_settings_damage_mult(
    mut commands: Commands,
    query: Query<(&Interaction, &SettingsDamageButton), Changed<Interaction>>,
    mut debug: ResMut<DebugSettings>,
    weapons: Res<PlayerWeapons>,
    panel_query: Query<Entity, With<SettingsPanel>>,
    music_state: Res<MusicState>,
    font_state: Res<GameFontState>,
) {
    for (interaction, btn) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        debug.damage_multiplier = btn.0;

        // Refresh the panel
        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
        return;
    }
}

pub fn settings_button_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &SettingsButtonKind),
        Changed<Interaction>,
    >,
    debug: Res<DebugSettings>,
    weapons: Res<PlayerWeapons>,
    music_state: Res<MusicState>,
) {
    for (interaction, mut bg, kind) in query.iter_mut() {
        match kind {
            SettingsButtonKind::Gear => {
                *bg = match interaction {
                    Interaction::Hovered => BackgroundColor(Color::srgba(0.15, 0.15, 0.3, 0.9)),
                    _ => BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.85)),
                };
            }
            SettingsButtonKind::Tab(tab) => {
                let is_active = *tab == debug.settings_tab;
                *bg = match interaction {
                    Interaction::Hovered => BackgroundColor(if is_active { TAB_ACTIVE_BG } else { BTN_HOVER }),
                    _ => BackgroundColor(if is_active { TAB_ACTIVE_BG } else { TAB_INACTIVE_BG }),
                };
            }
            SettingsButtonKind::SpawnBoss => {
                *bg = match interaction {
                    Interaction::Hovered => BackgroundColor(Color::srgb(0.4, 0.15, 0.15)),
                    _ => BackgroundColor(Color::srgb(0.3, 0.1, 0.1)),
                };
            }
            SettingsButtonKind::MusicMute => {
                let is_muted = music_state.muted;
                *bg = match interaction {
                    Interaction::Hovered => BackgroundColor(if is_muted { Color::srgb(0.4, 0.15, 0.15) } else { BTN_HOVER }),
                    _ => BackgroundColor(if is_muted { Color::srgb(0.3, 0.1, 0.1) } else { BTN_DEFAULT }),
                };
            }
            _ => {
                // Standard active/inactive pattern for all other button kinds
                let is_active = match kind {
                    SettingsButtonKind::Invincible => debug.invincible,
                    SettingsButtonKind::Weapon(wk) => weapons.has(*wk),
                    SettingsButtonKind::WeaponLevel(wk, lvl) => weapons.get(*wk).map(|w| w.level == *lvl).unwrap_or(false),
                    SettingsButtonKind::Damage(mult) => (debug.damage_multiplier - mult).abs() < 0.01,
                    SettingsButtonKind::MusicTrack(idx) => *idx == music_state.selected && !music_state.muted,
                    SettingsButtonKind::BossHitbox => debug.show_boss_hitboxes,
                    SettingsButtonKind::Crt => debug.crt_enabled,
                    SettingsButtonKind::FontCycle => false,
                    // Already handled above
                    SettingsButtonKind::Gear | SettingsButtonKind::Tab(_) | SettingsButtonKind::SpawnBoss | SettingsButtonKind::MusicMute => false,
                };
                *bg = match interaction {
                    Interaction::Hovered => BackgroundColor(if is_active { BTN_ACTIVE_HOVER } else { BTN_HOVER }),
                    _ => BackgroundColor(if is_active { BTN_ACTIVE } else { BTN_DEFAULT }),
                };
            }
        }
    }
}

pub fn handle_settings_tab_switch(
    mut commands: Commands,
    query: Query<(&Interaction, &SettingsTabButton), Changed<Interaction>>,
    mut debug: ResMut<DebugSettings>,
    weapons: Res<PlayerWeapons>,
    music_state: Res<MusicState>,
    font_state: Res<GameFontState>,
    panel_query: Query<Entity, With<SettingsPanel>>,
) {
    for (interaction, btn) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if debug.settings_tab == btn.0 {
            continue;
        }
        debug.settings_tab = btn.0;

        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
        return;
    }
}

pub fn handle_settings_spawn_boss(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<SettingsSpawnBossButton>)>,
    mut notif_events: MessageWriter<NotificationEvent>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        // Insert a resource to signal the boss system to spawn a boss
        commands.insert_resource(crate::resources::ForceSpawnBoss { kind: None });
        notif_events.write(NotificationEvent {
            message: "Debug: Spawning Boss!".into(),
            kind: NotificationType::Info,
        });
    }
}

pub fn handle_settings_boss_hitboxes(
    query: Query<&Interaction, (Changed<Interaction>, With<SettingsBossHitboxToggle>)>,
    mut debug: ResMut<DebugSettings>,
    mut check_query: Query<&mut TextColor, With<SettingsBossHitboxCheck>>,
    mut text_query: Query<&mut Text, With<SettingsBossHitboxCheck>>,
    mut bg_query: Query<&mut BackgroundColor, With<SettingsBossHitboxToggle>>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        debug.show_boss_hitboxes = !debug.show_boss_hitboxes;

        for mut color in check_query.iter_mut() {
            *color = TextColor(if debug.show_boss_hitboxes { Color::srgb(0.3, 1.0, 0.4) } else { MUTED });
        }
        for mut text in text_query.iter_mut() {
            **text = if debug.show_boss_hitboxes { "[ON]".into() } else { "[OFF]".into() };
        }
        for mut bg in bg_query.iter_mut() {
            *bg = BackgroundColor(if debug.show_boss_hitboxes { BTN_ACTIVE } else { BTN_DEFAULT });
        }
    }
}

pub fn handle_settings_font_cycle(
    mut commands: Commands,
    query: Query<&Interaction, (Changed<Interaction>, With<SettingsFontCycleButton>)>,
    mut font_state: ResMut<GameFontState>,
    panel_query: Query<Entity, With<SettingsPanel>>,
    debug: Res<DebugSettings>,
    weapons: Res<PlayerWeapons>,
    music_state: Res<MusicState>,
) {
    for interaction in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        font_state.cycle();

        // Rebuild settings panel to show new font name
        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        spawn_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
        return;
    }
}

pub fn apply_font_to_all_text(
    font_state: Res<GameFontState>,
    mut query: Query<&mut TextFont>,
) {
    if !font_state.is_changed() {
        return;
    }
    let font = font_state.current();
    for mut text_font in query.iter_mut() {
        text_font.font = font.clone();
    }
}

pub fn handle_spawn_rate_slider(
    mut debug: ResMut<DebugSettings>,
    track_query: Query<
        (&Interaction, &RelativeCursorPosition),
        With<SpawnRateSliderTrack>,
    >,
    mut fill_query: Query<&mut Node, With<SpawnRateSliderFill>>,
    mut text_query: Query<&mut Text, With<SpawnRateText>>,
) {
    let Ok((interaction, relative_cursor)) = track_query.single() else {
        return;
    };

    if *interaction != Interaction::Pressed {
        return;
    }

    let Some(pos) = relative_cursor.normalized else {
        return;
    };

    // Map 0..1 slider position to 0.0..5.0 multiplier, snap to nearest 0.1
    let raw = (pos.x + 0.5).clamp(0.0, 1.0) * 5.0;
    let value = (raw * 10.0).round() / 10.0;
    debug.spawn_rate_multiplier = value;

    if let Ok(mut fill_node) = fill_query.single_mut() {
        fill_node.width = Val::Percent((value / 5.0) * 100.0);
    }

    if let Ok(mut text) = text_query.single_mut() {
        **text = format!("{:.1}x", value);
    }
}

pub fn handle_crt_sliders(
    mut debug: ResMut<DebugSettings>,
    track_query: Query<
        (&Interaction, &RelativeCursorPosition, &CrtSliderTrack),
        With<CrtSliderTrack>,
    >,
    mut fill_query: Query<(&mut Node, &CrtSliderFill)>,
    mut text_query: Query<(&mut Text, &CrtSliderText)>,
    mut crt_query: Query<&mut CrtEffect>,
) {
    for (interaction, relative_cursor, track) in track_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(pos) = relative_cursor.normalized else {
            continue;
        };

        let param = track.0;
        let (min, max) = param.range();
        let t = (pos.x + 0.5).clamp(0.0, 1.0);
        let value = min + t * (max - min);

        match param {
            CrtParam::Curvature => debug.crt_curvature = value,
            CrtParam::ChromaticAberration => debug.crt_chromatic_aberration = value,
            CrtParam::ScanlineIntensity => debug.crt_scanline_intensity = value,
            CrtParam::PhosphorIntensity => debug.crt_phosphor_intensity = value,
            CrtParam::VignetteStrength => debug.crt_vignette_strength = value,
        }

        // Update fill bar
        for (mut node, fill) in fill_query.iter_mut() {
            if fill.0 == param {
                node.width = Val::Percent(t * 100.0);
            }
        }

        // Update text
        for (mut text, text_param) in text_query.iter_mut() {
            if text_param.0 == param {
                **text = param.format_value(value);
            }
        }

        // Update CrtEffect component on camera
        for mut crt in crt_query.iter_mut() {
            match param {
                CrtParam::Curvature => crt.curvature = value,
                CrtParam::ChromaticAberration => crt.chromatic_aberration = value,
                CrtParam::ScanlineIntensity => crt.scanline_intensity = value,
                CrtParam::PhosphorIntensity => crt.phosphor_intensity = value,
                CrtParam::VignetteStrength => crt.vignette_strength = value,
            }
        }
    }
}

pub fn despawn_settings_ui(
    mut commands: Commands,
    panel_query: Query<Entity, With<SettingsPanel>>,
    gear_query: Query<Entity, With<SettingsGearButton>>,
    mut paused: ResMut<GamePaused>,
) {
    paused.0 = false;
    for entity in panel_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in gear_query.iter() {
        commands.entity(entity).despawn();
    }
}
