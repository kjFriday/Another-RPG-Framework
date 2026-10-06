# Walkthrough: learn this codebase by changing it

A crash course through the whole program, bottom layer to top. Each
stop names the files to read, the one idea to take away, and an
exercise. Every exercise ends with a **Check**: a command whose
result proves you got it right. Hints are folded under "Solution";
try first.

Work on a branch so you can always get back:

```sh
git switch -c walkthrough
```

---

## Stop 0: run everything

```sh
cargo run      # Esc quits
cargo test     # all tests should pass
cargo doc --open --document-private-items
```

The last command builds a website from the `///` and `//!` comments.
Every module page starts with that module's overview.

**Idea:** the program is layered. Read the module map at the top of
`src/main.rs` now; the rest of this walkthrough follows it upward.

```text
main -> app -> ui -> command -> sim -> world -> procgen
```

A layer may use anything to its right, never to its left. That is why
you can test `procgen` without a terminal, and why `ui` cannot change
the world even by accident.

---

## Stop 1: randomness you can replay (`src/procgen/rng.rs`)

**Idea:** a seed fully determines every number that follows. Same
seed, same dungeon, forever. That is what makes a `Recipe` (settings
plus seed) enough to rebuild content.

**Exercise:** add a test to `rng.rs` proving that `chance(0)` is
never true and `chance(100)` is always true, over 1000 calls each.

**Check:** `cargo test rng`

<details><summary>Solution</summary>

```rust
#[test]
fn chance_extremes() {
	let mut rng = Rng::new(1);
	for _ in 0..1000 {
		assert!(!rng.chance(0));
		assert!(rng.chance(100));
	}
}
```

</details>

---

## Stop 2: settings as data (`src/procgen/params.rs`)

**Idea:** generators never draw UI. They return a list of `Param`s,
and one piece of UI code draws sliders for any list. Data in, UI out.

**Exercise:** in `generators/cave.rs`, add a `Param::toggle("Lakes",
false)`. When on, turn every floor tile with at least 6 floor
neighbours (out of 8) into `Tile::Water`. Then run the app and flip
it in Generate -> Map.

**Check:** the slider appears with no UI changes, and
`cargo test` still passes. (Does `grid_generators_produce_one_
connected_region` still hold with lakes on? Water blocks walking.
The test uses default params, so it does not notice. Think about why
that is a gap, not a pass.)

---

## Stop 3: let the compiler find your work (`src/procgen/grid.rs`)

**Idea:** Rust's `match` must cover every enum variant. Add a variant
and the compiler lists every place that needs a decision.

**Exercise:** add `Tile::Lava`. Do not search for anything first;
run `cargo build` and fix each error it reports.

**Check:** `cargo build` is clean. The compiler reports exactly two
places: `Tile::glyph` and `ui::theme::tile`. It says nothing about
`Tile::walkable`, which uses `matches!`: that macro has a built-in
catch-all, so Lava silently counts as walkable unless you add it
there yourself. Lesson: a `_` arm or `matches!` switches the
compiler's help off, so prefer a full `match` wherever a new case
deserves a decision.

---

## Stop 4: a generator, end to end (`src/procgen/generators/`)

**Idea:** a feature that touches every layer. This is the "full
stack" of this program.

**Exercise:** add an **Arena** generator: one big open room with
pillars every 4 tiles. Then wire it through every layer:

1. `procgen/generators/arena.rs`: `ArenaGen` implementing
   `Generator`, sliders `Width`, `Height`, `Pillar Gap`.
2. `procgen/generators/mod.rs` and `procgen/mod.rs`: re-export it,
   and add it to `tests::all`.
3. `world/kind.rs`: add `ObjectKind::Arena`. The compiler will send
   you to `label` and `generator`.
4. `app/menus.rs`: add `("Arena", OpenGenerator(Arena))` to
   `GENERATE` and `("Arenas", List(Arena))` to `WORLD`.

**Check:** `cargo test` passes (the determinism test now covers your
generator), and in the app Generate -> Arena -> Spawn, then World ->
Arenas lists it.

---

## Stop 5: the world store (`src/world/`)

**Idea:** `EntityId` is a newtype. The compiler will not mix it up
with any other `u32`. Ids are never reused.

**Exercise:** add `World::remove(&mut self, id: EntityId) ->
Option<GameObject>`. Write a test: spawn A, remove A, spawn B, and
assert B's id differs from A's.

**Check:** `cargo test world`

<details><summary>Solution</summary>

```rust
pub fn remove(&mut self, id: EntityId) -> Option<GameObject> {
	let index = self.objects.iter().position(|o| o.id == id)?;
	Some(self.objects.remove(index))
}
```

</details>

---

## Stop 6: game state without a screen (`src/sim/`)

**Idea:** `Simulation` has no idea a terminal exists. Anything you
can do from a key, a test can do by calling a method.

**Exercise:** add `Log::clear`, then a `Command::ClearLog` with a
"Clear Log" entry in the System tab. The compiler will point you to
`App::execute` for the new variant.

**Check:** run the app, press a few buttons, then System -> Clear
Log. The log should be completely empty, even though pressing Enter
logged `> Clear Log` first (why? read the order of `log.push` and
`execute` in `app/input.rs`, then decide whether that is what a
user would expect).

---

## Stop 7: drawing (`src/ui/`)

**Idea:** ratatui is immediate-mode. Every frame describes the whole
screen from scratch; nothing drawn is remembered. So the UI is a
pure function of state: change state, and the next frame shows it.

**Exercise:** make the log box 44 columns wide, and colour log lines
that start with `Spawned` green. Use a new function in `theme.rs`
rather than a colour inline.

**Check:** `$env:SHOW_SCREEN=1; cargo test generator_screen --
--nocapture` prints the frame; measure the log column.

---

## Stop 8: input and the loop (`src/app/`)

**Idea:** `app/input.rs` is the only code that changes state. To
know what a key can do, read one file.

**Exercise:** add a global `F1` key that toggles a help line in the
log listing every key. Follow the existing priority order in
`handle_key`: decide whether F1 should work inside the panel too.

**Check:** extend `generator_screen_renders_and_spawns` (or write a
sibling test) that presses `KeyCode::F(1)` and asserts the log grew.

---

## Stop 9: shipping (`src/deploy/`, `Cargo.toml`)

**Idea:** a release build is optimised and has no debug checks.

```sh
cargo build --release
cargo run --release -- --dist
```

**Exercise:** open `dist/project_info.toml`. Where did each value
come from? (Hint: `env!` in `deploy/dist.rs`, and `Cargo.toml`.)
Bump `version` in `Cargo.toml` and rebuild `dist`.

**Check:** the new version shows in `dist/project_info.toml`.

---

## Stop 10: working like a team

The habits that keep a shared codebase healthy, all already wired up
here:

| habit                    | how, in this repo                          |
|--------------------------|--------------------------------------------|
| one code style           | `rustfmt.toml`; run `cargo fmt`            |
| no ignored warnings      | `cargo clippy --all-targets -- -D warnings`|
| proof it works           | `cargo test`; tests sit next to the code   |
| checks nobody can forget | `.github/workflows/ci.yml` runs all three  |
| small reviewable changes | one branch and one commit per stop above   |
| docs that cannot drift   | doc comments live beside what they explain |

**Exercise:** commit each stop separately with a message saying
*why*, not just what:

```sh
git add -A
git commit -m "Add Arena generator to test the full add-a-kind path"
```

**Check:** `git log --oneline` reads like a changelog of what you
learned.
