//! [`EntityId`] and [`GameObject`]: one thing in the world.

use super::ObjectKind;
use crate::procgen::{ObjectData, Recipe};

/// A handle to a spawned object, in ECS terms an *entity*.
///
/// This is the "newtype" pattern: a one-field tuple struct wrapping a
/// `u32`. At run time it is just the number, but the compiler will not
/// let you pass a plain `u32` (say, a tick count) where an `EntityId`
/// is expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u32);

/// One object and everything known about it.
///
/// Each field corresponds to a future ECS *component*: in Phase 2 of
/// `docs/GENERATORS_PLAN.md` each becomes its own component type on a
/// Bevy entity.
#[derive(Debug, Clone)]
pub struct GameObject {
	pub id: EntityId,
	pub kind: ObjectKind,
	pub name: String,
	/// Generated content; `None` for objects spawned without one.
	pub data: Option<ObjectData>,
	/// How `data` was made; `None` exactly when `data` is `None`.
	pub recipe: Option<Recipe>,
}

impl GameObject {
	/// One line for the log, e.g.
	/// `Dungeon #3 60x30 [Dungeon seed 2eb8adac96edf083]`.
	pub fn summary(&self) -> String {
		let detail = match &self.data {
			Some(ObjectData::Grid(g)) => format!(" {}x{}", g.w, g.h),
			Some(ObjectData::Item(item)) => {
				format!(" ({:?} lvl {})", item.rarity, item.level)
			}
			None => String::new(),
		};
		// `{:x}` prints the seed in hexadecimal, matching the panel.
		let origin = match &self.recipe {
			Some(r) => format!(" [{} seed {:x}]", r.generator, r.seed),
			None => String::new(),
		};
		format!("{}{detail}{origin}", self.name)
	}
}
