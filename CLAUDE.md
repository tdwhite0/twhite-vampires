# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
cargo run              # Run in debug mode
cargo build --release  # Build optimized release binary
cargo check            # Fast type-check without building
```

No test suite exists. No linter configuration beyond default `cargo check`.

## Project Overview

**Survivor Arena** — a Vampire Survivors-style 2D arena roguelike built with Bevy 0.18, avian2d 0.5 (physics, used only for gravity=zero config), and rand 0.8. Rust 2024 edition.

## Architecture

### State Machine
`GameState` enum: `Title → Playing → LevelUp → GameOver`. LevelUp pauses gameplay and returns to Playing. GameOver triggers cleanup and returns to Title.

`NeedsGameInit` resource distinguishes first-start from resume-after-level-up to avoid re-initializing the game world.

### Module Structure (src/)
- **main.rs** — App plugin setup, system scheduling. Systems are split into 3 `Update` groups under `Playing` state due to Bevy tuple limits.
- **components.rs** — All ECS marker/data components (Player, Enemy, Boss, weapons, pickups, UI markers, particles).
- **resources.rs** — Game resources: `PlayerWeapons` (vector of `ActiveWeapon`), `PlayerAbilities`, `GameStats`, `WaveManager`, `BossSpawnTimer`, `DebugSettings`, `VisualEffects`. Also defines `WeaponKind`, `EnemyKind`, `UpgradeOption` enums.
- **systems/** — All game logic, one file per domain:
  - `startup.rs` — Camera, asset loading (sprite sheets, meshes, shader materials), HUD spawning, `on_enter_playing`, `cleanup_game`
  - `player.rs` — WASD movement, mouse aim, sprite animation, camera follow, dash ability, pet dog companion
  - `weapons.rs` — 6 weapon systems (orbit_shield, projectile_burst, lightning_zap, flame_aura, boomerang, holy_water) + pet bone attack. Each weapon spawns entities with custom shader materials.
  - `enemies.rs` — Wave-based spawning with time-scaling difficulty, 4 enemy types (Basic/Fast/Tank/Swarm), chase AI
  - `combat.rs` — All collision/damage checks (manual distance-based, not physics engine), XP gem/healing/weapon pickup collection, level-up and game-over triggers, particle effects
  - `boss.rs` — Boss spawning, phase-based laser attack (Charge→Fire→Cooldown), boss-specific collision, health bar, sprite phase changes
  - `ui.rs` — Title/LevelUp/GameOver screens, HUD (health/XP bars, stats), settings panel with debug cheats, notification toast system, floating damage numbers
  - `audio.rs` — Procedural SFX generation (WAV synthesis, pentatonic scale), background music playback (OGG files), music controls
  - `shader_materials.rs` — Custom `Material2d` definitions for all weapon visuals + CRT post-process effect

### Key Patterns
- **Manual collision detection** — All combat uses distance checks, not avian2d collision events. This is intentional.
- **Resource-based weapon tracking** — `PlayerWeapons` resource holds a `Vec<ActiveWeapon>` with kind, level, damage, cooldown, area, count. Weapons are not entity-based.
- **Custom Material2d shaders** — Each weapon has a WGSL shader in `assets/shaders/`. Materials use `@group(#{MATERIAL_BIND_GROUP})` preprocessor (not hardcoded group 2), `globals.time` for animation, and `AlphaMode2d::Blend`.
- **Procedural audio** — All SFX are synthesized at startup as WAV byte arrays (no audio asset files for SFX). Music uses pre-recorded OGG files in `assets/audio/music/`.
- **Notification messages** — `NotificationEvent` uses Bevy's Message system (`MessageWriter`/`MessageReader`, not EventWriter/EventReader).
- **Debug settings in settings menu** — When adding new features, add corresponding debug/cheat toggles to the `DebugSettings` resource and expose them in the in-game settings panel (`ui.rs`).

### Sprite Sheets
- `hero.png` — 40x64 frames, 4 columns x 3 rows (walk frames per direction)
- `creatures.png` — 16x16 frames, 10 columns x 18 rows
- `dog.png` — 16x16 frames, 6 columns x 5 rows
- `boss_eye_*.png` — 160x128 frames, 3 variant textures

## Rust 2024 Edition Gotchas

- `gen` is a reserved keyword — use `rng.gen_range()` instead of `rng.gen()` with rand 0.8

## Bevy 0.18 API Notes

- `single()` / `single_mut()` return `Result` (renamed from `get_single`)
- Window resolution takes `(u32, u32).into()` not floats
- `Sprite::from_atlas_image(texture, TextureAtlas { layout, index })` for sprite sheets
- `BorderColor::all(color)` not `BorderColor(color)`
- Messages not Events: `MessageWriter`/`MessageReader`, `#[derive(Message)]`, `add_message`
- `Children.iter()` yields `Entity` directly
- Shader imports: `bevy::sprite_render::{Material2d, Material2dPlugin, AlphaMode2d}`, `bevy::shader::ShaderRef`
