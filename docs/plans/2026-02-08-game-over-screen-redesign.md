# Game Over Screen Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the static Game Over screen with a dynamic, stats-rich, animated screen that shows a full run summary with staggered reveal animations.

**Architecture:** Expand `GameStats` to track additional stats (bosses killed, XP collected, damage taken, heals collected) throughout gameplay. Add a `GameOverAnimTimer` component to drive staggered reveal of stats rows on the Game Over screen. Weapons collected are read from `PlayerWeapons` resource at display time.

**Tech Stack:** Bevy 0.18 UI (Node, Text, BackgroundColor), existing component/resource patterns.

---

### Task 1: Expand GameStats resource to track more stats

**Files:**
- Modify: `src/resources.rs` (GameStats struct, lines 306-310)

**Step 1: Add new fields to GameStats**

In `src/resources.rs`, expand the `GameStats` struct:

```rust
#[derive(Resource, Default)]
pub struct GameStats {
    pub time_survived: f32,
    pub enemies_killed: u32,
    pub bosses_killed: u32,
    pub xp_collected: f32,
    pub damage_taken: f32,
    pub heals_collected: u32,
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: PASS (all fields default to 0)

---

### Task 2: Instrument gameplay systems to populate new stats

**Files:**
- Modify: `src/systems/boss.rs` (boss_death function, ~line 1409) — increment `stats.bosses_killed`
- Modify: `src/systems/combat.rs` (xp_gem_collection, ~line 334) — accumulate `stats.xp_collected`
- Modify: `src/systems/combat.rs` (player_enemy_collision, near damage logic) — accumulate `stats.damage_taken`
- Modify: `src/systems/combat.rs` (healing_dot_collection, ~line 373) — increment `stats.heals_collected`

**Step 1: Add `stats: ResMut<GameStats>` to `boss_death` signature and increment `stats.bosses_killed += 1` after boss entity is despawned.**

**Step 2: In `xp_gem_collection`, add `stats.xp_collected += gem.0;` when an XP gem is collected.**

**Step 3: In `player_enemy_collision`, add `stats.damage_taken += damage_amount;` when the player takes damage (find where `health.current -= X` happens and capture X).**

**Step 4: In `healing_dot_collection`, add `stats.heals_collected += 1;` when a heal is picked up.**

**Step 5: Verify it compiles**

Run: `cargo check`
Expected: PASS

**Step 6: Commit**

```bash
git add src/resources.rs src/systems/boss.rs src/systems/combat.rs
git commit -m "feat: expand GameStats to track bosses killed, XP, damage taken, heals"
```

---

### Task 3: Add GameOverAnimTimer component and update animation system

**Files:**
- Modify: `src/components.rs` — add `GameOverAnimTimer` component
- Modify: `src/main.rs` — register animation system for GameOver state

**Step 1: Add component to `src/components.rs`**

```rust
#[derive(Component)]
pub struct GameOverAnimTimer {
    pub elapsed: f32,
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: PASS

---

### Task 4: Redesign the Game Over screen spawn function

**Files:**
- Modify: `src/systems/ui/screens.rs` — rewrite `spawn_game_over_screen`
- Modify: `src/systems/ui/screens.rs` — rewrite `handle_game_over_input` to include animation update logic

**Step 1: Replace `spawn_game_over_screen` with new dynamic version**

The new screen layout:
- Full-screen dark overlay with red tint
- "GAME OVER" title (large, red, bold)
- Stats panel with individual rows, each hidden initially (opacity 0):
  - Time Survived: M:SS
  - Enemies Slain: N
  - Bosses Defeated: N
  - XP Collected: N
  - Damage Taken: N
  - Heals Collected: N
  - Level Reached: N
- Weapons row showing each equipped weapon name + level
- "Press SPACE to Restart" at bottom (pulsing)
- `GameOverAnimTimer { elapsed: 0.0 }` on root entity

Each stat row uses a marker component `GameOverStatRow(pub u32)` where the u32 is the row index (0-6). The animation system reveals rows sequentially based on elapsed time.

**Step 2: Add `GameOverStatRow` component to `src/components.rs`**

```rust
#[derive(Component)]
pub struct GameOverStatRow(pub u32);

#[derive(Component)]
pub struct GameOverRestartPrompt;
```

**Step 3: Write the spawn function that builds the full layout**

The function reads `GameStats`, `PlayerWeapons`, and player `Experience` to populate all fields. Each stat row starts with `BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0))` and text `TextColor(Color::srgba(..., 0.0))` (fully transparent).

**Step 4: Write `animate_game_over_screen` system**

This system:
- Queries `GameOverAnimTimer` and ticks `elapsed += delta`
- For each `GameOverStatRow(index)`, if `elapsed > 0.3 + index * 0.25`, fade in that row (lerp alpha from 0 to 1 over 0.3s)
- The title fades in first (elapsed 0.0 - 0.5)
- Stats rows stagger starting at 0.5s, each 0.25s apart
- Restart prompt appears after all rows, with pulsing alpha (sin wave)

**Step 5: Register `animate_game_over_screen` in main.rs under GameOver state**

In `main.rs`, change:
```rust
.add_systems(Update,
    systems::ui::handle_game_over_input
        .run_if(in_state(GameState::GameOver))
)
```
to:
```rust
.add_systems(Update, (
    systems::ui::handle_game_over_input,
    systems::ui::animate_game_over_screen,
).run_if(in_state(GameState::GameOver)))
```

**Step 6: Verify it compiles and run to test visually**

Run: `cargo check` then `cargo run`
Expected: Game Over screen shows animated stats reveal

**Step 7: Commit**

```bash
git add src/components.rs src/systems/ui/screens.rs src/main.rs
git commit -m "feat: redesign Game Over screen with animated stats reveal and weapon showcase"
```
