# Holy Water Weapon Design

AOE area-denial weapon inspired by Santa Water from Vampire Survivors.

## Behavior

- Timer-based: every 3.0s, spawns `count` puddles at random positions within 120 units of the player
- Each puddle persists for 3.0s, dealing 6 damage every 0.4s to enemies within its radius (40 units)
- Puddles are stationary - they create area denial where the player has been
- Puddles fade out as their lifetime expires

## Stats

| Stat     | Base Value | Upgrade Effect         |
|----------|-----------|------------------------|
| Damage   | 6/tick    | +30% (IncreaseDamage)  |
| Cooldown | 3.0s      | -20% (ReduceCooldown)  |
| Area     | 40 units  | +25% (IncreaseArea)    |
| Count    | 1 puddle  | +1 (AddProjectile)     |
| Tick rate| 0.4s      | Fixed                  |
| Duration | 3.0s      | Fixed                  |

## Architecture

- `WeaponKind::HolyWater` variant added to enum
- `HolyWaterZone` component: damage, tick_timer, tick_rate, lifetime, radius
- `HolyWaterMaterial` / `HolyWaterData` shader material (teal/aqua green)
- `holy_water_system()` - spawn puddles on timer
- `update_holy_water_zones()` - tick damage + lifetime, despawn when expired
- Damage applied directly (like flame aura), not via weapon_enemy_collision

## Shader

- Circular SDF with soft edge falloff
- Caustic pattern: layered sine waves in polar coordinates
- Expanding concentric ripple rings
- Color: teal core (0.2, 0.9, 0.7) → white center → transparent edges
- `lifetime_frac` uniform fades puddle out before despawn
- Gentle intensity pulse
- Rectangle quad mesh, z=0.5, AlphaMode2d::Blend
