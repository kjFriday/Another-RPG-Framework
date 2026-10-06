//! Colours and text styles, in one place so the look can change
//! without touching layout code.
//!
//! These are the 16 basic terminal colours. Each terminal theme picks
//! its own exact shade for "Yellow" or "DarkGray", so the same code
//! looks a little different in each terminal.

use crate::procgen::Tile;
use ratatui::style::{Color, Modifier, Style};

/// The highlighted menu entry, slider or button.
pub fn selected() -> Style {
	Style::default()
		.fg(Color::Yellow)
		.add_modifier(Modifier::BOLD)
}

/// Key names in the footer, such as `ENTER`.
pub fn key_hint() -> Style {
	Style::default().fg(Color::Yellow)
}

/// The quit key: red, because it ends the program.
pub fn danger() -> Style {
	Style::default().fg(Color::Red)
}

/// Border for panels that hold the main content.
pub fn border_primary() -> Style {
	Style::default().fg(Color::Cyan)
}

/// Border for supporting panels.
pub fn border_secondary() -> Style {
	Style::default().fg(Color::Gray)
}

/// Log lines that echo a button press (they start with `>`).
pub fn log_echo() -> Style {
	Style::default().fg(Color::Yellow)
}

/// Colour for each map tile. Floor and Sand share a glyph (`.`), so
/// colour is the only way to tell them apart.
pub fn tile(tile: Tile) -> Style {
	let fg = match tile {
		Tile::Floor => Color::Gray,
		Tile::Wall => Color::DarkGray,
		Tile::Water => Color::Blue,
		Tile::Sand => Color::LightYellow,
		Tile::Grass => Color::LightGreen,
		Tile::Forest => Color::Green,
		Tile::Mountain => Color::White,
		Tile::Road => Color::Yellow,
		Tile::Door => Color::LightRed,
	};
	Style::default().fg(fg)
}
