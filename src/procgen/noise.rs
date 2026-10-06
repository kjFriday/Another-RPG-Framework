//! Smooth 2-D noise for terrain.
//!
//! # Value noise, in three steps
//!
//! 1. [`lattice`]: every integer point `(x, y)` gets a fixed random
//!    height in `0.0..1.0`, derived by hashing the coordinates.
//! 2. [`value_noise`]: between lattice points, blend the four nearest
//!    corners. A smoothstep curve hides the grid's straight seams.
//! 3. [`fbm`] ("fractal Brownian motion"): add several layers, each
//!    with twice the detail and half the strength of the last. Big
//!    layers make continents; small ones roughen the coasts.
//!
//! Each step is a pure function of `(seed, x, y)`, so terrain needs
//! no stored state and any point can be computed in any order.

use super::rng::mix;

/// Fixed pseudo-random value in `0.0..1.0` at integer point `(x, y)`.
pub fn lattice(seed: u64, x: i32, y: i32) -> f32 {
	// Pack both coordinates into one u64: x in the low 32 bits, y in
	// the high 32. `as u32` keeps negative coordinates distinct.
	let key = (x as u32 as u64) | ((y as u32 as u64) << 32);
	// Keep the top 24 bits: an f32 has a 24-bit significand, so the
	// division below is exact and the result is always below 1.0.
	(mix(seed ^ mix(key)) >> 40) as f32 / (1u64 << 24) as f32
}

/// Smoothly interpolated noise in `0.0..1.0` at any real point.
pub fn value_noise(seed: u64, x: f32, y: f32) -> f32 {
	let (x0, y0) = (x.floor(), y.floor());
	let (ix, iy) = (x0 as i32, y0 as i32);

	// Smoothstep, 3t^2 - 2t^3: has zero slope at t = 0 and t = 1, so
	// neighbouring grid cells join without visible creases.
	let smooth = |t: f32| t * t * (3.0 - 2.0 * t);
	let (sx, sy) = (smooth(x - x0), smooth(y - y0));

	// Bilinear blend of the four corners. Each output is a weighted
	// average of values in 0..1, so it also stays in 0..1.
	let corner = |dx, dy| lattice(seed, ix + dx, iy + dy);
	let top = corner(0, 0) * (1.0 - sx) + corner(1, 0) * sx;
	let bottom = corner(0, 1) * (1.0 - sx) + corner(1, 1) * sx;
	top * (1.0 - sy) + bottom * sy
}

/// Four octaves of [`value_noise`], normalized back to `0.0..1.0`.
///
/// Octave `n` samples at frequency `2^n` with weight `0.5^(n+1)`.
/// Dividing by the summed weights (0.9375) undoes the shrinkage.
/// Each octave uses `seed + n` so layers are not copies of each other.
pub fn fbm(seed: u64, x: f32, y: f32) -> f32 {
	let (mut total, mut weight, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
	for octave in 0..4u64 {
		let layer_seed = seed.wrapping_add(octave);
		total += value_noise(layer_seed, x * freq, y * freq) * weight;
		norm += weight;
		weight *= 0.5;
		freq *= 2.0;
	}
	total / norm
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn noise_stays_in_unit_range() {
		for i in 0..2000 {
			let (x, y) = (i as f32 * 0.37 - 300.0, i as f32 * 0.11);
			let v = fbm(5, x, y);
			assert!((0.0..1.0).contains(&v), "fbm({x}, {y}) = {v}");
		}
	}
}
