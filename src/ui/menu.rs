//! The tabbed menu: categories across the top, entries down the left.
//!
//! [`MenuSystem`] is pure state (what exists, what is selected). The
//! `render_*` methods turn that state into widgets each frame;
//! nothing about the drawing is remembered between frames.

use super::{theme, wrap_step};
use crate::command::Command;
use ratatui::{
	Frame,
	layout::Rect,
	text::Line,
	widgets::{Block, Borders, List, ListItem, ListState, Tabs},
};

/// One selectable entry: the text shown and the command it runs.
#[derive(Debug, Clone)]
pub struct MenuItem {
	pub label: String,
	pub command: Command,
}

/// One tab and its entries.
#[derive(Debug, Clone, Default)]
pub struct MenuCategory {
	pub title: String,
	pub items: Vec<MenuItem>,
}

/// The whole menu plus the cursor position.
///
/// Invariant: `selected_item` is reset to 0 whenever the tab changes,
/// so it always indexes an entry of the current tab (or the tab is
/// empty, in which case [`MenuSystem::selected`] returns `None`).
#[derive(Debug, Clone, Default)]
pub struct MenuSystem {
	pub categories: Vec<MenuCategory>,
	pub selected_category: usize,
	pub selected_item: usize,
}

/// Menu definitions as compact literals: `(tab, [(label, command)])`.
/// See `app::menus` for the real menu written in this shape.
pub type MenuDef<'a> = (&'a str, &'a [(&'a str, Command)]);

impl MenuSystem {
	/// Builds a menu from definitions, with the cursor on the first
	/// entry of the first tab.
	pub fn from_defs(defs: &[MenuDef]) -> Self {
		let categories = defs
			.iter()
			.map(|(title, items)| MenuCategory {
				title: title.to_string(),
				items: items
					.iter()
					// `&(label, command)` destructures each tuple
					// reference in the parameter list itself.
					.map(|&(label, command)| MenuItem {
						label: label.to_string(),
						command,
					})
					.collect(),
			})
			.collect();
		// `..Default::default()` fills the remaining fields (both
		// cursor indices) with their defaults, which are 0.
		Self {
			categories,
			..Default::default()
		}
	}

	pub fn next_category(&mut self) {
		let len = self.categories.len();
		self.selected_category = wrap_step(self.selected_category, len, 1);
		self.selected_item = 0;
	}

	pub fn previous_category(&mut self) {
		let len = self.categories.len();
		self.selected_category = wrap_step(self.selected_category, len, -1);
		self.selected_item = 0;
	}

	pub fn next_item(&mut self) {
		let len = self.current_items().len();
		self.selected_item = wrap_step(self.selected_item, len, 1);
	}

	pub fn previous_item(&mut self) {
		let len = self.current_items().len();
		self.selected_item = wrap_step(self.selected_item, len, -1);
	}

	/// The highlighted entry, or `None` if the current tab is empty.
	pub fn selected(&self) -> Option<&MenuItem> {
		self.current_items().get(self.selected_item)
	}

	fn current_category(&self) -> Option<&MenuCategory> {
		self.categories.get(self.selected_category)
	}

	fn current_items(&self) -> &[MenuItem] {
		// `map_or(&[], ...)`: an empty slice when there are no tabs.
		self.current_category().map_or(&[], |c| c.items.as_slice())
	}

	/// Draws the tab bar.
	pub fn render_tabs(&self, frame: &mut Frame, area: Rect) {
		let titles: Vec<Line> = self
			.categories
			.iter()
			.map(|c| Line::from(c.title.as_str()))
			.collect();
		let block = Block::default()
			.title(" RPG Engine Menu ")
			.borders(Borders::ALL);
		let tabs = Tabs::new(titles)
			.block(block)
			.select(self.selected_category)
			.style(theme::border_secondary())
			.highlight_style(theme::selected().underlined());
		frame.render_widget(tabs, area);
	}

	/// Draws the current tab's entries as a list with a `>` cursor.
	pub fn render_items(&self, frame: &mut Frame, area: Rect) {
		let Some(category) = self.current_category() else {
			return;
		};
		let items: Vec<ListItem> = category
			.items
			.iter()
			.map(|item| ListItem::new(item.label.as_str()))
			.collect();
		let block = Block::default()
			.title(format!(" {} ", category.title))
			.borders(Borders::ALL)
			.border_style(theme::border_secondary());
		let list = List::new(items)
			.block(block)
			.highlight_style(theme::selected())
			.highlight_symbol("> ");

		// `ListState` tells the `List` widget which row to highlight.
		// It is rebuilt each frame from our own `selected_item`.
		let mut state =
			ListState::default().with_selected(Some(self.selected_item));
		frame.render_stateful_widget(list, area, &mut state);
	}
}
