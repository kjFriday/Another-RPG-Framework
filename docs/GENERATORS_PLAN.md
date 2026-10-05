# Plan: Generators, Sliders, and Bevy Asset Loading

## Where things stand

- `src/object.rs` contains a pseudo-ECS:
  - `EntityId` plays the role of an entity.
  - `ObjectKind` plays the role of a marker component.
  - `World` is a flat store with `spawn`, `get`, `of_kind` and `count`.
- `src/game_loop.rs`: menu items carry a `MenuAction` enum (`Spawn(kind)`, `List(kind)`, `TogglePause`, `Quit`, `Unimplemented`). `RpgState::apply` runs the action.
- The **Generate** tab already has an `Item / Map / Dungeon / Town / Overworld` button that spawns an empty object of that kind. Generators will put real content behind those buttons.

The plan has three phases. Each one ships on its own.

---

## Phase 1: Generators and parameter sliders (TUI only, no Bevy)

### 1.1 Generator trait

New file `src/gen/mod.rs`:

```rust
pub trait Generator {
    type Output;
    fn params(&self) -> &[Param];          // sliders shown in the UI
    fn params_mut(&mut self) -> &mut [Param];
    fn generate(&self, seed: u64) -> Self::Output;
}
```

- Each generator is deterministic for a given `(params, seed)`. That lets a result be saved as just its recipe instead of the full output.
- Use a seeded RNG: add `rand` + `rand_chacha` (`ChaCha8Rng::seed_from_u64`).

### 1.2 Parameters as data

```rust
pub enum ParamValue { Int { v: i32, min: i32, max: i32, step: i32 },
                      Float { v: f32, min: f32, max: f32, step: f32 },
                      Choice { idx: usize, options: &'static [&'static str] },
                      Toggle(bool) }
pub struct Param { pub label: &'static str, pub value: ParamValue }
```

The UI renders every variant the same way. Left/Right nudges the value, and a `LineGauge` (or the choice name) shows where it is. No widget code is specific to one generator.

### 1.3 The generators

| Kind | Output | Algorithm | Sliders |
|---|---|---|---|
| Item | `ItemData { name, rarity, slot, stats }` | Weighted affix tables | rarity bias, item level, slot |
| Map | `TileGrid` | Cellular automata caves | width, height, fill %, smoothing passes |
| Dungeon | `TileGrid` + room list | BSP split, then corridors | size, min/max room, depth, loop chance |
| Town | lots + roads on `TileGrid` | Main road + recursive lot subdivision | population, wall toggle, road density |
| Overworld | heightmap → biome grid | Value/Perlin noise (`noise` crate) + thresholds | size, sea level, roughness, moisture |

`TileGrid { w, h, tiles: Vec<Tile> }` with `enum Tile { Floor, Wall, Water, Road, Grass, Forest, Mountain, Door }`. All map-like generators share it.

### 1.4 Storing results

- Add a component-like enum so the world holds data as well as kinds:
  ```rust
  pub enum ObjectData { Item(ItemData), Grid(TileGrid), Town(TownData), Overworld(OverworldData) }
  ```
- `World::spawn_with(kind, data)`.
- Store `(generator id, params, seed)` next to each result for reproducibility and saving.

### 1.5 UI flow

1. On the Generate tab, Enter opens a **generator panel** in place of the log view. Add `MenuAction::OpenGenerator(ObjectKind)`.
2. The panel lists the sliders, and Up/Down/Left/Right edit them.
3. Two buttons at the bottom:
   - `[Generate]` builds a preview from the current params and seed.
   - `[Reroll]` builds a preview with a new seed.
4. `[Spawn]` commits the preview to the `World`. Esc closes the panel.
5. Grid previews render as coloured characters in a `Canvas` or a plain `Paragraph`, one character per tile.

This needs a small UI mode enum, `enum Screen { Main, Generator(ObjectKind) }`, in `RpgState`. `handle_key` dispatches on it.

---

## Phase 2: Move the object model onto `bevy_ecs`

Use only the ECS crate. This keeps the ratatui front end and avoids pulling in Bevy's renderer.

The mapping is mechanical because of the Phase 1 shapes:

| Now | Bevy |
|---|---|
| `EntityId` | `Entity` |
| `ObjectKind` | `#[derive(Component)]` enum (or one marker struct per kind) |
| `ObjectData` variants | separate components (`ItemData`, `TileGrid`, …) |
| `World` | `bevy_ecs::world::World` |
| `MenuAction` | an event/message type consumed by a system |
| `RpgState::tick` | a `Schedule` run once per tick |

The loop stays in `EngineLoop::run`, which calls `schedule.run(&mut world)` on each tick.

---

## Phase 3: Data-driven content via Bevy's asset loader

Goal: affix tables, tile palettes, town templates and generator presets live in `assets/` as RON files and hot-reload, so mods are just files.

### 3.1 Choose a host
- **Option A (recommended):** full `bevy` app with `MinimalPlugins + AssetPlugin`, and render the TUI through the `bevy_ratatui` crate. This gives asset hot-reload and the scheduler for free. `EngineLoop` gets replaced by Bevy's app runner.
- **Option B:** stay on `bevy_ecs` only and load RON yourself with `serde` + `ron`, with no hot-reload. Simpler, but this misses the point of the asset system.

### 3.2 Asset types
```rust
#[derive(Asset, TypePath, Deserialize)] struct AffixTable { prefixes: Vec<Affix>, suffixes: Vec<Affix> }
#[derive(Asset, TypePath, Deserialize)] struct GeneratorPreset { kind: String, params: Vec<(String, f32)> }
#[derive(Asset, TypePath, Deserialize)] struct TilePalette { tiles: Vec<(String, char, [u8; 3])> }
```
- Register them with the `bevy_common_assets` RON plugin (`RonAssetPlugin::<AffixTable>::new(&["affixes.ron"])`). Write a custom `AssetLoader` only if you need non-RON formats.
- Use `AssetServer::load` in a startup system, store the `Handle`s in a resource, and react to `AssetEvent::Modified` for hot-reload.

### 3.3 Layout
```
assets/
  items/affixes.ron
  presets/dungeon_small.ron, overworld_islands.ron
  palettes/default.ron
```
The Plugins tab's "Reload Mod Pipeline" item becomes a real action that reloads the whole `assets/` folder.

### 3.4 Version caveat
Bevy's `AssetLoader` signature and its event/message APIs change between releases. Pin one Bevy version, and pick `bevy_ratatui` / `bevy_common_assets` releases that match it in their compatibility tables before writing code.

---

## Order of work
1. `TileGrid` and the Dungeon generator. This is the most visual one and proves the preview path.
2. The `Param` sliders and the generator panel.
3. The other four generators.
4. `ObjectData` storage plus a save/load recipe (enables System → Save/Load State).
5. Port to `bevy_ecs`.
6. Bevy app with `bevy_ratatui` and RON assets.

## Verification per phase
- **Phase 1:**
  - Unit test that `generate(seed)` is deterministic: the same seed gives an identical grid.
  - Unit test the BSP rooms: they must not overlap, and every room must be reachable (flood fill).
  - Manual check: `cargo run`, then Generate → Dungeon, adjust the sliders, Reroll, Spawn. The header's object count should go up.
- **Phase 2:** the same manual flow, plus a test that spawns through the `World` and queries by component.
- **Phase 3:** edit `affixes.ron` while the app runs, and the next Item roll should use the new affixes.
