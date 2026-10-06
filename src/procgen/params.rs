//! Generator settings, described as data.
//!
//! A generator never draws its own UI. It returns a list of
//! [`Param`]s, and the generator panel (`ui::panel`) renders *any*
//! list as sliders. Adding a slider to a generator is therefore one
//! line in its `default_params`, with no UI code at all.

/// The value of one setting plus the limits the UI must respect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamValue {
	/// Whole number `v`, kept within `min..=max`, moved by `step`.
	Int {
		v: i32,
		min: i32,
		max: i32,
		step: i32,
	},
	/// Index `idx` into a fixed list of named `options`.
	Choice {
		idx: usize,
		options: &'static [&'static str],
	},
	/// On/off switch.
	Toggle(bool),
}

/// One labelled setting. The label doubles as its lookup key.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
	pub label: &'static str,
	pub value: ParamValue,
}

impl Param {
	/// An integer slider starting at `v`.
	pub fn int(
		label: &'static str,
		v: i32,
		min: i32,
		max: i32,
		step: i32,
	) -> Self {
		let value = ParamValue::Int { v, min, max, step };
		Self { label, value }
	}

	/// A choice that starts on its first option.
	pub fn choice(
		label: &'static str,
		options: &'static [&'static str],
	) -> Self {
		let value = ParamValue::Choice { idx: 0, options };
		Self { label, value }
	}

	/// A toggle that starts `on` or off.
	pub fn toggle(label: &'static str, on: bool) -> Self {
		let value = ParamValue::Toggle(on);
		Self { label, value }
	}

	/// Moves the value one notch: `dir` is `-1` (left) or `+1`
	/// (right). Ints stop at their limits, choices wrap around from
	/// last to first, and toggles flip whichever way you press.
	pub fn adjust(&mut self, dir: i32) {
		match &mut self.value {
			ParamValue::Int { v, min, max, step } => {
				*v = (*v + dir * *step).clamp(*min, *max);
			}
			ParamValue::Choice { idx, options } => {
				// rem_euclid, unlike %, never returns a negative
				// number, so stepping left from 0 wraps to the end.
				let len = options.len() as i32;
				*idx = (*idx as i32 + dir).rem_euclid(len) as usize;
			}
			ParamValue::Toggle(on) => *on = !*on,
		}
	}

	/// Where the value sits in its range, `0.0..=1.0`. The panel uses
	/// this to decide how much of the slider bar to fill.
	pub fn fraction(&self) -> f32 {
		match self.value {
			ParamValue::Int { v, min, max, .. } if max > min => {
				(v - min) as f32 / (max - min) as f32
			}
			ParamValue::Choice { idx, options } if options.len() > 1 => {
				idx as f32 / (options.len() - 1) as f32
			}
			ParamValue::Toggle(true) => 1.0,
			// Off toggles, and ranges with nothing to slide along.
			_ => 0.0,
		}
	}

	/// The value as shown to the user: `60`, `Weapon`, or `On`.
	pub fn value_text(&self) -> String {
		match self.value {
			ParamValue::Int { v, .. } => v.to_string(),
			ParamValue::Choice { idx, options } => options[idx].to_string(),
			ParamValue::Toggle(true) => "On".to_string(),
			ParamValue::Toggle(false) => "Off".to_string(),
		}
	}
}

/// A generator's full list of settings, in display order.
#[derive(Debug, Clone, PartialEq)]
pub struct Params(pub Vec<Param>);

impl Params {
	/// Finds a setting by label.
	///
	/// Panics on an unknown label. A typo here is a programming bug,
	/// and the `generators_are_deterministic` test calls every
	/// generator, so such a typo fails the test suite immediately.
	fn find(&self, label: &str) -> ParamValue {
		self.0
			.iter()
			.find(|p| p.label == label)
			.unwrap_or_else(|| panic!("unknown generator param `{label}`"))
			.value
	}

	/// Reads an integer setting. Panics if `label` is not an int.
	pub fn int(&self, label: &str) -> i32 {
		match self.find(label) {
			ParamValue::Int { v, .. } => v,
			other => panic!("param `{label}` is not an int: {other:?}"),
		}
	}

	/// Reads a choice's index. Panics if `label` is not a choice.
	pub fn choice(&self, label: &str) -> usize {
		match self.find(label) {
			ParamValue::Choice { idx, .. } => idx,
			other => panic!("param `{label}` is not a choice: {other:?}"),
		}
	}

	/// Reads a toggle. Panics if `label` is not a toggle.
	pub fn toggle(&self, label: &str) -> bool {
		match self.find(label) {
			ParamValue::Toggle(on) => on,
			other => panic!("param `{label}` is not a toggle: {other:?}"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn adjust_clamps_ints_and_wraps_choices() {
		let mut p = Param::int("Size", 9, 0, 10, 2);
		p.adjust(1);
		assert_eq!(p.value_text(), "10");
		p.adjust(1);
		assert_eq!(p.value_text(), "10");

		let mut c = Param::choice("Slot", &["A", "B", "C"]);
		c.adjust(-1);
		assert_eq!(c.value_text(), "C");
		c.adjust(1);
		assert_eq!(c.value_text(), "A");
	}
}
