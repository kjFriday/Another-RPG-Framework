//! The simulation: the game-side state that exists with or without a
//! screen. It knows nothing about terminals, keys or menus, so it can
//! be driven by tests, scripts or a different front end unchanged.

mod log;

pub use log::Log;

use crate::world::World;

/// How many log messages are remembered. The log box shows as many
/// of the newest ones as fit on screen.
const LOG_CAPACITY: usize = 200;

/// Everything that is "the game" rather than "the program".
pub struct Simulation {
	pub name: String,
	pub version: String,
	/// Ticks elapsed while unpaused. A `u64` at 30 ticks per second
	/// would take billions of years to overflow.
	pub tick_count: u64,
	pub paused: bool,
	pub log: Log,
	pub world: World,
}

impl Simulation {
	/// `impl Into<String>` lets callers pass `&str` or `String`.
	pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
		let mut sim = Self {
			name: name.into(),
			version: version.into(),
			tick_count: 0,
			paused: false,
			log: Log::with_capacity(LOG_CAPACITY),
			world: World::default(),
		};
		sim.log.push("Simulation initialized.");
		sim
	}

	/// Advances time by one step. Paused simulations do not advance.
	/// The app calls this once per tick interval (see `app::App`).
	pub fn tick(&mut self) {
		if !self.paused {
			self.tick_count += 1;
		}
	}

	pub fn toggle_pause(&mut self) {
		self.paused = !self.paused;
		let status = if self.paused { "paused" } else { "resumed" };
		self.log.push(format!("Simulation {status}."));
	}
}
