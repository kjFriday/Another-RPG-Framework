//! The generator panel: sliders on the left, live preview on the
//! right, and two buttons (Reroll, Spawn).
//!
//! # Cursor model
//!
//! The panel has one cursor that runs down every slider and then
//! every button, in that order:
//!
//! ```text
//!  index 0..n-1   sliders   (n = number of params)
//!  index n        [ Reroll ]
//!  index n+1      [ Spawn ]
//! ```
//!
//! Left/Right only act on sliders; Enter only acts on buttons.
//!
//! # Talking to the app
//!
//! The panel cannot reach the world or the log; it does not own
//! them. Instead [`GeneratorPanel::handle_key`] returns a
//! [`PanelEvent`] describing what should happen, and the app carries
//! it out. This keeps the panel testable on its own and keeps all
//! world changes in one place (`app/input.rs`).

use super::{theme, wrap_step};
use crate::procgen::{
	Generator, ObjectData, Param, Params, Recipe, Rng, TileGrid,
};
use crate::world::ObjectKind;
use crossterm::event::KeyCode;
use ratatui::{
	Frame,
	layout::{Constraint, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Line, Span},
	widgets::{Block, Borders, Paragraph},
};

const BUTTONS: [&str; 2] = ["Reroll", "Spawn"];
/// Characters in a slider bar, e.g. `[■■■·······]` is 10.
const SLIDER_WIDTH: usize = 10;
/// Width of the controls column, in terminal cells.
const CONTROLS_WIDTH: u16 = 36;

pub struct GeneratorPanel {
	kind: ObjectKind,
	/// Chosen at run time by `kind`, hence a trait object.
	generator: Box<dyn Generator>,
	params: Params,
	seed: u64,
	/// Always up to date with `params` and `seed`: every change to
	/// either regenerates it immediately.
	preview: ObjectData,
	cursor: usize,
}

/// What the app should do after a key press in the panel.
pub enum PanelEvent {
	/// Nothing beyond redrawing.
	None,
	/// Write this message to the log.
	Log(String),
	/// Close the panel and return to the menu.
	Close,
	/// Add this content to the world.
	Spawn {
		kind: ObjectKind,
		data: ObjectData,
		recipe: Recipe,
	},
}

impl GeneratorPanel {
	/// Opens a panel for `kind`, or returns `None` if that kind has
	/// no generator. The `?` returns `None` early in that case.
	pub fn open(kind: ObjectKind, seed: u64) -> Option<Self> {
		let generator = kind.generator()?;
		let params = generator.default_params();
		let preview = generator.generate(&params, seed);
		Some(Self {
			kind,
			generator,
			params,
			seed,
			preview,
			cursor: 0,
		})
	}

	fn regenerate(&mut self) {
		self.preview = self.generator.generate(&self.params, self.seed);
	}

	pub fn handle_key(&mut self, code: KeyCode) -> PanelEvent {
		let slots = self.params.0.len() + BUTTONS.len();
		match code {
			KeyCode::Char('q') | KeyCode::Char('Q') => PanelEvent::Close,
			KeyCode::Up => {
				self.cursor = wrap_step(self.cursor, slots, -1);
				PanelEvent::None
			}
			KeyCode::Down => {
				self.cursor = wrap_step(self.cursor, slots, 1);
				PanelEvent::None
			}
			KeyCode::Left | KeyCode::Right => {
				// `get_mut` returns None when the cursor is on a
				// button, so arrows do nothing there.
				let Some(param) = self.params.0.get_mut(self.cursor) else {
					return PanelEvent::None;
				};
				param.adjust(if code == KeyCode::Left { -1 } else { 1 });
				let msg = format!("{} = {}", param.label, param.value_text());
				self.regenerate();
				PanelEvent::Log(msg)
			}
			KeyCode::Enter => self.press_button(),
			_ => PanelEvent::None,
		}
	}

	/// Enter on a button. `checked_sub` turns the cursor into a button
	/// index, or `None` when it is still on a slider.
	fn press_button(&mut self) -> PanelEvent {
		match self.cursor.checked_sub(self.params.0.len()) {
			// Reroll: derive the next seed from the current one, so a
			// sequence of rerolls is itself repeatable.
			Some(0) => {
				self.seed = Rng::new(self.seed).next_u64();
				self.regenerate();
				PanelEvent::Log(format!("> Reroll (seed {:x})", self.seed))
			}
			// Spawn: hand a copy of the preview to the app. The panel
			// keeps its own copy so you can spawn again or keep tuning.
			Some(1) => PanelEvent::Spawn {
				kind: self.kind,
				data: self.preview.clone(),
				recipe: Recipe {
					generator: self.generator.name(),
					params: self.params.clone(),
					seed: self.seed,
				},
			},
			_ => PanelEvent::None,
		}
	}

	/// Draws the controls and preview side by side into `area`.
	pub fn render(&self, frame: &mut Frame, area: Rect) {
		let [controls, preview] = Layout::horizontal([
			Constraint::Length(CONTROLS_WIDTH),
			Constraint::Min(10),
		])
		.areas(area);
		self.render_controls(frame, controls);
		self.render_preview(frame, preview);
	}

	fn render_controls(&self, frame: &mut Frame, area: Rect) {
		let n = self.params.0.len();

		let mut lines: Vec<Line> = self
			.params
			.0
			.iter()
			.enumerate()
			.map(|(i, p)| slider_line(p, i == self.cursor))
			.collect();

		lines.push(Line::default()); // blank spacer row
		for (i, label) in BUTTONS.iter().enumerate() {
			let is_selected = n + i == self.cursor;
			let (marker, style) = if is_selected {
				("> ", theme::selected().add_modifier(Modifier::REVERSED))
			} else {
				("  ", Style::default().fg(Color::Cyan))
			};
			lines.push(Line::from(vec![
				Span::raw(marker),
				Span::styled(format!("[ {label} ]"), style),
			]));
		}

		lines.push(Line::default());
		// `{:016x}`: hexadecimal, zero-padded to 16 digits (64 bits).
		let seed = format!("Seed: {:016x}", self.seed);
		lines.push(Line::styled(seed, Style::default().fg(Color::DarkGray)));

		let block = Block::default()
			.title(format!(" {} Generator ", self.generator.name()))
			.borders(Borders::ALL)
			.border_style(theme::border_primary());
		frame.render_widget(Paragraph::new(lines).block(block), area);
	}

	fn render_preview(&self, frame: &mut Frame, area: Rect) {
		let lines = match &self.preview {
			ObjectData::Grid(grid) => grid_lines(grid),
			ObjectData::Item(item) => {
				let bold = Style::default().add_modifier(Modifier::BOLD);
				let mut lines = vec![
					Line::styled(item.name.clone(), bold),
					Line::raw(format!(
						"{:?} {:?}, level {}",
						item.rarity, item.slot, item.level
					)),
					Line::default(),
				];
				for (stat, v) in &item.stats {
					lines.push(Line::raw(format!("  +{v} {stat:?}")));
				}
				lines
			}
		};
		// Maps bigger than the box are clipped at the right and bottom.
		let block = Block::default()
			.title(" Preview ")
			.borders(Borders::ALL)
			.border_style(theme::border_secondary());
		frame.render_widget(Paragraph::new(lines).block(block), area);
	}
}

/// One slider row: `> Width        [■■■·······] 60`.
fn slider_line(p: &Param, selected: bool) -> Line<'static> {
	let filled = (p.fraction() * SLIDER_WIDTH as f32).round() as usize;
	let text = format!(
		"{}{:<13}[{}{}] {}", // `{:<13}` left-aligns in 13 columns
		if selected { "> " } else { "  " },
		p.label,
		"■".repeat(filled),
		"·".repeat(SLIDER_WIDTH - filled),
		p.value_text()
	);
	let style = if selected {
		theme::selected()
	} else {
		Style::default()
	};
	Line::styled(text, style)
}

/// Converts a grid to coloured text, one `Line` per row.
///
/// `chunk_by` groups runs of identical neighbouring tiles, so a row
/// like `####....##` becomes three styled spans instead of ten. Fewer
/// spans means less work for ratatui on every frame.
fn grid_lines(grid: &TileGrid) -> Vec<Line<'static>> {
	grid.rows()
		.map(|row| {
			let spans: Vec<Span> = row
				.chunk_by(|a, b| a == b)
				.map(|run| {
					let text: String = run.iter().map(|t| t.glyph()).collect();
					Span::styled(text, theme::tile(run[0]))
				})
				.collect();
			Line::from(spans)
		})
		.collect()
}
