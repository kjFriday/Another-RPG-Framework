//! [`ObjectKind`]: the "what is it?" tag on every object.

use crate::procgen::{
	CaveGen, DungeonGen, Generator, ItemGen, OverworldGen, TownGen,
};

/// Every kind of thing the world can hold.
///
/// In ECS terms this plays the role of a *marker component*: data-free
/// except for the tag itself, used to find objects ("all dungeons")
/// and to decide behaviour ("which generator builds these?").
///
/// It is `Copy`, so passing it around never moves or clones heap data.
/// It is `Hash + Eq`, so it can key a `HashMap` later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKind {
	Item,
	Map,
	Dungeon,
	Town,
	Overworld,
	PartyMember,
	Monster,
	Npc,
}

impl ObjectKind {
	/// Human-readable name, used in log messages and default names.
	pub fn label(self) -> &'static str {
		match self {
			ObjectKind::Item => "Item",
			ObjectKind::Map => "Map",
			ObjectKind::Dungeon => "Dungeon",
			ObjectKind::Town => "Town",
			ObjectKind::Overworld => "Overworld",
			ObjectKind::PartyMember => "Party Member",
			ObjectKind::Monster => "Monster",
			ObjectKind::Npc => "NPC",
		}
	}

	/// The generator that builds this kind, if one exists.
	///
	/// This `match` is the registry: the only place that ties kinds
	/// to generators. It lives here (in `world`) rather than in
	/// `procgen` so that `procgen` never depends on `world`, keeping
	/// the layers one-directional. Because `match` must be exhaustive,
	/// adding a new `ObjectKind` will not compile until you decide
	/// here whether it gets a generator.
	pub fn generator(self) -> Option<Box<dyn Generator>> {
		match self {
			ObjectKind::Item => Some(Box::new(ItemGen)),
			ObjectKind::Map => Some(Box::new(CaveGen)),
			ObjectKind::Dungeon => Some(Box::new(DungeonGen)),
			ObjectKind::Town => Some(Box::new(TownGen)),
			ObjectKind::Overworld => Some(Box::new(OverworldGen)),
			ObjectKind::PartyMember | ObjectKind::Monster | ObjectKind::Npc => {
				None
			}
		}
	}
}
