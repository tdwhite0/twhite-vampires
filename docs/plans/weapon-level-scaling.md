# Per-Weapon Level Scaling System — Implementation Plan

## Context

**Project:** Survivor Arena — a Vampire Survivors-style 2D arena roguelike.
**Stack:** Bevy 0.18, avian2d 0.5, rand 0.8, Rust 2024 edition.
**Working directory:** `/Users/twhite/Documents/Projects/twhite-topdown`

Weapon leveling currently uses generic scaling (`damage *= 1.2, cooldown *= 0.9` per level) for all 7 weapons. We want to replace this with explicit per-level stat tables, gameplay mechanic changes at key levels (piercing projectiles, chain lightning), visual color/intensity progression per level, and collapse of the 4 weapon upgrade types into a single "Level Up Weapon" option.

Max level (8) should feel like screen-clearing god mode.

## Critical Codebase Notes

- **Rust 2024 edition:** `gen` is a reserved keyword. Use `rng.gen_range()` not `rng.gen()`.
- **Bevy 0.18 Messages:** Use `MessageWriter`/`MessageReader`, `#[derive(Message)]`, NOT `EventWriter`/`EventReader`.
- **Bevy 0.18 queries:** `single()`/`single_mut()` return `Result` (renamed from `get_single`).
- **Manual collision:** All combat uses distance checks, NOT avian2d physics events. This is intentional.
- **Shader materials:** `@group(#{MATERIAL_BIND_GROUP})` preprocessor template in WGSL, NOT hardcoded group 2.
- After every code change, run `cargo check` to verify compilation.

## Key Files & Their Roles

| File | Lines | Role |
|------|-------|------|
| `src/components.rs` | ~475 | All ECS components. `WeaponKind` enum (line 52), `Projectile` component (line ~107, has `piercing: bool` field), `UpgradeKind` enum (line ~431) |
| `src/resources.rs` | ~500 | `ActiveWeapon` struct (line 57: kind/level/damage/cooldown/timer/area/count), `set_level()` (line 81), `new()` (line 98), `WeaponShaderHandles` (line 428), `PlayerWeapons` resource |
| `src/systems/weapons.rs` | ~745 | All 7 weapon fire/update systems. Each reads from `PlayerWeapons` resource. |
| `src/systems/combat.rs` | ~580 | `weapon_enemy_collision` (handles piercing at line ~174), `weapon_pickup_collection` (inline level-up at line 480) |
| `src/systems/boss.rs` | ~795 | `weapon_boss_collision` (handles piercing for projectiles) |
| `src/systems/startup.rs` | ~626 | `setup_assets` (creates shader materials), `spawn_orbit_shield_ball` (line ~390), `on_enter_playing` |
| `src/systems/ui/settings.rs` | ~946 | `handle_settings_weapon_level` (line ~793), weapon tab UI |
| `src/systems/ui/screens.rs` | ~449 | `generate_upgrades` (line 268), `apply_upgrade` (line 370), `UpgradeKind` display methods |
| `src/systems/shader_materials.rs` | ~321 | All Material2d definitions. Each has `color: Vec4` and `intensity: f32` uniforms. |
| `src/main.rs` | ~189 | System registration |

## Level Tables

### Orbit Shield
Cooldown is always 0.0 (continuous). Count = number of orbiting entities.

| Lv | Dmg | CD | Area | Count |
|----|-----|----|------|-------|
| 1 | 15.0 | 0.0 | 60.0 | 1 |
| 2 | 18.0 | 0.0 | 65.0 | 2 |
| 3 | 22.0 | 0.0 | 70.0 | 2 |
| 4 | 26.0 | 0.0 | 80.0 | 3 |
| 5 | 32.0 | 0.0 | 90.0 | 4 |
| 6 | 38.0 | 0.0 | 100.0 | 5 |
| 7 | 45.0 | 0.0 | 110.0 | 6 |
| 8 | 55.0 | 0.0 | 130.0 | 8 |

### Projectile Burst
Count = number of projectiles per burst. Area is unused (0.0). Piercing enabled at level 5+.

| Lv | Dmg | CD | Area | Count |
|----|-----|----|------|-------|
| 1 | 10.0 | 2.0 | 0.0 | 4 |
| 2 | 12.0 | 1.8 | 0.0 | 5 |
| 3 | 15.0 | 1.6 | 0.0 | 6 |
| 4 | 18.0 | 1.4 | 0.0 | 8 |
| 5 | 22.0 | 1.2 | 0.0 | 8 |
| 6 | 28.0 | 1.0 | 0.0 | 10 |
| 7 | 35.0 | 0.8 | 0.0 | 12 |
| 8 | 45.0 | 0.5 | 0.0 | 16 |

### Lightning Zap
Count = number of chain targets. Area = targeting range from each chain origin.

| Lv | Dmg | CD | Area | Count |
|----|-----|----|------|-------|
| 1 | 25.0 | 1.5 | 200.0 | 1 |
| 2 | 30.0 | 1.3 | 220.0 | 1 |
| 3 | 35.0 | 1.1 | 250.0 | 2 |
| 4 | 42.0 | 1.0 | 280.0 | 2 |
| 5 | 50.0 | 0.8 | 320.0 | 3 |
| 6 | 60.0 | 0.6 | 360.0 | 4 |
| 7 | 75.0 | 0.5 | 400.0 | 5 |
| 8 | 100.0 | 0.3 | 500.0 | 8 |

### Flame Aura
Count is unused (0). CD = damage tick rate. Area = aura radius.

| Lv | Dmg | CD | Area | Count |
|----|-----|----|------|-------|
| 1 | 8.0 | 0.5 | 60.0 | 0 |
| 2 | 10.0 | 0.45 | 70.0 | 0 |
| 3 | 13.0 | 0.4 | 85.0 | 0 |
| 4 | 16.0 | 0.35 | 100.0 | 0 |
| 5 | 20.0 | 0.3 | 120.0 | 0 |
| 6 | 26.0 | 0.25 | 145.0 | 0 |
| 7 | 33.0 | 0.2 | 175.0 | 0 |
| 8 | 45.0 | 0.15 | 220.0 | 0 |

### Boomerang
Count = number of boomerangs. Area = range (stored as max_range but currently unused in movement code).

| Lv | Dmg | CD | Area | Count |
|----|-----|----|------|-------|
| 1 | 15.0 | 2.5 | 250.0 | 1 |
| 2 | 18.0 | 2.2 | 280.0 | 1 |
| 3 | 22.0 | 2.0 | 300.0 | 2 |
| 4 | 28.0 | 1.7 | 330.0 | 2 |
| 5 | 35.0 | 1.4 | 360.0 | 3 |
| 6 | 42.0 | 1.1 | 400.0 | 4 |
| 7 | 52.0 | 0.8 | 450.0 | 5 |
| 8 | 65.0 | 0.5 | 500.0 | 8 |

### Holy Water
Count = number of zones per cast. Area = zone radius.

| Lv | Dmg | CD | Area | Count |
|----|-----|----|------|-------|
| 1 | 6.0 | 3.0 | 40.0 | 1 |
| 2 | 8.0 | 2.8 | 55.0 | 1 |
| 3 | 10.0 | 2.5 | 65.0 | 2 |
| 4 | 13.0 | 2.2 | 75.0 | 2 |
| 5 | 16.0 | 2.0 | 90.0 | 3 |
| 6 | 20.0 | 1.6 | 100.0 | 4 |
| 7 | 25.0 | 1.2 | 115.0 | 5 |
| 8 | 35.0 | 0.8 | 140.0 | 8 |

### UpDown
Count = number of wave pairs (each pair = 1 up + 1 down). Area = wave height.

| Lv | Dmg | CD | Area | Count |
|----|-----|----|------|-------|
| 1 | 10.0 | 2.0 | 80.0 | 1 |
| 2 | 13.0 | 1.8 | 95.0 | 1 |
| 3 | 16.0 | 1.6 | 110.0 | 2 |
| 4 | 20.0 | 1.4 | 130.0 | 2 |
| 5 | 25.0 | 1.2 | 150.0 | 3 |
| 6 | 32.0 | 1.0 | 175.0 | 4 |
| 7 | 40.0 | 0.7 | 200.0 | 5 |
| 8 | 55.0 | 0.4 | 250.0 | 8 |

## Implementation Steps

### Step 1: Level table data structures + rewrite set_level/new

**Files:** `src/components.rs`, `src/resources.rs`

**In `src/components.rs`** (where `WeaponKind` lives, around line 52):

Add a struct and const arrays:

```rust
pub struct WeaponLevelDef {
    pub damage: f32,
    pub cooldown: f32,
    pub area: f32,
    pub count: u32,
}

pub const ORBIT_SHIELD_LEVELS: [WeaponLevelDef; 8] = [
    WeaponLevelDef { damage: 15.0, cooldown: 0.0, area: 60.0, count: 1 },
    // ... all 8 levels from the table above
];
// ... repeat for all 7 weapons
```

Add a lookup method on `WeaponKind`:

```rust
impl WeaponKind {
    pub fn level_def(&self, level: u32) -> &'static WeaponLevelDef {
        let idx = (level.clamp(1, 8) - 1) as usize;
        match self {
            WeaponKind::OrbitShield => &ORBIT_SHIELD_LEVELS[idx],
            // ... etc
        }
    }
}
```

**In `src/resources.rs`:**

Rewrite `ActiveWeapon::set_level()` (currently line 81-96) to use table lookup:

```rust
pub fn set_level(&mut self, level: u32) {
    let level = level.clamp(1, Self::MAX_LEVEL);
    let def = self.kind.level_def(level);
    self.level = level;
    self.damage = def.damage;
    self.cooldown = def.cooldown;
    self.area = def.area;
    self.count = def.count;
    // Don't reset timer - preserve cooldown progress
}
```

Rewrite `ActiveWeapon::new()` (currently line 98-153) to use the table:

```rust
pub fn new(kind: WeaponKind) -> Self {
    let def = kind.level_def(1);
    Self {
        kind,
        level: 1,
        damage: def.damage,
        cooldown: def.cooldown,
        timer: 0.0,
        area: def.area,
        count: def.count,
    }
}
```

This eliminates the large match block.

**Verify:** `cargo check`

### Step 2: Projectile piercing from weapon level

**File:** `src/systems/weapons.rs`

In `projectile_burst_system()`, find where `Projectile` is spawned (around line 84-95). Change:

```rust
piercing: false,
```

to:

```rust
piercing: weapon.level >= 5,
```

That's the only change needed. The collision code in `combat.rs` (weapon_enemy_collision, around line 174) and `boss.rs` (weapon_boss_collision) already handles the `piercing` field: when true, the projectile tracks hit enemies instead of despawning on first hit.

**Verify:** `cargo check`

### Step 3: Chain lightning

**File:** `src/systems/weapons.rs`

The current `lightning_zap_system()` (around line 120-203) finds ONE nearest enemy, damages it, and spawns one bolt visual. The `count` field from `ActiveWeapon` is currently ignored by lightning.

Rewrite the targeting/damage section to chain to `weapon.count` targets:

1. The enemy query needs `Entity` added. Change from:
   `Query<(&Transform, &mut EnemyHealth), (With<Enemy>, Without<Player>)>`
   to:
   `Query<(Entity, &Transform, &mut EnemyHealth), (With<Enemy>, Without<Player>)>`

2. First pass — collect chain targets (to avoid borrow conflicts):
   ```rust
   let chain_count = weapon.count.max(1);
   let mut chain_targets: Vec<(Entity, Vec2)> = Vec::new();
   let mut current_origin = player_pos;

   for _ in 0..chain_count {
       let mut best_dist = f32::MAX;
       let mut best: Option<(Entity, Vec2)> = None;

       for (entity, transform, _) in enemy_query.iter() {
           if chain_targets.iter().any(|(e, _)| *e == entity) { continue; }
           let pos = transform.translation.truncate();
           let dist = current_origin.distance(pos);
           if dist < range && dist < best_dist {
               best_dist = dist;
               best = Some((entity, pos));
           }
       }

       let Some((entity, pos)) = best else { break; };
       chain_targets.push((entity, pos));
       current_origin = pos;
   }
   ```

3. Second pass — apply damage:
   ```rust
   for (entity, _) in &chain_targets {
       if let Ok((_, _, mut health)) = enemy_query.get_mut(*entity) {
           if health.0 > 0.0 {
               health.0 -= damage;
           }
       }
   }
   ```

4. Third pass — spawn bolt visuals for each chain link:
   ```rust
   if !chain_targets.is_empty() {
       play_sound(&mut commands, &sound_assets.weapon_lightning, 0.10);
       let mut bolt_origin = player_pos;
       for (_, target_pos) in &chain_targets {
           // spawn lightning bolt from bolt_origin to target_pos
           // (extract existing bolt spawning code into helper)
           bolt_origin = *target_pos;
       }
   }
   ```

Extract the bolt spawning code (currently lines ~180-202: creating material, spawning entity) into a helper function like `spawn_lightning_bolt(commands, weapon_shaders, lightning_mats, from, to, rng)`.

**Verify:** `cargo check`

### Step 4: Orbit shield entity count sync

**File:** `src/systems/startup.rs`

Add a public utility function after `spawn_orbit_shield_ball`:

```rust
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
```

**Verify:** `cargo check`

### Step 5: Update weapon_pickup_collection

**File:** `src/systems/combat.rs`

In `weapon_pickup_collection()`, find the inline upgrade logic (around line 478-486):

```rust
// Current code:
if weapons.has(kind) {
    if let Some(weapon) = weapons.get_mut(kind) {
        weapon.level += 1;
        weapon.damage *= 1.2;
        if weapon.cooldown > 0.0 {
            weapon.cooldown *= 0.9;
        }
    }
```

Replace with:

```rust
if weapons.has(kind) {
    if let Some(weapon) = weapons.get_mut(kind) {
        if weapon.level < ActiveWeapon::MAX_LEVEL {
            weapon.set_level(weapon.level + 1);
        }
    }
```

Also need to handle orbit shield entity sync here. Add `orbit_query: Query<Entity, With<OrbitShield>>` and `weapon_shaders: Res<WeaponShaderHandles>` parameters to the function signature, and after the level change:

```rust
if kind == WeaponKind::OrbitShield {
    if let Some(weapon) = weapons.get(kind) {
        startup::sync_orbit_shield_entities(&mut commands, &weapon_shaders, &orbit_query, weapon.count);
    }
}
```

Update the system registration in `main.rs` if the function signature changes require it (Bevy should auto-inject the new query params).

**Verify:** `cargo check`

### Step 6: Update settings UI level handler

**File:** `src/systems/ui/settings.rs`

The `handle_settings_weapon_level()` function (around line 793) currently calls `weapon.set_level(level)` but doesn't manage orbit shield entities.

Add parameters:
```rust
orbit_query: Query<Entity, With<OrbitShield>>,
weapon_shaders: Res<WeaponShaderHandles>,
```

After `weapon.set_level(level)`, add:

```rust
if kind == WeaponKind::OrbitShield {
    startup::sync_orbit_shield_entities(&mut commands, &weapon_shaders, &orbit_query, weapon.count);
}
```

**Verify:** `cargo check`

### Step 7: Collapse level-up upgrade types

**Files:** `src/components.rs`, `src/systems/ui/screens.rs`

**In `src/components.rs`** (UpgradeKind enum, around line 431):

Replace the 4 weapon stat variants:
```rust
IncreaseDamage(WeaponKind),
ReduceCooldown(WeaponKind),
IncreaseArea(WeaponKind),
AddProjectile(WeaponKind),
```

With a single:
```rust
LevelUpWeapon(WeaponKind),
```

Update `name()` and `description()` methods on `UpgradeKind`:
```rust
UpgradeKind::LevelUpWeapon(kind) => format!("{} Lv.{}", kind.display_name(), /* next level */),
```

Note: `name()` doesn't have access to the current weapon level. Either pass it in, or just show the weapon name and let the description say "Level up".

**In `src/systems/ui/screens.rs`:**

Update `generate_upgrades()` (line 268): Replace the 4-upgrade-per-weapon section with:
```rust
for weapon in &weapons.weapons {
    if weapon.level < ActiveWeapon::MAX_LEVEL {
        pool.push(UpgradeKind::LevelUpWeapon(weapon.kind));
    }
}
```

Update `apply_upgrade()` (line 370): Replace the 4 weapon stat cases with:
```rust
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
```

Fix any compiler errors from removed enum variants elsewhere in the file.

**Verify:** `cargo check`

### Step 8: Per-level visual materials

**Files:** `src/resources.rs`, `src/systems/startup.rs`, `src/systems/weapons.rs`

**8a. Define visual tables** (in `src/components.rs` alongside level tables):

```rust
pub struct WeaponVisualDef {
    pub color: [f32; 4],  // RGBA
    pub intensity: f32,
}
```

Define const arrays per weapon, 8 entries. Color progressions:
- **Orbit Shield:** cyan `(0.0, 0.8, 0.8)` -> electric blue `(0.3, 0.5, 1.0)` -> white-blue `(0.7, 0.85, 1.0)` | intensity 1.2 -> 2.5
- **Projectile:** pale blue `(0.9, 0.9, 1.0)` -> orange `(1.0, 0.6, 0.2)` -> magenta `(1.0, 0.2, 0.8)` | intensity 1.5 -> 3.0
- **Lightning:** blue `(0.3, 0.6, 1.0)` -> purple `(0.6, 0.3, 1.0)` -> white `(0.9, 0.9, 1.0)` | intensity 2.0 -> 4.0
- **Flame Aura:** orange `(1.0, 0.5, 0.0)` -> red `(1.0, 0.2, 0.0)` -> white-hot `(1.0, 0.9, 0.7)` | intensity 1.5 -> 3.5
- **Boomerang:** purple `(0.7, 0.3, 1.0)` -> hot pink `(1.0, 0.2, 0.6)` -> white `(1.0, 0.8, 1.0)` | intensity 1.5 -> 3.0
- **Holy Water:** teal `(0.2, 0.9, 0.7)` -> green `(0.1, 1.0, 0.4)` -> bright emerald `(0.5, 1.0, 0.7)` | intensity 1.5 -> 3.0
- **UpDown:** purple `(0.8, 0.2, 0.9)` -> magenta `(1.0, 0.3, 0.7)` -> white `(1.0, 0.8, 1.0)` | intensity 1.8 -> 3.5

Interpolate linearly between endpoints for levels 2-7.

Add `WeaponKind::visual_def(level) -> &'static WeaponVisualDef` lookup method.

**8b. Update `WeaponShaderHandles`** (in `src/resources.rs`, around line 428):

Replace single handles with arrays for shared-material weapons:
```rust
pub orbit_shield: [Handle<OrbitShieldMaterial>; 8],
pub projectile: [Handle<ProjectileMaterial>; 8],
pub flame_aura: [Handle<FlameAuraMaterial>; 8],
pub boomerang: [Handle<BoomerangMaterial>; 8],
```

Keep single handles for per-instance-material weapons (lightning, holy_water, updown) and for bone (pet, no levels).

**8c. Create materials in `setup_assets()`** (`src/systems/startup.rs`):

For each shared-material weapon, create 8 variants:
```rust
let orbit_shield_mats: [Handle<OrbitShieldMaterial>; 8] = std::array::from_fn(|i| {
    let vis = WeaponKind::OrbitShield.visual_def((i + 1) as u32);
    orbit_mats.add(OrbitShieldMaterial {
        data: OrbitShieldData {
            color: Vec4::from_array(vis.color),
            intensity: vis.intensity,
            _pad1: 0.0, _pad2: 0.0, _pad3: 0.0,
        },
    })
});
```

**8d. Update weapon spawn code** (`src/systems/weapons.rs`):

For shared-material weapons, index into arrays:
```rust
// projectile_burst_system:
MeshMaterial2d(weapon_shaders.projectile[(weapon.level.clamp(1, 8) - 1) as usize].clone()),

// boomerang_system:
MeshMaterial2d(weapon_shaders.boomerang[(weapon.level.clamp(1, 8) - 1) as usize].clone()),
```

For per-instance-material weapons (lightning, holy_water, updown), use the visual table's color/intensity when creating the per-instance material instead of hardcoded values:
```rust
// lightning_zap_system - when creating per-bolt material:
let vis = WeaponKind::LightningZap.visual_def(weapon.level);
let bolt_mat = lightning_mats.add(LightningMaterial {
    data: LightningData {
        color: Vec4::from_array(vis.color),
        intensity: vis.intensity,
        // ...
    },
});
```

Same pattern for `holy_water_system` and `updown_system`.

**8e. Update `manage_flame_aura_entity`** (`src/systems/weapons.rs`):

When the aura entity already exists, swap its material to match current level:
```rust
let level_idx = (weapons.get(WeaponKind::FlameAura).map(|w| w.level).unwrap_or(1).clamp(1, 8) - 1) as usize;
commands.entity(entity).insert(MeshMaterial2d(weapon_shaders.flame_aura[level_idx].clone()));
```

**8f. Update `orbit_shield_system`** (`src/systems/weapons.rs`):

Orbit entities persist across levels, so their material needs updating. Either:
- Swap material each frame (cheap, just a handle clone), or
- Only swap on level change detection

Simplest: check level and swap each frame in the orbit loop:
```rust
let level_idx = (weapon.level.clamp(1, 8) - 1) as usize;
// In the orbit iter loop, insert new material:
commands.entity(orbit_entity).insert(MeshMaterial2d(weapon_shaders.orbit_shield[level_idx].clone()));
```

Note: this requires adding `mut commands: Commands` to the orbit_shield_system parameters and getting entity from the query.

**Verify:** `cargo check`, then `cargo run` and visually confirm color progression.

## Dependency Order

```
Step 1 (tables + set_level)  <-- foundation, do first
  ├── Step 2 (piercing - one line change)
  ├── Step 3 (chain lightning - most complex)
  ├── Step 4 (orbit sync utility)
  │     ├── Step 5 (pickup collection uses sync)
  │     ├── Step 6 (settings UI uses sync)
  │     └── Step 7 (upgrade screen uses sync)
  └── Step 8 (visual materials - cosmetic, do last)
```

Steps 2, 3, 4 can be done in parallel after Step 1.
Steps 5, 6, 7 can be done in parallel after Step 4.
Step 8 can be done last, independently.

## Verification

After all steps complete:

1. `cargo check` — must compile with no errors
2. `cargo run` — launch game
3. Open settings > Weapons tab
4. Equip each weapon, click through levels 1-8, verify:
   - Orbit shield: correct number of orbs appear/disappear at each level
   - Projectile burst: pierces through enemies at level 5+
   - Lightning: chains to multiple enemies at level 3+ (visible bolt per chain)
   - All weapons: fire rate, damage, counts match the tables
   - Visual colors progress from dim to bright across levels
5. Start a game, play through level-ups:
   - "Level Up [Weapon]" appears as a single upgrade option per owned weapon
   - Selecting it increments weapon level by 1
   - Orbit shield orbs sync correctly on level-up
6. Pick up weapon pickups on the ground — verify they level up the weapon
