//! The world: every object that exists, and the store that owns them.
//!
//! This is a stand-in for an ECS (entity-component-system) `World`.
//! The names are chosen to map one-to-one onto Bevy later:
//!
//! | here                 | Bevy                              |
//! |----------------------|-----------------------------------|
//! | [`EntityId`]         | `Entity`                          |
//! | [`ObjectKind`]       | a marker `Component`              |
//! | `GameObject` fields  | one `Component` each              |
//! | [`World`]            | `bevy_ecs::world::World`          |
//! | [`World::of_kind`]   | a `Query` filtered by kind        |

mod kind;
mod object;

pub use kind::ObjectKind;
pub use object::{EntityId, GameObject};

use crate::procgen::{ObjectData, Recipe};

/// Owns every [`GameObject`], in spawn order.
///
/// Lookups scan the whole `Vec`. That is O(n) per lookup but simple,
/// and plenty fast for the hundreds of objects a session creates.
#[derive(Debug, Default)]
pub struct World {
	/// Id for the next spawn. Ids are never reused, even if objects
	/// are removed later, so an old id can never point at a new object.
	next_id: u32,
	objects: Vec<GameObject>,
}

impl World {
	/// Adds an object with no generated content.
	pub fn spawn(&mut self, kind: ObjectKind) -> EntityId {
		self.insert(kind, None, None)
	}

	/// Adds generated content along with the recipe that made it.
	pub fn spawn_generated(
		&mut self,
		kind: ObjectKind,
		data: ObjectData,
		recipe: Recipe,
	) -> EntityId {
		self.insert(kind, Some(data), Some(recipe))
	}

	fn insert(
		&mut self,
		kind: ObjectKind,
		data: Option<ObjectData>,
		recipe: Option<Recipe>,
	) -> EntityId {
		let id = EntityId(self.next_id);
		self.next_id += 1;
		// Items carry their own rolled name; anything else is named
		// after its kind and id, e.g. "Dungeon #3".
		let name = match &data {
			Some(ObjectData::Item(item)) => item.name.clone(),
			_ => format!("{} #{}", kind.label(), id.0),
		};
		let object = GameObject {
			id,
			kind,
			name,
			data,
			recipe,
		};
		self.objects.push(object);
		id
	}

	pub fn get(&self, id: EntityId) -> Option<&GameObject> {
		self.objects.iter().find(|o| o.id == id)
	}

	/// All objects of one kind, oldest first. Lazy: nothing is
	/// collected until the caller iterates.
	pub fn of_kind(
		&self,
		kind: ObjectKind,
	) -> impl Iterator<Item = &GameObject> {
		// `move` copies `kind` into the closure, so the iterator can
		// outlive this function call.
		self.objects.iter().filter(move |o| o.kind == kind)
	}

	pub fn count(&self) -> usize {
		self.objects.len()
	}
}
