//! The fixed panels around the edge: header, log and footer.
//!
//! Each function draws one panel from the current state. They hold no
//! state of their own (ratatui is an "immediate mode" library: the
//! whole screen is described again every frame, and ratatui compares
//! it with the previous frame to send only changed cells to the
//! terminal).

use super::theme;
use crate::sim::{Log, Simulation};
use ratatui::{
	Frame,
	layout::Rect,
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, Borders, Paragraph, Wrap},
};

/// Title, version, run state, tick count and object count.
pub fn header(frame: &mut Frame, area: Rect, sim: &Simulation) {
	let (status, color) = if sim.paused {
		("PAUSED", Color::Yellow)
	} else {
		("RUNNING", Color::Green)
	};
	let bold = Style::default().add_modifier(Modifier::BOLD);

	let text = vec![
		Line::from(vec![
			Span::styled(&sim.name, bold),
			Span::raw(" v"),
			Span::raw(&sim.version),
		]),
		Line::from(vec![
			Span::raw("Status: "),
			Span::styled(status, bold.fg(color)),
			Span::raw(format!(
				" | Tick: {} | Objects: {}",
				sim.tick_count,
				sim.world.count()
			)),
		]),
	];

	let block = Block::default()
		.title(" Engine Control ")
		.borders(Borders::ALL)
		.border_style(theme::border_primary());
	frame.render_widget(Paragraph::new(text).block(block), area);
}

/// The newest log messages that fit, word-wrapped, newest at the
/// bottom like a chat window.
pub fn log(frame: &mut Frame, area: Rect, log: &Log) {
	let block = Block::default()
		.title(" Log ")
		.borders(Borders::ALL)
		.border_style(theme::border_secondary());
	let inner = block.inner(area); // the area minus its border
	let width = inner.width.max(1) as usize;

	// Walk back from the newest message, estimating how many rows
	// each needs once wrapped (characters / width, rounded up), and
	// stop when the box is full. The estimate counts characters, so a
	// line broken early at a word boundary can need one extra row;
	// the paragraph then clips at the bottom edge.
	let mut rows = 0;
	let mut shown = Vec::new();
	for msg in log.iter().rev() {
		let height = msg.chars().count().div_ceil(width).max(1);
		if rows + height > inner.height as usize && !shown.is_empty() {
			break;
		}
		rows += height;
		let style = if msg.starts_with('>') {
			theme::log_echo()
		} else {
			Style::default()
		};
		shown.push(Line::styled(msg, style));
	}
	shown.reverse(); // back to oldest-first for top-to-bottom drawing

	// `trim: false` keeps leading spaces on wrapped lines.
	let paragraph = Paragraph::new(shown)
		.wrap(Wrap { trim: false })
		.block(block);
	frame.render_widget(paragraph, area);
}

/// Key hints. They change with the screen, because the same keys do
/// different things in the menu and in the generator panel.
pub fn footer(frame: &mut Frame, area: Rect, in_generator: bool) {
	// Each pair is (key name, what it does).
	let common = [("SPACE/P", "Pause")];
	let screen_keys: &[(&str, &str)] = if in_generator {
		&[
			("\u{2191}\u{2193}", "Select"),
			("\u{2190}\u{2192}", "Adjust"),
			("ENTER", "Press"),
			("Q", "Back"),
		]
	} else {
		&[
			("\u{2190}\u{2192}", "Tabs"),
			("\u{2191}\u{2193}/ENTER", "Select"),
		]
	};

	let mut spans = Vec::new();
	for (key, action) in common.iter().chain(screen_keys) {
		spans.push(Span::raw("[ "));
		spans.push(Span::styled(*key, theme::key_hint()));
		spans.push(Span::raw(format!(" ] {action} | ")));
	}
	// Quit goes last, in red.
	spans.push(Span::raw("[ "));
	spans.push(Span::styled("ESC", theme::danger()));
	spans.push(Span::raw(" ] Quit"));

	let block = Block::default()
		.title(" Controls ")
		.borders(Borders::ALL)
		.border_style(Style::default().fg(Color::DarkGray));
	frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}
