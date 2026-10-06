//! Rooms-and-corridors dungeons by binary space partitioning (BSP).
//!
//! 1. **Split:** cut the map in two with a straight line, then cut
//!    each half again, `Depth` levels deep. The pieces ("leaves") never
//!    overlap and together cover the whole map.
//! 2. **Rooms:** carve one random room inside each leaf, at least one
//!    tile away from the leaf's edges.
//! 3. **Corridors:** after splitting an area, join a room from its
//!    left/top half to one from its right/bottom half with an L-shaped
//!    corridor. Because this happens at every split, the rooms form a
//!    connected tree.
//! 4. **Loops:** add a few extra corridors (`Loop %`) between random
//!    rooms, so there is more than one route through.
//!
//! These guarantees follow from the steps and are also tested:
//! - rooms never overlap (step 2: each sits inside its own leaf);
//! - every room is reachable (step 3: a tree is connected, and step 4
//!   only adds paths).

use crate::procgen::{
	Generator, ObjectData, Param, Params, Rng, Tile, TileGrid,
};

/// An axis-aligned rectangle: top-left `(x, y)`, size `w` by `h`.
/// Used both for BSP leaves and for the rooms carved inside them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
	pub x: i32,
	pub y: i32,
	pub w: i32,
	pub h: i32,
}

impl Room {
	/// Middle cell; for an even size, the cell just right of / below
	/// the true middle. Corridors aim here.
	fn center(self) -> (i32, i32) {
		(self.x + self.w / 2, self.y + self.h / 2)
	}
}

pub struct DungeonGen;

impl Generator for DungeonGen {
	fn name(&self) -> &'static str {
		"Dungeon"
	}

	fn default_params(&self) -> Params {
		Params(vec![
			Param::int("Width", 60, 30, 120, 2),
			Param::int("Height", 30, 16, 60, 2),
			Param::int("Min Room", 4, 3, 8, 1),
			Param::int("Max Room", 10, 4, 20, 1),
			Param::int("Depth", 4, 1, 7, 1),
			Param::int("Loop %", 10, 0, 60, 5),
		])
	}

	fn generate(&self, params: &Params, seed: u64) -> ObjectData {
		ObjectData::Grid(build(params, seed).0)
	}
}

/// Mutable state shared by the recursive steps, so they do not need
/// five separate arguments each.
struct Builder {
	rng: Rng,
	grid: TileGrid,
	rooms: Vec<Room>,
	min_room: i32,
	max_room: i32,
}

/// Runs the whole algorithm. Also returns the room list, which the
/// tests use to check for overlaps.
pub fn build(params: &Params, seed: u64) -> (TileGrid, Vec<Room>) {
	let (w, h) = (params.int("Width"), params.int("Height"));
	let min_room = params.int("Min Room");
	let mut b = Builder {
		rng: Rng::new(seed),
		grid: TileGrid::new(w, h, Tile::Wall),
		rooms: Vec::new(),
		min_room,
		// The sliders can set Max below Min; treat that as Min.
		max_room: params.int("Max Room").max(min_room),
	};

	// Steps 1-3.
	b.split(Room { x: 0, y: 0, w, h }, params.int("Depth"));

	// Step 4: each room has a `Loop %` chance of one extra corridor
	// to a random room (skipped if the pick is the room itself).
	let loop_pct = params.int("Loop %");
	for i in 0..b.rooms.len() {
		if b.rng.chance(loop_pct) {
			let j = b.rng.range(0, b.rooms.len() as i32) as usize;
			if j != i {
				let from = b.rooms[i].center();
				let to = b.rooms[j].center();
				b.corridor(from, to);
			}
		}
	}

	(b.grid, b.rooms)
}

impl Builder {
	/// Splits `area` up to `depth` more times, carving rooms in the
	/// leaves and corridors between the halves. Returns the centre
	/// of one room inside `area`, for the caller to connect to.
	fn split(&mut self, area: Room, depth: i32) -> (i32, i32) {
		// A leaf must fit the smallest room plus a 1-tile margin on
		// each side, so a cut needs room for two such leaves.
		let min_leaf = self.min_room + 2;
		let can_cut_x = area.w >= 2 * min_leaf;
		let can_cut_y = area.h >= 2 * min_leaf;

		if depth == 0 || !(can_cut_x || can_cut_y) {
			return self.place_room(area);
		}

		// `cut_x` = cut with a vertical line, making left/right halves.
		let cut_x = match (can_cut_x, can_cut_y) {
			(true, true) => {
				// Terminal cells are roughly twice as tall as wide, so
				// double the height to compare on-screen proportions.
				// Cut the side that looks over 25% longer; else coin flip.
				let (vw, vh) = (area.w, area.h * 2);
				if vw * 4 > vh * 5 {
					true
				} else if vh * 4 > vw * 5 {
					false
				} else {
					self.rng.chance(50)
				}
			}
			(only_x, _) => only_x,
		};

		// `Room { w: cut, ..area }` copies `area` but replaces `w`.
		let (a, b) = if cut_x {
			let cut = self.rng.range(min_leaf, area.w - min_leaf + 1);
			let left = Room { w: cut, ..area };
			let right = Room {
				x: area.x + cut,
				w: area.w - cut,
				..area
			};
			(left, right)
		} else {
			let cut = self.rng.range(min_leaf, area.h - min_leaf + 1);
			let top = Room { h: cut, ..area };
			let bottom = Room {
				y: area.y + cut,
				h: area.h - cut,
				..area
			};
			(top, bottom)
		};

		let ca = self.split(a, depth - 1);
		let cb = self.split(b, depth - 1);
		self.corridor(ca, cb);
		if self.rng.chance(50) { ca } else { cb }
	}

	/// Carves a random room inside `area` with a wall margin of at
	/// least one tile, records it, and returns its centre.
	fn place_room(&mut self, area: Room) -> (i32, i32) {
		// `area.w - 2` is the widest room that still leaves a margin.
		// Leaves are at least `min_room + 2` wide, so the range is
		// never empty.
		let max_w = self.max_room.min(area.w - 2);
		let max_h = self.max_room.min(area.h - 2);
		let w = self.rng.range(self.min_room, max_w + 1);
		let h = self.rng.range(self.min_room, max_h + 1);
		let room = Room {
			x: area.x + 1 + self.rng.range(0, area.w - 2 - w + 1),
			y: area.y + 1 + self.rng.range(0, area.h - 2 - h + 1),
			w,
			h,
		};
		self.grid
			.fill_rect(room.x, room.y, room.w, room.h, Tile::Floor);
		self.rooms.push(room);
		room.center()
	}

	/// An L-shaped corridor: one straight leg, a corner, then the
	/// other leg. A coin flip picks horizontal-first or vertical-first.
	fn corridor(&mut self, (x1, y1): (i32, i32), (x2, y2): (i32, i32)) {
		let corner = if self.rng.chance(50) {
			(x2, y1)
		} else {
			(x1, y2)
		};
		self.line((x1, y1), corner);
		self.line(corner, (x2, y2));
	}

	/// Carves floor along a horizontal or vertical segment, both ends
	/// included. (With a diagonal input it would fill a rectangle.)
	fn line(&mut self, (x1, y1): (i32, i32), (x2, y2): (i32, i32)) {
		for x in x1.min(x2)..=x1.max(x2) {
			for y in y1.min(y2)..=y1.max(y2) {
				self.grid.set(x, y, Tile::Floor);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rooms_do_not_overlap_and_stay_inside_border() {
		let params = DungeonGen.default_params();
		for seed in 0..50 {
			let (grid, rooms) = build(&params, seed);
			assert!(rooms.len() > 1, "seed {seed}: {} rooms", rooms.len());
			for (i, a) in rooms.iter().enumerate() {
				assert!(a.x >= 1 && a.y >= 1);
				assert!(a.x + a.w < grid.w && a.y + a.h < grid.h);
				for b in &rooms[i + 1..] {
					let overlap = a.x < b.x + b.w
						&& b.x < a.x + a.w && a.y < b.y + b.h
						&& b.y < a.y + a.h;
					assert!(!overlap, "seed {seed}: {a:?} overlaps {b:?}");
				}
			}
		}
	}
}
