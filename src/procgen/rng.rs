//! Seeded pseudo-random numbers.
//!
//! # Algorithm
//!
//! This is SplitMix64, as published by Sebastiano Vigna at
//! <https://prng.di.unimi.it/splitmix64.c>. It derives from Steele,
//! Lea & Flood, "Fast Splittable Pseudorandom Number Generators"
//! (OOPSLA 2014). The whole state is one `u64`; each call adds a
//! fixed odd constant to it and scrambles the result with [`mix`].
//!
//! # Why not the `rand` crate?
//!
//! Generators here must give *identical* output for a seed forever,
//! so a saved seed always rebuilds the same dungeon. The `rand` docs
//! warn that `StdRng` is not guaranteed to be reproducible across
//! releases. Owning these ~40 lines removes that risk.
//!
//! # Not for security
//!
//! SplitMix64 is predictable from its output. Never use it for
//! anything secret.

/// A seeded random number generator. Cloning it forks the sequence:
/// both copies then produce the same numbers.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
	/// Starts a sequence. Equal seeds always give equal sequences.
	pub fn new(seed: u64) -> Self {
		Self(seed)
	}

	/// Returns the next 64 random bits.
	pub fn next_u64(&mut self) -> u64 {
		// 0x9E37_79B9_7F4A_7C15 is floor(2^64 / golden ratio). It is
		// odd, so repeatedly adding it visits all 2^64 states before
		// repeating. `wrapping_add` makes the overflow explicit.
		self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
		mix(self.0)
	}

	/// Uniform integer in `lo..hi` (`hi` excluded). Returns `lo` when
	/// the range is empty (`hi <= lo`) instead of panicking.
	///
	/// Uses `% n`, which very slightly favours small values when
	/// 2^64 is not a multiple of `n`. The skew is at most `n / 2^64`,
	/// far below anything visible in generated content.
	pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
		if hi <= lo {
			return lo;
		}
		let span = (hi - lo) as u64;
		lo + (self.next_u64() % span) as i32
	}

	/// True with probability `percent / 100`. Values `<= 0` are never
	/// true and values `>= 100` are always true.
	pub fn chance(&mut self, percent: i32) -> bool {
		self.range(0, 100) < percent
	}

	/// A uniformly chosen element. Panics if `items` is empty.
	pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
		&items[self.range(0, items.len() as i32) as usize]
	}
}

/// The SplitMix64 output function: two xor-shift-multiply rounds that
/// spread every input bit across the whole output. Also used on its
/// own as a hash (see `noise.rs`) and to derive new seeds.
pub fn mix(mut z: u64) -> u64 {
	z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
	z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn range_stays_in_bounds() {
		let mut rng = Rng::new(7);
		for _ in 0..1000 {
			let v = rng.range(-3, 4);
			assert!((-3..4).contains(&v));
		}
		assert_eq!(rng.range(5, 5), 5);
	}

	#[test]
	fn same_seed_same_sequence() {
		let (mut a, mut b) = (Rng::new(99), Rng::new(99));
		for _ in 0..100 {
			assert_eq!(a.next_u64(), b.next_u64());
		}
	}
}
