//! # Extensible RPG Framework
//!
//! A terminal app for generating RPG content (items, caves, dungeons,
//! towns and overworlds) from seeded, repeatable recipes. It draws
//! with `ratatui`, which talks to the terminal through `crossterm`.
//!
//! # Architecture
//!
//! The modules form layers. Each one uses only the layers below it,
//! so any layer can be read, tested and changed without the ones
//! above it:
//!
//! ```text
//!   main      entry point: CLI flag -> deploy, otherwise -> app
//!   app       owns the state, runs the loop, routes keys
//!   ui        draws frames; menu + generator panel state
//!   command   the verbs a menu entry can trigger
//!   sim       clock, pause, message log, the world
//!   world     object store: ids, kinds, generated content
//!   procgen   pure generators: settings + seed -> content
//!
//!   deploy    install/package tasks (used only by main)
//! ```
//!
//! Start reading at `docs/WALKTHROUGH.md`, which tours the code in
//! this order with exercises.

// Each `mod` line pulls in `src/<name>.rs` or `src/<name>/mod.rs`.
// Without it, the file is not part of the program at all.
mod app;
mod command;
mod deploy;
mod procgen;
mod sim;
mod ui;
mod world;

use std::{env, process::ExitCode};

/// With no arguments, starts the terminal UI. With a flag, runs one
/// deploy task instead and never touches the screen (see `deploy`).
/// Only the first argument is read; any others are ignored.
///
/// Returning `ExitCode` lets the shell and scripts see success (0)
/// or failure (1), e.g. via `$LASTEXITCODE` in PowerShell.
fn main() -> ExitCode {
	match env::args().nth(1) {
		Some(flag) => deploy::run_flag(&flag),
		None => match app::run() {
			Ok(()) => ExitCode::SUCCESS,
			Err(err) => {
				eprintln!("Engine loop error: {err:?}");
				ExitCode::FAILURE
			}
		},
	}
}
