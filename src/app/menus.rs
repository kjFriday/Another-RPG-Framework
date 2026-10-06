//! The menu's contents, written as data.
//!
//! To add an entry, add a `("Label", Command)` pair to a tab below.
//! To give it new behaviour, add a variant to `Command`
//! (`src/command.rs`) and handle it in `App::execute`
//! (`app/input.rs`); the compiler will point at every `match` that
//! needs the new case.

use crate::command::Command::{self, *};
use crate::ui::MenuSystem;
use crate::world::ObjectKind::*;

/// Shorthand for one entry.
type Entry = (&'static str, Command);

const WORLD: &[Entry] = &[
	("Maps", List(Map)),
	("Dungeons", List(Dungeon)),
	("Towns", List(Town)),
	("Overworlds", List(Overworld)),
	("Inspect Zone", Unimplemented),
	("Time Controls", TogglePause),
	("Fast Travel", Unimplemented),
];

const GENERATE: &[Entry] = &[
	("Item", OpenGenerator(Item)),
	("Map", OpenGenerator(Map)),
	("Dungeon", OpenGenerator(Dungeon)),
	("Town", OpenGenerator(Town)),
	("Overworld", OpenGenerator(Overworld)),
];

const ENTITIES: &[Entry] = &[
	("Items", List(Item)),
	("Party Members", List(PartyMember)),
	("Monsters", List(Monster)),
	("NPCs", List(Npc)),
	("Spawn Monster", Spawn(Monster)),
];

const PLUGINS: &[Entry] = &[
	("Loaded Crates", Unimplemented),
	("Lua Scripts", Unimplemented),
	("Reload Mod Pipeline", Unimplemented),
];

const SYSTEM: &[Entry] = &[
	("Save State", Unimplemented),
	("Load State", Unimplemented),
	("Engine Settings", Unimplemented),
	("Exit", Quit),
];

/// The menu the program starts with, tabs in display order.
pub fn main_menu() -> MenuSystem {
	MenuSystem::from_defs(&[
		("World", WORLD),
		("Generate", GENERATE),
		("Entities", ENTITIES),
		("Plugins", PLUGINS),
		("System", SYSTEM),
	])
}
