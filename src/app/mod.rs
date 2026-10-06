//! The running program: owns all state and drives the main loop.
//!
//! # One pass of the loop
//!
//! ```text
//!   +--> draw a frame      (ui::render reads the state)
//!   |    wait for a key    (at most until the next tick is due)
//!   |    handle the key    (input.rs changes the state)
//!   |    tick if it's time (Simulation::tick)
//!   +------ repeat until `quit` is set
//! ```
//!
//! # Files
//!
//! - `mod.rs` (this file): [`App`], the loop, terminal setup.
//! - `input.rs`: keys to actions; the only code that changes state.
//! - `menus.rs`: the menu's contents, written as data.

mod input;
mod menus;

use crate::sim::Simulation;
use crate::ui::{self, GeneratorPanel, MenuSystem};
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::Backend};
use std::io;
use std::time::{Duration, Instant};

/// Simulation steps per second. Also the most frames per second the
/// loop draws while idle; key presses add extra frames.
const TICKS_PER_SECOND: u64 = 30;

/// Starts the terminal UI and blocks until the user quits.
pub fn run() -> io::Result<()> {
	// `ratatui::init` switches the terminal into "raw mode" (keys
	// arrive one at a time, unechoed) and the "alternate screen" (a
	// blank page that disappears on exit, leaving your shell history
	// untouched). It also installs a panic hook that undoes both
	// before a panic message prints, so a crash cannot leave the
	// terminal unusable. It panics if setup fails; `ratatui::try_init`
	// is the variant that returns a `Result` instead.
	let mut terminal = ratatui::init();

	let sim = Simulation::new("Extensible RPG Framework", "1.0.0");
	let mut app = App::new(sim, menus::main_menu(), TICKS_PER_SECOND);
	let result = app.run(&mut terminal);

	// Undo raw mode and the alternate screen on the normal exit path.
	ratatui::restore();
	result
}

/// Which view has the keyboard and the main area of the screen.
pub enum Screen {
	Main,
	/// Boxed because a panel is much larger than `Main`; boxing keeps
	/// the enum itself small (clippy's `large_enum_variant` lint).
	Generator(Box<GeneratorPanel>),
}

/// All state for one run of the program.
pub struct App {
	sim: Simulation,
	menu: MenuSystem,
	screen: Screen,
	/// Set by the Quit command or the Esc key; ends the loop.
	quit: bool,
	tick_rate: Duration,
}

impl App {
	/// `ticks_per_second` of 0 is treated as 1, avoiding a division by
	/// zero. The interval is whole milliseconds, rounded down: 30 per
	/// second gives 33 ms, which is about 30.3 ticks per second.
	pub fn new(
		sim: Simulation,
		menu: MenuSystem,
		ticks_per_second: u64,
	) -> Self {
		let tick_ms = 1000 / ticks_per_second.max(1);
		Self {
			sim,
			menu,
			screen: Screen::Main,
			quit: false,
			tick_rate: Duration::from_millis(tick_ms),
		}
	}

	/// The open generator panel, if any.
	fn panel(&self) -> Option<&GeneratorPanel> {
		match &self.screen {
			// `&**p` goes from &Box<GeneratorPanel> to &GeneratorPanel.
			Screen::Generator(p) => Some(&**p),
			Screen::Main => None,
		}
	}

	/// The main loop. Generic over the ratatui `Backend`, so tests
	/// can pass an in-memory `TestBackend` instead of a real terminal.
	pub fn run<B: Backend>(
		&mut self,
		terminal: &mut Terminal<B>,
	) -> io::Result<()> {
		let mut last_tick = Instant::now();

		while !self.quit {
			terminal
				.draw(|frame| {
					ui::render(frame, &self.sim, &self.menu, self.panel())
				})
				// The backend's error type varies by backend; turn it
				// into an `io::Error` so this function has one type.
				.map_err(|err| io::Error::other(err.to_string()))?;

			// Wait for input, but no longer than the time left until
			// the next tick. `saturating_sub` gives zero, rather than
			// panicking, if that time has already passed.
			let timeout = self.tick_rate.saturating_sub(last_tick.elapsed());
			if event::poll(timeout)? {
				// On Windows, crossterm reports key releases as well
				// as presses. Acting only on `Press` stops every key
				// from firing twice.
				if let Event::Key(key) = event::read()?
					&& key.kind == KeyEventKind::Press
				{
					self.handle_key(key.code);
				}
			}

			// Ticks that were missed (e.g. while the terminal was
			// busy) are skipped, not replayed: at most one per pass.
			if last_tick.elapsed() >= self.tick_rate {
				self.sim.tick();
				last_tick = Instant::now();
			}
		}

		Ok(())
	}
}
