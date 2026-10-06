//! Procedural content generation: the bottom layer of the program.
//!
//! Nothing in here knows about the terminal, the menu or the world.
//! A generator is a pure function from `(settings, seed)` to content:
//! call it twice with the same inputs and you get the same output.
//! That one rule is what makes [`Recipe`]s work. Store the inputs, not
//! the output, and the output can always be rebuilt.
//!
//! # Files
//!
//! - `rng.rs`: seeded random numbers ([`Rng`]).
//! - `noise.rs`: smooth terrain noise (used by the overworld).
//! - `params.rs`: settings described as data ([`Param`], [`Params`]).
//! - `grid.rs`: the shared tile map ([`TileGrid`], [`Tile`]).
//! - `generators/`: one file per generator.
//!
//! # Adding a generator
//!
//! 1. Create `generators/<name>.rs` with a unit struct implementing
//!    [`Generator`], and re-export it from `generators/mod.rs`.
//! 2. Return it from `ObjectKind::generator` in `world/kind.rs`.
//! 3. Add it to `tests::all` (bottom of this file) so the shared
//!    tests cover it.

mod generators;
mod grid;
mod noise;
mod params;
mod rng;

// `pub use` re-exports items so callers write `procgen::Tile`, not
// `procgen::grid::Tile`. The file layout stays a private detail.
pub use generators::{
	CaveGen, DungeonGen, ItemData, ItemGen, OverworldGen, TownGen,
};
pub use grid::{Tile, TileGrid};
pub use params::{Param, Params};
pub use rng::{Rng, mix};

/// What a generator produces.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectData {
	Item(ItemData),
	/// Any map-like output: caves, dungeons, towns, overworlds.
	Grid(TileGrid),
}

/// The inputs that produced some content. Generators are
/// deterministic, so feeding these back in rebuilds it exactly.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
	/// [`Generator::name`] of the generator that ran.
	pub generator: &'static str,
	pub params: Params,
	pub seed: u64,
}

/// The contract every generator fulfils.
///
/// The trait is "object safe": no generic methods, and no `Self` in
/// return position. That allows `Box<dyn Generator>`, a pointer to
/// *some* generator whose concrete type is chosen at run time, which
/// is how the UI holds whichever generator the user opened.
pub trait Generator {
	/// Display name, also stored in [`Recipe::generator`].
	fn name(&self) -> &'static str;

	/// The settings to show, with their starting values.
	fn default_params(&self) -> Params;

	/// Builds content. Must depend on `params` and `seed` only.
	fn generate(&self, params: &Params, seed: u64) -> ObjectData;
}

#[cfg(test)]
pub mod tests {
	use super::*;

	/// Every generator, for tests that should cover them all.
	pub fn all() -> Vec<Box<dyn Generator>> {
		vec![
			Box::new(ItemGen),
			Box::new(CaveGen),
			Box::new(DungeonGen),
			Box::new(TownGen),
			Box::new(OverworldGen),
		]
	}

	#[test]
	fn generators_are_deterministic() {
		for generator in all() {
			let params = generator.default_params();
			let first = generator.generate(&params, 42);
			let second = generator.generate(&params, 42);
			assert_eq!(first, second, "{} differs", generator.name());
		}
	}

	/// Overworlds are left out: water and mountains split land
	/// into islands on purpose.
	#[test]
	fn grid_generators_produce_one_connected_region() {
		let walkable_maps: [Box<dyn Generator>; 3] =
			[Box::new(CaveGen), Box::new(DungeonGen), Box::new(TownGen)];
		for generator in walkable_maps {
			let params = generator.default_params();
			for seed in 0..20 {
				let ObjectData::Grid(grid) = generator.generate(&params, seed)
				else {
					panic!("{} did not produce a grid", generator.name());
				};
				let (_, sizes) = grid.regions();
				assert_eq!(
					sizes.len(),
					1,
					"{} seed {seed}: {} regions",
					generator.name(),
					sizes.len()
				);
			}
		}
	}
}
