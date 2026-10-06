//! Overworld terrain from two noise fields: elevation and moisture.
//!
//! Every cell samples both fields (see `procgen::noise`) and picks a
//! biome by threshold, checked in this order:
//!
//! | condition                     | tile     |
//! |-------------------------------|----------|
//! | elevation < sea               | Water    |
//! | elevation < sea + 0.02        | Sand     |
//! | elevation > sea + 0.22        | Mountain |
//! | moisture > 0.55               | Forest   |
//! | otherwise                     | Grass    |
//!
//! `sea` runs from 0.30 (`Sea Level` 0) to 0.70 (`Sea Level` 100).
//! `Moisture` shifts the moisture field by up to +/-0.2.

use crate::procgen::noise::fbm;
use crate::procgen::rng::mix;
use crate::procgen::{Generator, ObjectData, Param, Params, Tile, TileGrid};

pub struct OverworldGen;

impl Generator for OverworldGen {
	fn name(&self) -> &'static str {
		"Overworld"
	}

	fn default_params(&self) -> Params {
		Params(vec![
			Param::int("Width", 80, 40, 160, 4),
			Param::int("Height", 30, 16, 60, 2),
			Param::int("Sea Level", 45, 0, 100, 5),
			Param::int("Scale", 12, 4, 40, 2),
			Param::int("Moisture", 50, 0, 100, 5),
		])
	}

	fn generate(&self, params: &Params, seed: u64) -> ObjectData {
		let (w, h) = (params.int("Width"), params.int("Height"));
		// Larger `Scale` = noise sampled more slowly = bigger features.
		let scale = params.int("Scale") as f32;
		let sea = 0.3 + 0.4 * params.int("Sea Level") as f32 / 100.0;
		let wetness = (params.int("Moisture") - 50) as f32 / 250.0;
		// A second, unrelated seed so moisture does not mirror height.
		let moisture_seed = mix(seed ^ 0xA5A5_A5A5);

		let mut grid = TileGrid::new(w, h, Tile::Water);
		for y in 0..h {
			for x in 0..w {
				// Cells are ~2x taller than wide, so step y twice as
				// fast to keep islands round on screen, not tall.
				let nx = x as f32 / scale;
				let ny = y as f32 * 2.0 / scale;
				let elevation = fbm(seed, nx, ny);
				let moisture = fbm(moisture_seed, nx, ny) + wetness;

				let tile = if elevation < sea {
					Tile::Water
				} else if elevation < sea + 0.02 {
					Tile::Sand
				} else if elevation > sea + 0.22 {
					Tile::Mountain
				} else if moisture > 0.55 {
					Tile::Forest
				} else {
					Tile::Grass
				};
				grid.set(x, y, tile);
			}
		}

		ObjectData::Grid(grid)
	}
}
