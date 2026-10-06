//! Natural-looking caves from a cellular automaton.
//!
//! 1. **Noise:** each interior cell becomes wall with probability
//!    `Fill %`, otherwise floor. The outer border is always wall.
//! 2. **Smoothing:** repeat `Smoothing` times. A cell becomes wall if
//!    5 or more of the 9 cells in its 3x3 block (itself included) are
//!    walls, otherwise floor. Lone specks vanish, and walls clump into
//!    rounded shapes. Roguelike writers often call this the "4-5 rule".
//! 3. **Cleanup:** keep only the largest connected open area, so every
//!    floor tile can reach every other.

use crate::procgen::{
	Generator, ObjectData, Param, Params, Rng, Tile, TileGrid,
};

pub struct CaveGen;

impl Generator for CaveGen {
	fn name(&self) -> &'static str {
		"Cave Map"
	}

	fn default_params(&self) -> Params {
		Params(vec![
			Param::int("Width", 60, 30, 120, 2),
			Param::int("Height", 30, 16, 60, 2),
			Param::int("Fill %", 45, 30, 65, 1),
			Param::int("Smoothing", 4, 0, 8, 1),
		])
	}

	fn generate(&self, params: &Params, seed: u64) -> ObjectData {
		let mut rng = Rng::new(seed);
		let (w, h) = (params.int("Width"), params.int("Height"));
		let fill = params.int("Fill %");

		// Step 1: random fill. Loops cover 1..w-1 and 1..h-1, so the
		// border cells keep their initial Wall.
		let mut grid = TileGrid::new(w, h, Tile::Wall);
		for y in 1..h - 1 {
			for x in 1..w - 1 {
				if !rng.chance(fill) {
					grid.set(x, y, Tile::Floor);
				}
			}
		}

		// Step 2: smoothing. Each pass reads from `prev`, a snapshot,
		// so every cell is judged on the same generation of the map
		// rather than on neighbours already changed in this pass.
		for _ in 0..params.int("Smoothing") {
			let prev = grid.clone();
			for y in 1..h - 1 {
				for x in 1..w - 1 {
					let walls = (-1..=1)
						.flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
						.filter(|&(dx, dy)| {
							prev.get(x + dx, y + dy) == Tile::Wall
						})
						.count();
					let tile =
						if walls >= 5 { Tile::Wall } else { Tile::Floor };
					grid.set(x, y, tile);
				}
			}
		}

		// Step 3: one connected cave.
		grid.retain_largest_region(Tile::Wall);
		ObjectData::Grid(grid)
	}
}
