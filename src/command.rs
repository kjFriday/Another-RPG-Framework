//! [`Command`]: the verbs a menu entry can trigger.
//!
//! A command is plain data. Menu entries carry one (`app::menus`),
//! and `App::execute` (`app/input.rs`) decides what it does. Because
//! commands are an enum and not closures, each one is `Copy`,
//! printable with `{:?}`, comparable in tests, and could later be
//! loaded from a data file.

use crate::world::ObjectKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
	/// Add an empty object of this kind to the world (no generator).
	Spawn(ObjectKind),
	/// Open the slider panel for this kind's generator.
	OpenGenerator(ObjectKind),
	/// Log a one-line summary of every object of this kind.
	List(ObjectKind),
	/// Pause or resume the simulation clock.
	TogglePause,
	/// Exit the program.
	Quit,
	/// Placeholder for menu entries with no behaviour yet.
	Unimplemented,
}
