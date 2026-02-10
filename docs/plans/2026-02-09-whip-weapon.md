# Whip Weapon Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a "Whip" weapon that slashes horizontally toward the mouse cursor, hitting all enemies in its path.

**Architecture:** The Whip fires a rectangular slash entity from the player toward the mouse aim direction. It uses the same resource-based weapon tracking (`PlayerWeapons` + `ActiveWeapon`) as existing weapons, with a custom `WhipMaterial` shader for the slash visual. Collision uses rectangle-based distance checks against enemies.

**Tech Stack:** Bevy 0.18, WGSL shader, Rust 2024

---

### Task 1: Add WhipSlash component and WeaponKind::Whip to components.rs

**Files:**
- Modify: `src/components.rs`

**Steps:**
1. Add `Whip` variant to `WeaponKind` enum
2. Add `WHIP_LEVELS` and `WHIP_VISUALS` level/visual tables (8 levels)
3. Wire `Whip` into all `WeaponKind` match arms: `level_def`, `visual_def`, `all()`, `display_name`, `color`
4. Add `WhipSlash` component struct with: `damage`, `lifetime`, `max_lifetime`, `direction` (Vec2), `half_length`, `half_width`, `hit_enemies` (Vec<Entity>)

### Task 2: Add WhipMaterial to shader_materials.rs

**Files:**
- Modify: `src/systems/shader_materials.rs`

**Steps:**
1. Add `WhipData` struct (color, intensity, lifetime_frac, _pad)
2. Add `WhipMaterial` with `Material2d` impl pointing to `shaders/whip_slash.wgsl`

### Task 3: Create whip_slash.wgsl shader

**Files:**
- Create: `assets/shaders/whip_slash.wgsl`

**Steps:**
1. Create WGSL shader with whip slash visual: a bright arc/slash that fades over its lifetime
2. Use `#{MATERIAL_BIND_GROUP}` preprocessor, `globals.time`
3. Red-orange color theme with white-hot core, animating outward sweep

### Task 4: Add whip handles to resources.rs

**Files:**
- Modify: `src/resources.rs`

**Steps:**
1. Add `whip: Handle<WhipMaterial>` to `WeaponShaderHandles`
2. Add `whip_quad: Handle<Mesh>` to `WeaponShaderHandles`
3. Add `pickup_whip: Handle<ColorMaterial>` to `GameMaterials`

### Task 5: Load whip materials in startup.rs

**Files:**
- Modify: `src/systems/startup.rs`

**Steps:**
1. Add `mut whip_mats: ResMut<Assets<WhipMaterial>>` param to `setup_assets`
2. Create whip material handle and whip_quad mesh (rectangular, ~200x80)
3. Add pickup_whip color material (red-orange)

### Task 6: Register WhipMaterial plugin in main.rs

**Files:**
- Modify: `src/main.rs`

**Steps:**
1. Add `Material2dPlugin::<WhipMaterial>::default()`
2. Add `systems::weapons::whip_system` to Playing state update systems
3. Add `systems::weapons::update_whip_slashes` to second Playing update group

### Task 7: Implement whip_system and update_whip_slashes in weapons.rs

**Files:**
- Modify: `src/systems/weapons.rs`

**Steps:**
1. Add `whip_system`: on cooldown, reads mouse aim direction from player facing, spawns WhipSlash entity(ies) with WhipMaterial. For count=2, second slash fires in opposite direction.
2. Add `update_whip_slashes`: ticks lifetime, updates shader lifetime_frac, despawns when expired.

### Task 8: Add whip collision detection in combat.rs and boss.rs

**Files:**
- Modify: `src/systems/combat.rs`
- Modify: `src/systems/collision.rs`
- Modify: `src/systems/boss.rs`

**Steps:**
1. Add `WHIP` constant to `weapon_radii` module (unused but for consistency, or use the slash's own half_width)
2. Add WhipSlash query to `weapon_enemy_collision` - use oriented rectangle collision: check if enemy center is within the slash rectangle (rotated by direction angle)
3. Add WhipSlash query to `weapon_boss_collision` in boss.rs
4. Add WhipSlash to `spawn_weapon_pickups` match arm in combat.rs

### Task 9: Build and verify

**Steps:**
1. `cargo check` to verify no compile errors
2. `cargo run` to visually test
