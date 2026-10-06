//! Items: choose a base by slot, roll a rarity, then add affixes.
//!
//! | rarity    | name shape                        | stats      |
//! |-----------|-----------------------------------|------------|
//! | Common    | `Sword`                           | base       |
//! | Magic     | `Keen Sword`                      | base + 1   |
//! | Rare      | `Keen Sword of Haste`             | base + 2   |
//! | Legendary | `Legendary Keen Sword of Haste`   | as Rare x2 |
//!
//! The tables are Rust constants for now. Phase 3 of
//! `docs/GENERATORS_PLAN.md` moves them into RON asset files.

use crate::procgen::{Generator, ObjectData, Param, Params, Rng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
	Common,
	Magic,
	Rare,
	Legendary,
}

/// Where the item is worn or held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
	Weapon,
	Armor,
	Trinket,
}

/// A stat an item can raise. Shown with `{:?}`, so the variant name
/// is also the on-screen text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
	Damage,
	Armor,
	Health,
	Strength,
	Speed,
	Wisdom,
	Fire,
	Crit,
}

/// A finished item.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemData {
	pub name: String,
	pub rarity: Rarity,
	pub slot: Slot,
	pub level: i32,
	/// `(stat, bonus)` pairs; the first is always the base stat.
	pub stats: Vec<(Stat, i32)>,
}

const SLOTS: [Slot; 3] = [Slot::Weapon, Slot::Armor, Slot::Trinket];

/// Words placed before the base name, with the stat each adds.
const PREFIXES: [(&str, Stat); 4] = [
	("Flaming", Stat::Fire),
	("Sturdy", Stat::Armor),
	("Keen", Stat::Crit),
	("Vital", Stat::Health),
];

/// Phrases placed after the base name, with the stat each adds.
const SUFFIXES: [(&str, Stat); 3] = [
	("of Might", Stat::Strength),
	("of Haste", Stat::Speed),
	("of the Owl", Stat::Wisdom),
];

/// Base names for a slot, and the stat every one of them grants.
fn bases(slot: Slot) -> (&'static [&'static str], Stat) {
	match slot {
		Slot::Weapon => (&["Sword", "Axe", "Dagger", "Staff"], Stat::Damage),
		Slot::Armor => (&["Helm", "Cuirass", "Boots", "Shield"], Stat::Armor),
		Slot::Trinket => (&["Ring", "Amulet", "Charm"], Stat::Health),
	}
}

pub struct ItemGen;

impl Generator for ItemGen {
	fn name(&self) -> &'static str {
		"Item"
	}

	fn default_params(&self) -> Params {
		Params(vec![
			Param::int("Item Level", 10, 1, 60, 1),
			Param::int("Rarity Bias", 30, 0, 100, 5),
			Param::choice("Slot", &["Any", "Weapon", "Armor", "Trinket"]),
		])
	}

	fn generate(&self, params: &Params, seed: u64) -> ObjectData {
		let mut rng = Rng::new(seed);
		let level = params.int("Item Level");

		// Choice 0 is "Any"; choices 1..=3 line up with SLOTS[0..=2].
		let slot = match params.choice("Slot") {
			0 => *rng.pick(&SLOTS),
			i => SLOTS[i - 1],
		};

		// score = d100 + bias% of a second d100, so 0..=198.
		// At bias 0 the score is 0..=99: 30% Magic, never Rare or
		// Legendary. Raising the bias adds up to 99 more points.
		let bias = params.int("Rarity Bias");
		let score = rng.range(0, 100) + bias * rng.range(0, 100) / 100;
		let rarity = match score {
			135.. => Rarity::Legendary,
			105.. => Rarity::Rare,
			70.. => Rarity::Magic,
			_ => Rarity::Common,
		};

		// Each bonus is (level / 2 + 1) times 1, 2 or 3, doubled for
		// Legendary items. Integer division rounds down.
		let power = if rarity == Rarity::Legendary { 2 } else { 1 };
		let roll = |rng: &mut Rng| (level / 2 + 1) * rng.range(1, 4) * power;

		let (names, base_stat) = bases(slot);
		let mut name = rng.pick(names).to_string();
		let mut stats = vec![(base_stat, roll(&mut rng))];

		if rarity != Rarity::Common {
			let (prefix, stat) = *rng.pick(&PREFIXES);
			name = format!("{prefix} {name}");
			stats.push((stat, roll(&mut rng)));
		}
		if matches!(rarity, Rarity::Rare | Rarity::Legendary) {
			let (suffix, stat) = *rng.pick(&SUFFIXES);
			name = format!("{name} {suffix}");
			stats.push((stat, roll(&mut rng)));
		}
		if rarity == Rarity::Legendary {
			name = format!("Legendary {name}");
		}

		ObjectData::Item(ItemData {
			name,
			rarity,
			slot,
			level,
			stats,
		})
	}
}
