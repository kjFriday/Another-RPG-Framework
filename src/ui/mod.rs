//! Everything drawn on screen.
//!
//! # Screen layout
//!
//! ```text
//! +--------------------------------------------------+
//! | tabs (menu categories)                  3 rows   |
//! +--------------------------------------------------+
//! | header (name, status, tick, objects)    5 rows   |
//! +----------------------------------+---------------+
//! | main area                        | log           |
//! |  menu screen: entry list         | (LOG_WIDTH    |
//! |  generator:   sliders + preview  |  columns)     |
//! +----------------------------------+---------------+
//! | footer (key hints)                      3 rows   |
//! +--------------------------------------------------+
//! ```
//!
//! # Files
//!
//! - `menu.rs`: tab bar and entry list ([`MenuSystem`]).
//! - `panel.rs`: generator sliders and preview ([`GeneratorPanel`]).
//! - `widgets.rs`: header, log and footer.
//! - `theme.rs`: every colour and style.
//!
//! The UI reads the simulation but never changes it. All changes go
//! through the app (`app/input.rs`).

mod menu;
mod panel;
mod theme;
mod widgets;

pub use menu::MenuSystem;
pub use panel::{GeneratorPanel, PanelEvent};

use crate::sim::Simulation;
use ratatui::{
	Frame,
	layout::{Constraint, Layout},
};

/// Width of the log column on the right, in terminal cells.
const LOG_WIDTH: u16 = 36;

/// Draws one complete frame.
///
/// `panel` is `Some` while the generator panel is open; it then
/// replaces the entry list in the main area.
pub fn render(
	frame: &mut Frame,
	sim: &Simulation,
	menu: &MenuSystem,
	panel: Option<&GeneratorPanel>,
) {
	// `.areas(...)` returns a fixed-size array, so the four names
	// below are checked at compile time against the four constraints.
	let [tabs, header, body, footer] = Layout::vertical([
		Constraint::Length(3),
		Constraint::Length(5),
		Constraint::Min(5), // takes whatever height is left
		Constraint::Length(3),
	])
	.margin(1) // one blank cell around the edge of the terminal
	.areas(frame.area());

	let [main, log] = Layout::horizontal([
		Constraint::Min(20),
		Constraint::Length(LOG_WIDTH),
	])
	.areas(body);

	menu.render_tabs(frame, tabs);
	widgets::header(frame, header, sim);
	match panel {
		Some(panel) => panel.render(frame, main),
		None => {
			let [items, _] = Layout::horizontal([
				Constraint::Length(28),
				Constraint::Min(0),
			])
			.areas(main);
			menu.render_items(frame, items);
		}
	}
	widgets::log(frame, log, &sim.log);
	widgets::footer(frame, footer, panel.is_some());
}

/// Moves `index` one step (`dir` = +1 or -1) through `0..len`,
/// wrapping past either end. Returns 0 when `len` is 0.
///
/// Shared by the menu and the panel so every list wraps the same way.
fn wrap_step(index: usize, len: usize, dir: i32) -> usize {
	if len == 0 {
		return 0;
	}
	// Signed maths so that 0 - 1 can go negative, then rem_euclid
	// (always >= 0) folds it back to len - 1.
	(index as i64 + dir as i64).rem_euclid(len as i64) as usize
}

#[cfg(test)]
mod tests {
	use super::wrap_step;

	#[test]
	fn wrap_step_wraps_both_ways() {
		assert_eq!(wrap_step(0, 3, -1), 2);
		assert_eq!(wrap_step(2, 3, 1), 0);
		assert_eq!(wrap_step(1, 3, 1), 2);
		assert_eq!(wrap_step(5, 0, 1), 0);
	}
}
