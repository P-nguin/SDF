# Predictive Targeting System

## Purpose

Add a player-issued attack order for selected ships. Each turret should aim ahead of its chosen target using projectile speed and target velocity, and independently choose the highest-priority target it can engage. This plan reflects the current Bevy ECS structure and separates the first usable targeting flow from later extensions.

## Current Code Findings

- `Ship` entities own their hull cells through Bevy's `ChildOf` relationship. A turret is nested several levels below its ship: ship cell (`TurretBase`) -> generated voxel wrapper -> base sprite -> rotating turret sprite.
- `TargetControl` currently lives on the rotating turret entity and stores one `current_target` plus aim/lock state.
- `target_control_system` turns each turret toward the target's current world position. `Turret.projectile_speed` is available, but target entities are not queried for `LinearVelocity`.
- `debug_targeting_system` in `src/main.rs` continuously assigns a dummy target to turrets whose *direct* parent is selected. That assumption does not match the actual hierarchy; the system also moves the dummy while `T` is held rather than issuing a click order.
- `handle_selection_input` in `src/controls/systems.rs` already converts the cursor to world coordinates and supports click/box selection. A click currently considers every ship within 100 world units of the cursor; it is not a physics raycast or a nearest-entity pick.
- `fire_control_system` climbs `ChildOf` links to infer the projectile's source ship. This is a useful precedent, but it assumes the topmost ancestor is the ship.
- `firing_arc` and `max_range` are stored on each turret. The current aiming and firing code does not enforce either value. `is_locked_on` only indicates that the turret has finished turning.
- Ships have `LinearVelocity`; projectiles are spawned with a configured `LinearVelocity` and Avian collision handling. There is no existing faction/team component, target-acquisition system, or targeting-specific raycast/picking system.

## Recommended Design

### Ownership and targeting state

Keep the Bevy hierarchy as the source of truth for weapon ownership; do not maintain a duplicate list of weapon entities on `Ship`. Add a small ancestor-walking helper that resolves the nearest ancestor with `Ship`, and use it wherever a weapon needs its owner. This avoids relying on either a direct parent or the topmost root being the ship.

Add a ship-level component, for example `ShipTargetingOrders`, containing an ordered `Vec<Entity>` of commanded targets. The first entry is the primary priority; later entries are fallbacks. Preserve `TargetControl.current_target` as the turret's current engagement choice, not as the player's order. A turret can therefore choose a different entry from the same ship order without mutating the command.

Before acquisition, prune despawned or otherwise invalid entries. Initially a valid target is a live `Ship` other than the owning ship. Until the project has factions, treat every other ship as targetable; do not describe this as hostility detection. Introduce team/faction filtering only when the game defines that model.

### Player command input

Extend the controls layer with a targeting mode/state and handle it before ordinary selection release logic:

1. `T` just-pressed arms a one-shot target command. `Shift+T` arms an append command so existing priorities remain intact.
2. Convert the cursor to world coordinates using the existing camera conversion. Pick exactly one target ship deterministically: the nearest eligible ship within the project's existing 100-unit click radius. Keep the current ship-center picking behavior for the first iteration; replacing it with hull/physics picking is a separate improvement.
3. On a valid target click, replace the selected ships' order list for `T`, or append the target without duplicates for `Shift+T`. A click on empty space leaves existing orders unchanged and exits the one-shot mode.
4. Cancel targeting on Escape or right-click. A target-mode click must not also select/deselect ships or begin a selection box.
5. Remove the debug dummy-target behavior and its setup entity once the real order path is working.

Only selected `Ship` entities receive orders. Resolve turrets from their hierarchy during acquisition rather than iterating only direct children of selected ships.

### Turret acquisition and engagement

Add a turret acquisition system in `src/weapons/systems/weapon_systems.rs`. For each turret:

1. Resolve its owning ship through `ChildOf` ancestors.
2. Walk the owner's ordered target list and select the first candidate that still exists, is not the owner, is within that turret's `max_range`, and is engageable under its firing-arc rule.
3. Store that choice in `TargetControl.current_target`. Clear the choice and lock state when none qualifies.

Define `firing_arc` as the total permitted angle in radians around the turret mount's forward direction. A full-circle value permits any bearing. Calculate bearing in world space, compare against the mount's world-space forward vector, and convert the desired world aim into the turret's local rotation before mutating `Transform`; turret transforms are local because the sprites are nested under rotating ship/cell entities. Avoid comparing a world-space angle directly to a local angle.

Keep range, arc eligibility, aiming, and firing consistent: a turret must not be marked locked or fire while outside its allowed arc/range. Line-of-sight checks are out of scope for this first version; projectile collision remains responsible for impacts.

### Predictive aiming

Query target `GlobalTransform` and optional `LinearVelocity`. Use the target's current position if it has no velocity or a valid intercept cannot be solved. In 2D, let `r = P_target - P_turret`, target velocity be `v`, and projectile speed be `s`. Solve:

```text
a = dot(v, v) - s*s
b = 2 * dot(r, v)
c = dot(r, r)
a*t*t + b*t + c = 0
```

Choose the smallest positive real solution. Handle the near-linear case (`a` approximately zero) separately; reject a negative discriminant, non-finite result, or roots that are not positive. The aim point is `P_target + v*t`. If the target is stationary, already coincident with the turret, or has no feasible intercept, use a safe current-position fallback and avoid normalizing a zero-length direction.

Use the same solved aim point to determine bearing and lock-on. Keep the solver as a small pure helper so its numerical cases can be unit tested independently from Bevy systems.

### Scheduling and module ownership

- `src/controls/components.rs` / `src/controls/systems.rs`: targeting mode/order input and ship picking.
- `src/controls/plugins.rs`: register the input behavior and resource/component initialization as needed.
- `src/weapons/components/weapon.rs`: ship order component and any small engagement-state additions.
- `src/weapons/systems/weapon_systems.rs`: owner resolution, stale-order pruning/acquisition, predictive aim, and arc/range-aware lock state.
- `src/weapons/plugins/weapon_plugin.rs`: run acquisition before aim, and aim before fire; make ordering relative to target-order input explicit where same-frame response is required.
- `src/main.rs`: remove dummy-target setup and `debug_targeting_system` after the replacement command is registered.

Do not add a `WeaponProfile` containing child weapon entities. Use hierarchy traversal, which is already the project's ownership mechanism.

## Implementation Sequence

1. **Ownership and order state:** add the ordered ship target component and a tested helper to resolve a weapon's owning ship. Update projectile source-ship resolution to use the same owner rule.
2. **Target command:** add `T` replace and `Shift+T` append behavior, deterministic target picking, cancel handling, and ensure targeting clicks do not also affect selection.
3. **Per-turret acquisition:** walk ordered targets, reject stale/self/out-of-range/out-of-arc candidates, and clear stale turret locks. Define local/world angle handling before changing aim transforms.
4. **Predictive aim and fire gating:** add the intercept solver, aim at predicted positions, and only permit firing when the chosen target is in range, in arc, and aligned.
5. **Remove debug path and validate:** delete dummy-target wiring; run tests and a manual two-ship scene check.

Implement and validate each step before proceeding to the next. Keep the debug input behavior until the real order path can assign and clear targets end-to-end.

## Validation

### Automated checks

- Unit-test intercepts for a stationary target, a target moving across the firing line, a target moving away faster than the projectile, no real positive solution, the near-linear coefficient case, and zero relative distance.
- Test target-order behavior for replace, append-without-duplicates, invalid/despawned target pruning, and owner exclusion.
- Run `cargo build` after integration. Run `cargo test` for the pure solver and order helpers.

### In-game checks

- Select one ship, press `T`, click the other ship, and verify all eligible turrets on the selected ship acquire it; the click must not change ship selection.
- Press `Shift+T` and click another ship; verify turrets that cannot engage the primary can choose the secondary, while eligible turrets continue to prefer the primary.
- Verify Escape/right-click cancels, empty-space clicks do not erase orders, and orders on unselected ships remain unchanged.
- Verify turrets respect range and firing arc, smoothly track a moving target ahead of its current position, and do not fire until aligned.
- Verify a ship never selects itself and its projectiles still identify their source ship correctly through the nested hierarchy.

## Scope and Open Decisions

- This plan assumes all non-own ships are valid targets until teams/factions are introduced.
- The current 100-unit center-distance click behavior is intentionally reused for targeting consistency. If precise hull picking is required, first add a shared picking abstraction or targetable ship bounds; no such API currently exists in the project.
- `firing_arc` currently has data but no established semantics beyond radians. The plan defines it as a total arc around the turret mount's forward axis; confirm that this matches intended ship designs before implementing arc constraints.
- This first version does not add target health/priority scoring, line-of-sight checks, automatic target search without player orders, target UI markers, or network/multiplayer authority.
- Ordered fallback targeting is included, but target command UX is constrained to one-shot replace/append commands. A persistent multi-click targeting mode can be added later if preferred.