mod deploy;
mod game_loop;
mod object;

use deploy::Project;
use game_loop::{EngineLoop, MenuAction::*, RpgState, load_menus};
use object::ObjectKind;
use std::{env, io, process::ExitCode};

fn main() -> ExitCode {
	// Deployment subcommands run without starting the TUI
	if let Some(arg) = env::args().nth(1) {
		let result = match arg.as_str() {
			"--install" => deploy::deploy_local_bin(),
			"--dist" => deploy::deploy_to_dist(&Project {
				name: env!("CARGO_PKG_NAME").into(),
				version: env!("CARGO_PKG_VERSION").into(),
				description: "Extensible RPG Framework".into(),
			}),
			"--install-wezterm" => deploy::deploy_wezterm_config(),
			_ => {
				eprintln!("Unknown argument: {arg}");
				eprintln!("Usage: [--install | --dist | --install-wezterm]");
				return ExitCode::FAILURE;
			}
		};
		return match result {
			Ok(_) => ExitCode::SUCCESS,
			Err(e) => {
				eprintln!("Deployment failed: {e}");
				ExitCode::FAILURE
			}
		};
	}

	match run_tui() {
		Ok(()) => ExitCode::SUCCESS,
		Err(err) => {
			eprintln!("Engine loop error: {err:?}");
			ExitCode::FAILURE
		}
	}
}

fn run_tui() -> io::Result<()> {
	// ratatui::init enables raw mode + alternate screen and installs a panic hook that restores the terminal
	let mut terminal = ratatui::init();

	let mut state = RpgState::new("Extensible RPG Framework", "1.0.0");
	let engine = EngineLoop::new(30);

	// Initialize menu system
	let mut menu = load_menus(&[
		("World", &[
			("Map View", List(ObjectKind::Map)),
			("Inspect Zone", Unimplemented),
			("Time Controls", TogglePause),
			("Fast Travel", Unimplemented),
		]),
		("Generate", &[
			("Item", Spawn(ObjectKind::Item)),
			("Map", Spawn(ObjectKind::Map)),
			("Dungeon", Spawn(ObjectKind::Dungeon)),
			("Town", Spawn(ObjectKind::Town)),
			("Overworld", Spawn(ObjectKind::Overworld)),
		]),
		("Entities", &[
			("Party Members", List(ObjectKind::PartyMember)),
			("Monsters", List(ObjectKind::Monster)),
			("NPCs", List(ObjectKind::Npc)),
			("Spawn Monster", Spawn(ObjectKind::Monster)),
		]),
		("Plugins", &[
			("Loaded Crates", Unimplemented),
			("Lua Scripts", Unimplemented),
			("Reload Mod Pipeline", Unimplemented),
		]),
		("System", &[
			("Save State", Unimplemented),
			("Load State", Unimplemented),
			("Engine Settings", Unimplemented),
			("Exit", Quit),
		]),
	]);

	let result = engine.run(&mut terminal, &mut state, &mut menu);
	ratatui::restore();
	result
}
