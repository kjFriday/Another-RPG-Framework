//! Turning key presses into changes. This is the only file that
//! changes [`App`] state, which makes "what can change what" easy to
//! audit: read this file.
//!
//! # Key routing, in priority order
//!
//! 1. Global keys, which work on every screen: Esc quits, and P or
//!    Space pause.
//! 2. If the generator panel is open, every other key goes to it.
//! 3. Otherwise the keys drive the menu.

use super::{App, Screen};
use crate::command::Command;
use crate::procgen::mix;
use crate::ui::{GeneratorPanel, PanelEvent};
use crate::world::EntityId;
use crossterm::event::KeyCode;
use std::time::{SystemTime, UNIX_EPOCH};

impl App {
	pub(super) fn handle_key(&mut self, code: KeyCode) {
		// 1. Global keys.
		match code {
			KeyCode::Esc => {
				self.quit = true;
				return;
			}
			KeyCode::Char('p' | 'P' | ' ') => {
				self.sim.toggle_pause();
				return;
			}
			_ => {}
		}

		// 2. Generator panel. The `&mut` borrow of `self.screen` ends
		// when `handle_key` returns its event, so the match arms are
		// free to change `self.screen` and `self.sim`.
		if let Screen::Generator(panel) = &mut self.screen {
			match panel.handle_key(code) {
				PanelEvent::None => {}
				PanelEvent::Log(msg) => self.sim.log.push(msg),
				PanelEvent::Close => {
					self.sim.log.push("> Back");
					self.screen = Screen::Main;
				}
				PanelEvent::Spawn { kind, data, recipe } => {
					self.sim.log.push("> Spawn");
					let id = self.sim.world.spawn_generated(kind, data, recipe);
					self.log_spawned(id);
				}
			}
			return;
		}

		// 3. Menu.
		match code {
			KeyCode::Left => self.menu.previous_category(),
			KeyCode::Right => self.menu.next_category(),
			KeyCode::Up => self.menu.previous_item(),
			KeyCode::Down => self.menu.next_item(),
			KeyCode::Enter => {
				// Copy the two fields out first: `selected()` borrows
				// `self.menu`, which `execute` would conflict with.
				if let Some(item) = self.menu.selected() {
					let (label, command) = (item.label.clone(), item.command);
					self.sim.log.push(format!("> {label}"));
					self.execute(command);
				}
			}
			_ => {}
		}
	}

	/// Carries out a menu command.
	fn execute(&mut self, command: Command) {
		match command {
			Command::Spawn(kind) => {
				let id = self.sim.world.spawn(kind);
				self.log_spawned(id);
			}
			Command::OpenGenerator(kind) => {
				let seed = fresh_seed(self.sim.tick_count);
				match GeneratorPanel::open(kind, seed) {
					Some(panel) => {
						self.screen = Screen::Generator(Box::new(panel));
					}
					None => self
						.sim
						.log
						.push(format!("No generator for {}.", kind.label())),
				}
			}
			Command::List(kind) => {
				// Collect first: the iterator borrows `world`, and
				// pushing to `log` while it is alive would need a
				// second borrow of `self.sim` at the same time.
				let lines: Vec<String> =
					self.sim.world.of_kind(kind).map(|o| o.summary()).collect();
				if lines.is_empty() {
					let msg = format!("No {} objects.", kind.label());
					self.sim.log.push(msg);
				}
				for line in lines {
					self.sim.log.push(line);
				}
			}
			Command::TogglePause => self.sim.toggle_pause(),
			Command::Quit => self.quit = true,
			Command::Unimplemented => self.sim.log.push("Not implemented yet."),
		}
	}

	fn log_spawned(&mut self, id: EntityId) {
		if let Some(object) = self.sim.world.get(id) {
			let msg = format!("Spawned {}.", object.summary());
			self.sim.log.push(msg);
		}
	}
}

/// A starting seed for a newly opened generator panel.
///
/// Mixes the wall clock (nanoseconds since 1970) with the tick count,
/// so two panels opened in the same session get different seeds.
/// Falls back to 0 for the clock if the system time is before 1970.
fn fresh_seed(tick: u64) -> u64 {
	let nanos = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map_or(0, |d| d.as_nanos() as u64);
	mix(nanos ^ tick)
}

#[cfg(test)]
mod tests {
	use crate::app::{App, Screen};
	use crate::command::Command;
	use crate::sim::Simulation;
	use crate::ui::{self, MenuSystem};
	use crate::world::ObjectKind;
	use crossterm::event::KeyCode;
	use ratatui::{Terminal, backend::TestBackend};

	const WIDTH: u16 = 120;

	/// Drives the generator panel exactly as a user would, and draws
	/// it into an in-memory terminal. Run with `SHOW_SCREEN=1` and
	/// `-- --nocapture` to print the rendered frame.
	#[test]
	fn generator_screen_renders_and_spawns() {
		let open_dungeon = Command::OpenGenerator(ObjectKind::Dungeon);
		let menu = MenuSystem::from_defs(&[(
			"Generate",
			&[("Dungeon", open_dungeon)],
		)]);
		let mut app = App::new(Simulation::new("Test", "0"), menu, 30);

		app.handle_key(KeyCode::Enter);
		assert!(matches!(app.screen, Screen::Generator(_)));

		let mut terminal = Terminal::new(TestBackend::new(WIDTH, 40)).unwrap();
		terminal
			.draw(|f| ui::render(f, &app.sim, &app.menu, app.panel()))
			.unwrap();
		let cells = terminal.backend().buffer().content();
		let screen: String = cells.iter().map(|c| c.symbol()).collect();
		assert!(screen.contains("Dungeon Generator"));
		if std::env::var_os("SHOW_SCREEN").is_some() {
			let chars: Vec<char> = screen.chars().collect();
			for row in chars.chunks(WIDTH as usize) {
				println!("{}", row.iter().collect::<String>());
			}
		}

		// Cursor: 6 sliders, then Reroll, then Spawn. 7 downs = Spawn.
		for _ in 0..7 {
			app.handle_key(KeyCode::Down);
		}
		app.handle_key(KeyCode::Enter);
		assert_eq!(app.sim.world.count(), 1);

		app.handle_key(KeyCode::Char('q'));
		assert!(matches!(app.screen, Screen::Main));

		app.handle_key(KeyCode::Esc);
		assert!(app.quit);
	}
}
