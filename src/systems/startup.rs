use bevy::prelude::*;
use bevy::image::TextureAtlasLayout;
use bevy::camera::visibility::RenderLayers;
use crate::components::*;
use crate::resources::*;
use crate::systems::shader_materials::*;

// Constants
pub const PLAYER_RADIUS: f32 = 15.0;
pub const PLAYER_SPEED: f32 = 200.0;
pub const PLAYER_MAX_HEALTH: f32 = 100.0;

pub fn setup_camera(mut commands: Commands) {
    // Main game camera (layer 0, CRT post-process)
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.02, 0.06)),
            ..default()
        },
        CrtEffect {
            time: 0.0,
            curvature: 0.15,
            chromatic_aberration: 0.003,
            scanline_intensity: 0.25,
            phosphor_intensity: 0.25,
            vignette_strength: 1.8,
        },
    ));

    // HUD camera (layer 1, renders after CRT, no post-processing)
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(1),
        HudCamera,
    ));
}

pub fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut orbit_mats: ResMut<Assets<OrbitShieldMaterial>>,
    mut proj_mats: ResMut<Assets<ProjectileMaterial>>,
    mut lightning_mats: ResMut<Assets<LightningMaterial>>,
    mut flame_mats: ResMut<Assets<FlameAuraMaterial>>,
    mut boom_mats: ResMut<Assets<BoomerangMaterial>>,
    mut holy_water_mats: ResMut<Assets<HolyWaterMaterial>>,
    mut updown_mats: ResMut<Assets<UpDownMaterial>>,
    mut phiera_mats: ResMut<Assets<PhieraMaterial>>,
    mut laser_mats: ResMut<Assets<BossLaserMaterial>>,
    mut health_bar_mats: ResMut<Assets<HealthBarMaterial>>,
    mut xp_bar_mats: ResMut<Assets<XpBarMaterial>>,
) {
    // Load sprite sheets
    let hero_texture: Handle<Image> = asset_server.load("textures/hero.png");
    // Hero: 160x192, 4 cols x 3 rows, each frame 40x64
    let hero_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(40, 64),
        4,
        3,
        None,
        None,
    ));

    let creatures_texture: Handle<Image> = asset_server.load("textures/creatures.png");
    // Creatures: 160x288, 10 cols x 18 rows, each frame 16x16
    let creatures_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        10,
        18,
        None,
        None,
    ));

    let dog_texture: Handle<Image> = asset_server.load("textures/dog.png");
    // Dog: 96x80, 6 cols x 5 rows, each frame 16x16
    let dog_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        6,
        5,
        None,
        None,
    ));

    commands.insert_resource(SpriteAssets {
        hero_texture,
        hero_layout,
        creatures_texture,
        creatures_layout,
        dog_texture,
        dog_layout,
    });

    // Load boss sprites - eyeball boss (320x384, 2 cols x 3 rows, 160x128 per frame)
    let boss_eye_texture_1: Handle<Image> = asset_server.load("textures/boss_eye_1.png");
    let boss_eye_texture_2: Handle<Image> = asset_server.load("textures/boss_eye_2.png");
    let boss_eye_texture_3: Handle<Image> = asset_server.load("textures/boss_eye_3.png");
    let boss_eye_layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(160, 128),
        2,
        3,
        None,
        None,
    ));
    commands.insert_resource(BossAssets {
        eye_texture_1: boss_eye_texture_1,
        eye_texture_2: boss_eye_texture_2,
        eye_texture_3: boss_eye_texture_3,
        eye_layout: boss_eye_layout,
    });

    // Keep mesh assets for weapons, pickups, particles, orbit shields
    let game_meshes = GameMeshes {
        player: meshes.add(RegularPolygon::new(PLAYER_RADIUS, 3)),
        enemy_basic: meshes.add(Circle::new(12.0)),
        enemy_fast: meshes.add(Circle::new(8.0)),
        enemy_tank: meshes.add(Rectangle::new(24.0, 24.0)),
        enemy_swarm: meshes.add(Circle::new(5.0)),
        projectile: meshes.add(Circle::new(4.0)),
        xp_gem: meshes.add(Rectangle::new(6.0, 6.0)),
        orbit_shield: meshes.add(Circle::new(8.0)),
        pickup: meshes.add(Circle::new(10.0)),
        boomerang: meshes.add(RegularPolygon::new(6.0, 3)),
        healing_dot: meshes.add(Circle::new(5.0)),
    };

    let game_materials = GameMaterials {
        player: materials.add(Color::srgb(0.0, 1.0, 1.0)),
        player_hit: materials.add(Color::srgb(1.0, 1.0, 1.0)),
        enemy_basic: materials.add(Color::srgb(0.9, 0.2, 0.2)),
        enemy_fast: materials.add(Color::srgb(1.0, 0.9, 0.2)),
        enemy_tank: materials.add(Color::srgb(0.6, 0.1, 0.1)),
        enemy_swarm: materials.add(Color::srgb(1.0, 0.4, 0.7)),
        projectile: materials.add(Color::srgb(0.9, 0.9, 1.0)),
        xp_gem: materials.add(Color::srgb(0.2, 1.0, 0.3)),
        orbit_shield: materials.add(Color::srgb(0.0, 0.9, 0.9)),
        boomerang: materials.add(Color::srgb(0.7, 0.3, 1.0)),
        pickup_orbit: materials.add(Color::srgb(0.0, 1.0, 1.0)),
        pickup_burst: materials.add(Color::srgb(0.9, 0.9, 1.0)),
        pickup_lightning: materials.add(Color::srgb(0.3, 0.5, 1.0)),
        pickup_flame: materials.add(Color::srgb(1.0, 0.5, 0.0)),
        pickup_boomerang: materials.add(Color::srgb(0.7, 0.3, 1.0)),
        pickup_holy_water: materials.add(Color::srgb(0.2, 0.9, 0.7)),
        pickup_updown: materials.add(Color::srgb(0.8, 0.2, 0.9)),
        pickup_phiera: materials.add(Color::srgb(1.0, 0.3, 0.1)),
        healing_dot: materials.add(Color::srgb(0.3, 0.6, 1.0)),
    };

    commands.insert_resource(game_meshes);
    commands.insert_resource(game_materials);

    // Weapon shader materials and quad meshes
    let weapon_shaders = WeaponShaderHandles {
        orbit_shield: std::array::from_fn(|i| {
            let vis = WeaponKind::OrbitShield.visual_def((i + 1) as u32);
            orbit_mats.add(OrbitShieldMaterial {
                data: OrbitShieldData {
                    color: Vec4::from_array(vis.color),
                    intensity: vis.intensity,
                    _pad1: 0.0, _pad2: 0.0, _pad3: 0.0,
                },
            })
        }),
        projectile: std::array::from_fn(|i| {
            let vis = WeaponKind::ProjectileBurst.visual_def((i + 1) as u32);
            proj_mats.add(ProjectileMaterial {
                data: ProjectileData {
                    color: Vec4::from_array(vis.color),
                    intensity: vis.intensity,
                    _pad1: 0.0, _pad2: 0.0, _pad3: 0.0,
                },
            })
        }),
        lightning: lightning_mats.add(LightningMaterial {
            data: LightningData {
                color: Vec4::new(0.3, 0.6, 1.0, 1.0),
                intensity: 2.0,
                lifetime: 1.0,
                seed: 0.0,
                _pad: 0.0,
            },
        }),
        flame_aura: std::array::from_fn(|i| {
            let vis = WeaponKind::FlameAura.visual_def((i + 1) as u32);
            flame_mats.add(FlameAuraMaterial {
                data: FlameAuraData {
                    color: Vec4::from_array(vis.color),
                    intensity: vis.intensity,
                    inner_radius: 0.35,
                    outer_radius: 0.5,
                    _pad: 0.0,
                },
            })
        }),
        boomerang: std::array::from_fn(|i| {
            let vis = WeaponKind::Boomerang.visual_def((i + 1) as u32);
            boom_mats.add(BoomerangMaterial {
                data: BoomerangData {
                    color: Vec4::from_array(vis.color),
                    intensity: vis.intensity,
                    _pad1: 0.0, _pad2: 0.0, _pad3: 0.0,
                },
            })
        }),
        bone: boom_mats.add(BoomerangMaterial {
            data: BoomerangData {
                color: Vec4::new(0.95, 0.9, 0.8, 1.0), // Cream/bone white
                intensity: 1.8,
                _pad1: 0.0, _pad2: 0.0, _pad3: 0.0,
            },
        }),
        holy_water: holy_water_mats.add(HolyWaterMaterial {
            data: HolyWaterData {
                color: Vec4::new(0.2, 0.9, 0.7, 0.85),
                intensity: 1.5,
                lifetime_frac: 1.0,
                _pad1: 0.0,
                _pad2: 0.0,
            },
        }),
        updown: updown_mats.add(UpDownMaterial {
            data: UpDownData {
                color: Vec4::new(0.8, 0.2, 0.9, 0.9),
                intensity: 1.8,
                lifetime_frac: 1.0,
                _pad1: 0.0,
                _pad2: 0.0,
            },
        }),
        phiera: std::array::from_fn(|i| {
            let vis = WeaponKind::Phiera.visual_def((i + 1) as u32);
            phiera_mats.add(PhieraMaterial {
                data: PhieraData {
                    color: Vec4::from_array(vis.color),
                    intensity: vis.intensity,
                    _pad1: 0.0, _pad2: 0.0, _pad3: 0.0,
                },
            })
        }),
        boss_laser: laser_mats.add(BossLaserMaterial {
            data: BossLaserData {
                color: Vec4::new(0.2, 1.0, 0.3, 0.9),
                intensity: 2.0,
                charge_progress: 1.0,
                _pad1: 0.0,
                _pad2: 0.0,
            },
        }),
        orbit_shield_quad: meshes.add(Rectangle::new(30.0, 30.0)),
        projectile_quad: meshes.add(Rectangle::new(20.0, 20.0)),
        lightning_quad: meshes.add(Rectangle::new(1.0, 1.0)),
        flame_aura_quad: meshes.add(Rectangle::new(1.0, 1.0)),
        boomerang_quad: meshes.add(Rectangle::new(24.0, 24.0)),
        bone_quad: meshes.add(Rectangle::new(18.0, 18.0)),
        holy_water_quad: meshes.add(Rectangle::new(1.0, 1.0)),
        updown_quad: meshes.add(Rectangle::new(1.0, 1.0)),
        phiera_quad: meshes.add(Rectangle::new(16.0, 16.0)),
        boss_laser_quad: meshes.add(Rectangle::new(800.0, 80.0)),
    };
    commands.insert_resource(weapon_shaders);

    // HUD bar shader meshes and materials
    let hud_bars = HudBarHandles {
        health_bar_mat: health_bar_mats.add(HealthBarMaterial {
            data: HealthBarData {
                color: Vec4::new(0.8, 0.1, 0.1, 1.0),
                fill_percent: 1.0,
                _pad1: 0.0,
                _pad2: 0.0,
                _pad3: 0.0,
            },
        }),
        xp_bar_mat: xp_bar_mats.add(XpBarMaterial {
            data: XpBarData {
                color: Vec4::new(0.2, 0.5, 1.0, 1.0),
                fill_percent: 0.0,
                _pad1: 0.0,
                _pad2: 0.0,
                _pad3: 0.0,
            },
        }),
        health_bar_mesh: meshes.add(Rectangle::new(300.0, 16.0)),
        xp_bar_mesh: meshes.add(Rectangle::new(300.0, 10.0)),
        health_bg_mat: materials.add(Color::srgba(0.15, 0.02, 0.02, 0.9)),
        xp_bg_mat: materials.add(Color::srgba(0.02, 0.02, 0.15, 0.9)),
        health_bg_mesh: meshes.add(Rectangle::new(300.0, 16.0)),
        xp_bg_mesh: meshes.add(Rectangle::new(300.0, 10.0)),
    };
    commands.insert_resource(hud_bars);

    // Load game fonts
    let font_pixel: Handle<Font> = asset_server.load("fonts/PressStart2P-Regular.ttf");
    let font_retro: Handle<Font> = asset_server.load("fonts/VT323-Regular.ttf");
    let font_mono: Handle<Font> = asset_server.load("fonts/SpaceMono-Regular.ttf");
    commands.insert_resource(GameFontState {
        fonts: vec![
            GameFont { name: "Default", handle: Handle::default() },
            GameFont { name: "Pixel", handle: font_pixel },
            GameFont { name: "Retro", handle: font_retro },
            GameFont { name: "Mono", handle: font_mono },
        ],
        selected: 0,
    });
}

pub fn on_enter_playing(
    mut commands: Commands,
    mut needs_init: ResMut<NeedsGameInit>,
    mut wave_manager: ResMut<WaveManager>,
    mut weapons: ResMut<PlayerWeapons>,
    mut abilities: ResMut<PlayerAbilities>,
    mut stats: ResMut<GameStats>,
    mut pickup_timer: ResMut<PickupSpawnTimer>,
    mut boss_timer: ResMut<BossSpawnTimer>,
    mut pickup_range: ResMut<PickupRange>,
    _game_meshes: Res<GameMeshes>,
    _game_materials: Res<GameMaterials>,
    sprites: Res<SpriteAssets>,
    weapon_shaders: Res<WeaponShaderHandles>,
    hud_bars: Res<HudBarHandles>,
) {
    if !needs_init.0 {
        return;
    }
    needs_init.0 = false;

    // Reset resources
    *wave_manager = WaveManager::default();
    *weapons = PlayerWeapons::default();
    *stats = GameStats::default();
    *pickup_timer = PickupSpawnTimer::default();
    *boss_timer = BossSpawnTimer::default();
    *pickup_range = PickupRange::default();

    // Give starting weapon
    weapons.weapons.push(ActiveWeapon::new(WeaponKind::OrbitShield));

    // Give starting ability
    abilities.abilities.clear();
    abilities.abilities.push(ActiveAbility::new(AbilityKind::Dash));

    // Spawn player with hero sprite
    // Hero sprite: row 0 = front-facing walk, frames 0-3
    commands.spawn((
        Sprite::from_atlas_image(
            sprites.hero_texture.clone(),
            TextureAtlas {
                layout: sprites.hero_layout.clone(),
                index: 0,
            },
        ),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0))
            .with_scale(Vec3::splat(0.75)),
        Player,
        Health {
            current: PLAYER_MAX_HEALTH,
            max: PLAYER_MAX_HEALTH,
        },
        DamageCooldown(0.0),
        Experience {
            current: 0.0,
            next_level: 10.0,
            level: 1,
        },
        MoveSpeed(PLAYER_SPEED),
        AnimationIndices { first: 0, last: 3 },
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        GameEntity,
    ));

    // Spawn initial orbit shield ball
    spawn_orbit_shield_ball(&mut commands, &weapon_shaders, 0);

    // Spawn pet dog companion using dedicated dog sprite sheet
    // Dog sheet: 6 cols x 5 rows, 16x16 frames. Row 0 (frames 0-5) = walk cycle
    commands.spawn((
        Sprite::from_atlas_image(
            sprites.dog_texture.clone(),
            TextureAtlas {
                layout: sprites.dog_layout.clone(),
                index: 0,
            },
        ),
        Transform::from_translation(Vec3::new(30.0, -20.0, 9.0))
            .with_scale(Vec3::splat(2.0)),
        PetDog {
            bone_timer: 0.0,
            bone_cooldown: 2.0,
            bone_damage: 12.0,
        },
        AnimationIndices { first: 0, last: 5 },
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        GameEntity,
    ));

    // Spawn HUD
    spawn_hud(&mut commands);

    // Spawn shader-based health/XP bars as world-space meshes
    // Parented to an anchor entity that follows the camera each frame
    spawn_shader_bars(&mut commands, &hud_bars);

    // Spawn ability slot UI
    spawn_ability_slots(&mut commands, &abilities);
}

pub fn spawn_orbit_shield_ball(
    commands: &mut Commands,
    shaders: &WeaponShaderHandles,
    index: u32,
) {
    commands.spawn((
        Mesh2d(shaders.orbit_shield_quad.clone()),
        MeshMaterial2d(shaders.orbit_shield[0].clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 8.0)),
        OrbitShield {
            orbit_index: index,
            hit_cooldown: 0.0,
        },
        GameEntity,
    ));
}

pub fn sync_orbit_shield_entities(
    commands: &mut Commands,
    shaders: &WeaponShaderHandles,
    orbit_query: &Query<Entity, With<OrbitShield>>,
    target_count: u32,
) {
    let current: Vec<Entity> = orbit_query.iter().collect();
    let current_count = current.len() as u32;
    if current_count > target_count {
        for (i, entity) in current.iter().enumerate() {
            if i as u32 >= target_count {
                commands.entity(*entity).despawn();
            }
        }
    } else {
        for idx in current_count..target_count {
            spawn_orbit_shield_ball(commands, shaders, idx);
        }
    }
}

fn spawn_shader_bars(commands: &mut Commands, hud_bars: &HudBarHandles) {
    // Offsets from HUD camera center (0,0) to top-left bar positions
    // Window: 1280x720 → half = (640, 360)
    // With 16px padding and ~30px for text row:
    const HEALTH_OFFSET: Vec3 = Vec3::new(-474.0, 303.0, 0.0);
    const XP_OFFSET: Vec3 = Vec3::new(-474.0, 286.0, 0.0);
    let hud_layer = RenderLayers::layer(1);

    commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            HudBarAnchor,
            GameEntity,
        ))
        .with_children(|parent| {
            // Health bar background
            parent.spawn((
                Mesh2d(hud_bars.health_bg_mesh.clone()),
                MeshMaterial2d(hud_bars.health_bg_mat.clone()),
                Transform::from_translation(HEALTH_OFFSET + Vec3::Z * 899.0),
                hud_layer.clone(),
                GameEntity,
            ));
            // Health bar fill (shader)
            parent.spawn((
                Mesh2d(hud_bars.health_bar_mesh.clone()),
                MeshMaterial2d(hud_bars.health_bar_mat.clone()),
                Transform::from_translation(HEALTH_OFFSET + Vec3::Z * 900.0),
                hud_layer.clone(),
                HealthBarShader,
                GameEntity,
            ));
            // XP bar background
            parent.spawn((
                Mesh2d(hud_bars.xp_bg_mesh.clone()),
                MeshMaterial2d(hud_bars.xp_bg_mat.clone()),
                Transform::from_translation(XP_OFFSET + Vec3::Z * 899.0),
                hud_layer.clone(),
                GameEntity,
            ));
            // XP bar fill (shader)
            parent.spawn((
                Mesh2d(hud_bars.xp_bar_mesh.clone()),
                MeshMaterial2d(hud_bars.xp_bar_mat.clone()),
                Transform::from_translation(XP_OFFSET + Vec3::Z * 900.0),
                hud_layer.clone(),
                XpBarShader,
                GameEntity,
            ));
        });
}

fn spawn_hud(commands: &mut Commands) {
    // Root HUD container (text labels only — bars are shader meshes now)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            HudEntity,
            GameEntity,
        ))
        .with_children(|parent| {
            // Top row: level + timer + kills
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Level 1"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        LevelText,
                    ));
                    row.spawn((
                        Text::new("0:00"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        TimerText,
                    ));
                    row.spawn((
                        Text::new("Kills: 0"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        KillCountText,
                    ));
                });
        });
}

fn spawn_ability_slots(commands: &mut Commands, abilities: &PlayerAbilities) {
    // Bottom-left container for ability slots
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            },
            HudEntity,
            GameEntity,
        ))
        .with_children(|parent| {
            for (i, ability) in abilities.abilities.iter().enumerate() {
                // Each ability slot
                parent
                    .spawn((
                        Node {
                            width: Val::Px(50.0),
                            height: Val::Px(50.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::End,
                            align_items: AlignItems::Center,
                            overflow: Overflow::clip(),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.2, 0.8)),
                        BorderColor::all(ability.kind.color()),
                        AbilitySlotUI(i),
                    ))
                    .with_children(|slot| {
                        // Cooldown overlay (fills from top down)
                        slot.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(0.0),
                                left: Val::Px(0.0),
                                width: Val::Percent(100.0),
                                height: Val::Percent(0.0), // 0% when ready
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                            AbilityCooldownFill(i),
                        ));

                        // Red flash overlay (hidden by default)
                        slot.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(0.0),
                                left: Val::Px(0.0),
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0)),
                            AbilityCooldownFlash { index: i, remaining: 0.0 },
                        ));

                        // Ability name label
                        slot.spawn((
                            Text::new(ability.kind.display_name()),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(4.0),
                                ..default()
                            },
                        ));

                        // Keybind label at bottom
                        slot.spawn((
                            Text::new(ability.kind.keybind_label()),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.3)),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(2.0),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

pub fn cleanup_game(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
    game_over_entities: Query<Entity, With<GameOverEntity>>,
) {
    for entity in game_entities.iter() {
        commands.entity(entity).despawn();
    }
    for entity in game_over_entities.iter() {
        commands.entity(entity).despawn();
    }
}
