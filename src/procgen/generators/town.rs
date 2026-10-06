//! Towns from recursive road subdivision.
//!
//! 1. Start with one grass block (inside an optional town wall).
//! 2. While a block is bigger than `Road Spacing`, run a one-tile road
//!    across it, splitting it in two, and repeat on each half.
//! 3. Each block too small to split is a lot. With probability
//!    `Density` it gets a walled building with a door on its south
//!    side.
//! 4. Where a road touches the town wall, open a gate.
//!
//! Every lot keeps grass on its left, right and bottom edges, so you
//! can walk around each building and reach its door from a road. The
//! test `grid_generators_produce_one_connected_region` checks this.

use crate::procgen::{
	Generator, ObjectData, Param, Params, Rng, Tile, TileGrid,
};

pub struct TownGen;

impl Generator for TownGen {
	fn name(&self) -> &'static str {
		"Town"
	}

	fn default_params(&self) -> Params {
		Params(vec![
			Param::int("Width", 50, 30, 100, 2),
			Param::int("Height", 26, 16, 50, 2),
			Param::int("Road Spacing", 10, 6, 20, 1),
			Param::int("Density", 70, 0, 100, 5),
			Param::toggle("Walls", true),
		])
	}

	fn generate(&self, params: &Params, seed: u64) -> ObjectData {
		let (w, h) = (params.int("Width"), params.int("Height"));
		let mut town = Town {
			rng: Rng::new(seed),
			grid: TileGrid::new(w, h, Tile::Grass),
			spacing: params.int("Road Spacing"),
			density: params.int("Density"),
		};

		if params.toggle("Walls") {
			// A 1-tile ring of wall, with the grass block inside it.
			town.grid.fill_rect(0, 0, w, h, Tile::Wall);
			town.grid.fill_rect(1, 1, w - 2, h - 2, Tile::Grass);
			town.subdivide(1, 1, w - 2, h - 2);
			town.open_gates();
		} else {
			town.subdivide(0, 0, w, h);
		}

		ObjectData::Grid(town.grid)
	}
}

struct Town {
	rng: Rng,
	grid: TileGrid,
	spacing: i32,
	density: i32,
}

impl Town {
	/// Steps 2 and 3 for the block with top-left `(x, y)`, size `w`
	/// by `h`.
	fn subdivide(&mut self, x: i32, y: i32, w: i32, h: i32) {
		// Rows are cut at half the spacing because terminal cells are
		// about twice as tall as wide. The `+ 2` and the ranges below
		// guarantee each half is non-empty.
		let can_cut_x = w > self.spacing;
		let can_cut_y = h >= self.spacing / 2 + 2;

		let cut_x = match (can_cut_x, can_cut_y) {
			(false, false) => return self.build_on_lot(x, y, w, h),
			// Cut across the side that looks longer on screen.
			(true, true) => w >= h * 2,
			(only_x, _) => only_x,
		};

		if cut_x {
			// A vertical road at column `at`, leaving >= 3 columns on
			// the left (`w > spacing >= 6` keeps the range non-empty).
			let at = x + self.rng.range(3, w - 3);
			self.grid.fill_rect(at, y, 1, h, Tile::Road);
			self.subdivide(x, y, at - x, h);
			self.subdivide(at + 1, y, x + w - at - 1, h);
		} else {
			// A horizontal road at row `at`, leaving >= 2 rows above.
			let at = y + self.rng.range(2, h - 2);
			self.grid.fill_rect(x, at, w, 1, Tile::Road);
			self.subdivide(x, y, w, at - y);
			self.subdivide(x, at + 1, w, y + h - at - 1);
		}
	}

	/// Step 3. The building spans the lot minus one column on each
	/// side and the bottom row, which stay grass as a walkway. The
	/// door sits mid-way along the bottom wall, facing that walkway.
	fn build_on_lot(&mut self, x: i32, y: i32, w: i32, h: i32) {
		let (bx, bw, bh) = (x + 1, w - 2, h - 1);
		// 3x3 is the smallest building with a floor tile inside.
		if bw < 3 || bh < 3 || !self.rng.chance(self.density) {
			return;
		}
		self.grid.fill_rect(bx, y, bw, bh, Tile::Wall);
		self.grid
			.fill_rect(bx + 1, y + 1, bw - 2, bh - 2, Tile::Floor);
		self.grid.set(bx + bw / 2, y + bh - 1, Tile::Door);
	}

	/// Step 4: wherever a road runs into the wall ring, replace that
	/// wall tile with road.
	fn open_gates(&mut self) {
		let (w, h) = (self.grid.w, self.grid.h);
		// Top and bottom walls: look at the row just inside.
		for x in 1..w - 1 {
			for (inner, edge) in [(1, 0), (h - 2, h - 1)] {
				if self.grid.get(x, inner) == Tile::Road {
					self.grid.set(x, edge, Tile::Road);
				}
			}
		}
		// Left and right walls: look at the column just inside.
		for y in 1..h - 1 {
			for (inner, edge) in [(1, 0), (w - 2, w - 1)] {
				if self.grid.get(inner, y) == Tile::Road {
					self.grid.set(edge, y, Tile::Road);
				}
			}
		}
	}
}
