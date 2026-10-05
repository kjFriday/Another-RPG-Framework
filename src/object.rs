//! Pseudo-ECS object model. `EntityId` stands in for an ECS entity handle and
//! `ObjectKind` for a marker component, so these map 1:1 onto Bevy later.

/// Opaque handle for a spawned object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u32);

/// Every kind of thing the world can hold.
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
}

#[derive(Debug, Clone)]
pub struct GameObject {
	pub id: EntityId,
	pub kind: ObjectKind,
	pub name: String,
}

/// Flat object store. Plays the role of an ECS `World` until Bevy is wired in.
#[derive(Debug, Default)]
pub struct World {
	next_id: u32,
	objects: Vec<GameObject>,
}

impl World {
	pub fn spawn(&mut self, kind: ObjectKind) -> EntityId {
		let id = EntityId(self.next_id);
		self.next_id += 1;
		self.objects.push(GameObject {
			id,
			kind,
			name: format!("{} #{}", kind.label(), id.0),
		});
		id
	}

	pub fn get(&self, id: EntityId) -> Option<&GameObject> {
		self.objects.iter().find(|o| o.id == id)
	}

	pub fn of_kind(&self, kind: ObjectKind) -> impl Iterator<Item = &GameObject> {
		self.objects.iter().filter(move |o| o.kind == kind)
	}

	pub fn count(&self) -> usize {
		self.objects.len()
	}
}
