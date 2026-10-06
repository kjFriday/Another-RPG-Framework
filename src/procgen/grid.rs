//! The tile map shared by every map-like generator.
//!
//! Coordinates are `(x, y)` with `(0, 0)` at the top-left corner, `x`
//! growing right and `y` growing down, the same as terminal cells.
//! They are `i32` rather than `usize` so neighbour maths such as
//! `x - 1` can go negative at an edge and be rejected by
//! [`TileGrid::in_bounds`] instead of underflowing.

/// What occupies one cell of a map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
	Floor,
	Wall,
	Water,
	Sand,
	Grass,
	Forest,
	Mountain,
	Road,
	Door,
}

impl Tile {
	/// The character drawn for this tile. Floor and Sand share `.`;
	/// the UI tells them apart by colour (see `ui::theme`).
	pub fn glyph(self) -> char {
		match self {
			Tile::Floor => '.',
			Tile::Wall => '#',
			Tile::Water => '~',
			Tile::Sand => '.',
			Tile::Grass => '"',
			Tile::Forest => 'T',
			Tile::Mountain => '^',
			Tile::Road => '=',
			Tile::Door => '+',
		}
	}

	/// Whether a walker can stand here. Used by the connectivity
	/// checks: walls, water and mountains block movement.
	pub fn walkable(self) -> bool {
		!matches!(self, Tile::Wall | Tile::Water | Tile::Mountain)
	}
}

/// A `w` by `h` rectangle of tiles.
///
/// Stored as one flat `Vec`, row after row ("row-major"): the tile at
/// `(x, y)` lives at index `y * w + x`. One allocation, and each row
/// is a contiguous slice, which [`TileGrid::rows`] hands out directly.
#[derive(Debug, Clone, PartialEq)]
pub struct TileGrid {
	pub w: i32,
	pub h: i32,
	tiles: Vec<Tile>,
}

impl TileGrid {
	/// A grid with every cell set to `fill`.
	pub fn new(w: i32, h: i32, fill: Tile) -> Self {
		let tiles = vec![fill; (w * h) as usize];
		Self { w, h, tiles }
	}

	pub fn in_bounds(&self, x: i32, y: i32) -> bool {
		x >= 0 && y >= 0 && x < self.w && y < self.h
	}

	/// Flat index of `(x, y)`. Callers must check bounds first.
	fn idx(&self, x: i32, y: i32) -> usize {
		(y * self.w + x) as usize
	}

	/// The tile at `(x, y)`. Anything outside the grid reads as
	/// `Wall`, so edge cells need no special cases: the map behaves
	/// as if it were surrounded by solid rock.
	pub fn get(&self, x: i32, y: i32) -> Tile {
		if self.in_bounds(x, y) {
			self.tiles[self.idx(x, y)]
		} else {
			Tile::Wall
		}
	}

	/// Writes a tile. Writes outside the grid are silently ignored,
	/// which lets generators carve shapes that overlap the edge.
	pub fn set(&mut self, x: i32, y: i32, tile: Tile) {
		if self.in_bounds(x, y) {
			let i = self.idx(x, y);
			self.tiles[i] = tile;
		}
	}

	/// Sets every cell in the `w` by `h` box whose top-left is
	/// `(x, y)`. Parts outside the grid are skipped (see [`set`]).
	///
	/// [`set`]: TileGrid::set
	pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, tile: Tile) {
		for yy in y..y + h {
			for xx in x..x + w {
				self.set(xx, yy, tile);
			}
		}
	}

	/// Each row as a slice, top to bottom.
	pub fn rows(&self) -> impl Iterator<Item = &[Tile]> {
		self.tiles.chunks(self.w as usize)
	}

	/// Groups walkable tiles into connected regions.
	///
	/// Two tiles connect if they share an edge (up, down, left or
	/// right; diagonals do not count). Returns:
	/// - a label per tile: `Some(region number)`, or `None` if blocked;
	/// - the size of each region, indexed by region number.
	///
	/// A fully connected map therefore has exactly one region.
	pub fn regions(&self) -> (Vec<Option<usize>>, Vec<usize>) {
		let mut labels = vec![None; self.tiles.len()];
		let mut sizes = Vec::new();

		for start in 0..self.tiles.len() {
			if labels[start].is_some() || !self.tiles[start].walkable() {
				continue;
			}
			// Flood fill from `start`. An explicit stack (instead of
			// recursion) cannot overflow the call stack on big maps.
			let id = sizes.len();
			let mut size = 0;
			let mut stack = vec![start];
			labels[start] = Some(id);

			while let Some(i) = stack.pop() {
				size += 1;
				let (x, y) = (i as i32 % self.w, i as i32 / self.w);
				for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
					let (nx, ny) = (x + dx, y + dy);
					if !self.in_bounds(nx, ny) {
						continue;
					}
					let j = self.idx(nx, ny);
					if labels[j].is_none() && self.tiles[j].walkable() {
						// Label on push, not on pop, so no tile is
						// ever pushed twice.
						labels[j] = Some(id);
						stack.push(j);
					}
				}
			}
			sizes.push(size);
		}

		(labels, sizes)
	}

	/// Fills every walkable tile outside the biggest region with
	/// `fill`, leaving one connected area. On a tie, the
	/// highest-numbered region is kept, because `max_by_key` returns
	/// the last of several equal maxima.
	pub fn retain_largest_region(&mut self, fill: Tile) {
		let (labels, sizes) = self.regions();
		let Some(largest) = (0..sizes.len()).max_by_key(|&i| sizes[i]) else {
			return; // no walkable tiles at all
		};
		for (tile, label) in self.tiles.iter_mut().zip(labels) {
			if label.is_some_and(|l| l != largest) {
				*tile = fill;
			}
		}
	}
}
