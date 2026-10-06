# Extensible RPG Framework

A terminal app for generating RPG content (items, caves, dungeons,
towns and overworlds) from seeded, repeatable recipes. Built in Rust
with [ratatui](https://ratatui.rs) and
[crossterm](https://github.com/crossterm-rs/crossterm).

## Run it

Requires Rust 1.88 or newer (`rustc --version`).

```sh
cargo run                      # start the terminal UI
cargo run -- --dist            # package ./dist (see src/deploy/)
cargo run -- --install         # copy the binary to a per-user folder
cargo run -- --install-wezterm # write a WezTerm config, if none exists
```

## Keys

| key           | menu screen        | generator panel           |
|---------------|--------------------|---------------------------|
| Left / Right  | switch tab         | change selected slider    |
| Up / Down     | move through items | move between sliders      |
| Enter         | run selected item  | press Reroll or Spawn     |
| Q             | (nothing)          | back to the menu          |
| P / Space     | pause / resume     | pause / resume            |
| Esc           | quit               | quit                      |

## Develop

The same three checks run in CI (`.github/workflows/ci.yml`):

```sh
cargo fmt --check                          # formatting (80 columns)
cargo clippy --all-targets -- -D warnings  # lints, warnings = errors
cargo test                                 # unit tests
```

See the generator UI without a terminal (PowerShell):

```powershell
$env:SHOW_SCREEN=1; cargo test generator_screen -- --nocapture
```

## Read next

- [docs/WALKTHROUGH.md](docs/WALKTHROUGH.md): a guided tour of the
  code, bottom layer to top, with hands-on exercises.
- [docs/GENERATORS_PLAN.md](docs/GENERATORS_PLAN.md): the roadmap
  (Bevy ECS, data-driven assets).
- `cargo doc --open`: the code's doc comments as a browsable site.
  Add `--document-private-items` to include private items; this is a
  binary crate, so almost everything is private.
