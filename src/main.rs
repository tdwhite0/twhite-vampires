use bevy::prelude::*;
use bevy::core_pipeline::fullscreen_material::FullscreenMaterialPlugin;
use bevy::sprite_render::Material2dPlugin;
use avian2d::prelude::*;

mod components;
mod resources;
mod systems;

use components::NotificationEvent;
use resources::*;
use systems::shader_materials::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Survivor Arena".into(),
                    resolution: (1280, 720).into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()) // Pixel-art crisp rendering
        )
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(Material2dPlugin::<OrbitShieldMaterial>::default())
        .add_plugins(Material2dPlugin::<ProjectileMaterial>::default())
        .add_plugins(Material2dPlugin::<LightningMaterial>::default())
        .add_plugins(Material2dPlugin::<FlameAuraMaterial>::default())
        .add_plugins(Material2dPlugin::<BoomerangMaterial>::default())
        .add_plugins(Material2dPlugin::<HolyWaterMaterial>::default())
        .add_plugins(Material2dPlugin::<UpDownMaterial>::default())
        .add_plugins(Material2dPlugin::<PhieraMaterial>::default())
        .add_plugins(Material2dPlugin::<BossLaserMaterial>::default())
        .add_plugins(Material2dPlugin::<HealthBarMaterial>::default())
        .add_plugins(Material2dPlugin::<XpBarMaterial>::default())
        .add_plugins(FullscreenMaterialPlugin::<CrtEffect>::default())
        .insert_resource(Gravity(Vec2::ZERO))
        // State
        .init_state::<GameState>()
        // Resources
        .init_resource::<WaveManager>()
        .init_resource::<PlayerWeapons>()
        .init_resource::<GameStats>()
        .init_resource::<NeedsGameInit>()
        .init_resource::<PickupSpawnTimer>()
        .init_resource::<VisualEffects>()
        .init_resource::<PlayerAbilities>()
        .init_resource::<BossSpawnTimer>()
        .init_resource::<DebugSettings>()
        .init_resource::<PickupRange>()
        .add_message::<NotificationEvent>()
        // Startup
        .add_systems(Startup, (
            systems::startup::setup_camera,
            systems::startup::setup_assets,
            systems::audio::setup_sound_assets,
            systems::audio::setup_music,
        ))
        // State enter/exit
        .add_systems(OnEnter(GameState::Title), (
            systems::ui::spawn_title_screen,
            systems::audio::stop_background_music,
        ))
        .add_systems(OnExit(GameState::Title), systems::ui::despawn_title_screen)
        .add_systems(OnEnter(GameState::Playing), (
            systems::startup::on_enter_playing,
            systems::audio::start_background_music,
            systems::ui::spawn_settings_gear,
        ))
        .add_systems(OnEnter(GameState::LevelUp), systems::ui::spawn_level_up_screen)
        .add_systems(OnExit(GameState::LevelUp), systems::ui::despawn_level_up_screen)
        .add_systems(OnEnter(GameState::GameOver), (
            systems::ui::spawn_game_over_screen,
            systems::audio::stop_background_music,
        ))
        .add_systems(OnExit(GameState::GameOver), systems::startup::cleanup_game)
        .add_systems(OnExit(GameState::Playing), systems::ui::despawn_settings_ui)
        // Title state
        .add_systems(Update,
            systems::ui::handle_title_input
                .run_if(in_state(GameState::Title))
        )
        // Playing state - split into groups to stay under tuple limit
        .add_systems(Update, (
            systems::player::player_movement,
            systems::player::player_aim_at_mouse,
            systems::player::camera_follow,
            systems::player::update_damage_flash,
            systems::player::animate_sprites,
            systems::player::dash_ability_system,
            systems::player::update_dash,
            systems::enemies::enemy_spawning,
            systems::enemies::enemy_movement,
            systems::weapons::orbit_shield_system,
            systems::weapons::projectile_burst_system,
            systems::weapons::update_projectiles,
            systems::weapons::lightning_zap_system,
            systems::weapons::flame_aura_system,
            systems::weapons::boomerang_system,
            systems::weapons::bone_attack_system,
            systems::weapons::holy_water_system,
            systems::weapons::updown_system,
            systems::weapons::phiera_system,
            systems::player::pet_follow_player,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (
            systems::weapons::update_boomerangs,
            systems::weapons::update_bone_projectiles,
            systems::weapons::update_holy_water_zones,
            systems::weapons::manage_flame_aura_entity,
            systems::weapons::update_lightning_bolt_entities,
            systems::weapons::update_updown_waves,
            systems::combat::player_enemy_collision,
            systems::combat::dash_enemy_collision,
            systems::combat::weapon_enemy_collision,
            systems::combat::enemy_death,
            systems::combat::xp_gem_collection,
            systems::combat::healing_dot_collection,
            systems::combat::check_level_up,
            systems::combat::check_game_over,
            systems::combat::weapon_pickup_collection,
            systems::combat::spawn_weapon_pickups,
            systems::combat::update_particles,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (
            systems::ui::update_hud,
            systems::ui::update_floating_text,
            systems::ui::draw_weapon_effects,
            systems::ui::update_ability_ui,
            systems::boss::boss_spawning,
            systems::boss::boss_movement,
            systems::boss::boss_laser_attack,
            systems::boss::update_boss_laser_beam,
            systems::boss::boss_laser_damage_player,
            systems::boss::boss_contact_damage,
            systems::boss::weapon_boss_collision,
            systems::boss::dash_boss_collision,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (
            systems::boss::boss_death,
            systems::boss::boss_sprite_phase,
            systems::boss::boss_animate,
            systems::boss::flame_aura_damages_boss,
            systems::boss::holy_water_damages_boss,
            systems::boss::spawn_boss_health_bar,
            systems::boss::update_boss_health_bar,
            systems::shader_materials::update_crt_time,
            systems::ui::update_ability_cooldown_flash,
            systems::ui::update_shader_bars,
        ).run_if(in_state(GameState::Playing)))
        // New boss attack systems
        .add_systems(Update, (
            systems::boss::dragon_breath_attack,
            systems::boss::update_dragon_fireballs,
            systems::boss::dragon_fire_damage_player,
            systems::boss::cleanup_dragon_fire_zones,
            systems::boss::necromancer_attack,
            systems::boss::update_necro_orbs,
            systems::boss::necro_orb_damage_player,
            systems::boss::slime_king_attack_system,
            systems::boss::update_slime_shockwaves,
            systems::boss::update_slime_splits,
            systems::boss::slime_split_damage_player,
            systems::boss::weapon_slime_split_collision,
        ).run_if(in_state(GameState::Playing)))
        // Level up state
        .add_systems(Update, (
            systems::ui::handle_level_up_selection,
            systems::ui::button_hover_visuals,
        ).run_if(in_state(GameState::LevelUp)))
        // Game over state
        .add_systems(Update, (
            systems::ui::handle_game_over_input,
            systems::ui::animate_game_over_screen,
        ).run_if(in_state(GameState::GameOver)))
        // Settings menu (runs during Playing state)
        .add_systems(Update, (
            systems::ui::toggle_settings_panel,
            systems::ui::handle_settings_tab_switch,
            systems::ui::handle_settings_invincible,
            systems::ui::handle_settings_weapon_equip,
            systems::ui::handle_settings_weapon_level,
            systems::ui::handle_settings_damage_mult,
            systems::ui::handle_settings_crt,
            systems::ui::handle_settings_spawn_boss,
            systems::ui::handle_settings_boss_hitboxes,
            systems::ui::handle_spawn_rate_slider,
            systems::ui::settings_button_hover,
            systems::audio::handle_music_track_selection,
            systems::audio::handle_music_mute,
            systems::audio::handle_music_volume_slider,
            systems::ui::handle_settings_font_cycle,
        ).run_if(in_state(GameState::Playing)))
        // Always running
        .add_systems(Update, (
            systems::ui::draw_arena_grid,
            systems::ui::draw_pickup_labels,
            systems::ui::draw_boss_hitboxes,
            systems::ui::spawn_notification_toasts,
            systems::ui::update_notification_toasts,
            systems::ui::apply_font_to_all_text,
        ))
        .run();
}
