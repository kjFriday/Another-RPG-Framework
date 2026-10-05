use crate::object::{ObjectKind, World};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEventKind};
use ratatui::{
	Frame, Terminal, backend::Backend, layout::{Constraint, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};
use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};

/// Maximum number of log lines kept in the simulation log.
const MAX_LOG_LINES: usize = 5;

/// Core state for the RPG simulation.
pub struct RpgState {
	pub name: String,
	pub version: String,
	pub tick_count: u64,
	pub is_paused: bool,
	pub should_quit: bool,
	pub log_messages: VecDeque<String>,
	pub world: World,
}

impl RpgState {
	pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
		let mut state = Self {
			name: name.into(),
			version: version.into(),
			tick_count: 0,
			is_paused: false,
			should_quit: false,
			log_messages: VecDeque::with_capacity(MAX_LOG_LINES),
			world: World::default(),
		};
		state.log("Simulation initialized.");
		state
	}

	/// Appends a message to the log, dropping the oldest lines past `MAX_LOG_LINES`.
	pub fn log(&mut self, msg: impl Into<String>) {
		self.log_messages.push_back(msg.into());
		while self.log_messages.len() > MAX_LOG_LINES {
			self.log_messages.pop_front();
		}
	}

	/// Advances the RPG simulation by 1 step/tick.
	pub fn tick(&mut self) {
		if !self.is_paused {
			self.tick_count += 1;
			if self.tick_count % 10 == 0 {
				self.log(format!("Simulation tick: {}", self.tick_count));
			}
		}
	}

	pub fn toggle_pause(&mut self) {
		self.is_paused = !self.is_paused;
		let status = if self.is_paused { "paused" } else { "resumed" };
		self.log(format!("Simulation {status}."));
	}

	/// Executes a menu action against the simulation.
	pub fn apply(&mut self, action: MenuAction) {
		match action {
			MenuAction::Spawn(kind) => {
				let id = self.world.spawn(kind);
				if let Some(obj) = self.world.get(id) {
					let msg = format!("Spawned {}.", obj.name);
					self.log(msg);
				}
			}
			MenuAction::List(kind) => {
				let names: Vec<&str> = self.world.of_kind(kind).map(|o| o.name.as_str()).collect();
				let msg = if names.is_empty() {
					format!("No {} objects.", kind.label())
				} else {
					names.join(", ")
				};
				self.log(msg);
			}
			MenuAction::TogglePause => self.toggle_pause(),
			MenuAction::Quit => self.should_quit = true,
			MenuAction::Unimplemented => self.log("Not implemented yet."),
		}
	}
}

/// What a menu item does when activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
	Spawn(ObjectKind),
	List(ObjectKind),
	TogglePause,
	Quit,
	Unimplemented,
}

/// The main loop runner for the RPG Simulator.
pub struct EngineLoop {
	pub target_fps: u64,
}

impl EngineLoop {
	pub fn new(target_fps: u64) -> Self {
		Self { target_fps: target_fps.max(1) }
	}

	/// Runs the unified event and simulation loop.
	pub fn run<B: Backend>(
		&self,
		terminal: &mut Terminal<B>,
		state: &mut RpgState,
		menu: &mut MenuSystem,
	) -> io::Result<()> {
		let tick_rate = Duration::from_millis(1000 / self.target_fps);
		let mut last_tick = Instant::now();

		while !state.should_quit {
			// 1. Render Frame
			terminal
				.draw(|f| render_ui(f, state, menu))
				.map_err(|err| io::Error::other(err.to_string()))?;

			// 2. Poll & Handle Input Events
			let timeout = tick_rate.saturating_sub(last_tick.elapsed());
			if event::poll(timeout)? {
				if let CrosstermEvent::Key(key) = event::read()? {
					if key.kind == KeyEventKind::Press {
						handle_key(key.code, state, menu);
					}
				}
			}

			// 3. Update Simulation Engine (Tick)
			if last_tick.elapsed() >= tick_rate {
				state.tick();
				last_tick = Instant::now();
			}
		}

		Ok(())
	}
}

fn handle_key(code: KeyCode, state: &mut RpgState, menu: &mut MenuSystem) {
	match code {
		KeyCode::Char('q') | KeyCode::Char('Q') => state.should_quit = true,
		KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => state.toggle_pause(),
		KeyCode::Left => menu.previous_category(),
		KeyCode::Right => menu.next_category(),
		KeyCode::Up => menu.previous_item(),
		KeyCode::Down => menu.next_item(),
		KeyCode::Enter => {
			if let Some(item) = menu.selected() {
				state.apply(item.action);
			}
		}
		_ => {}
	}
}

/// Draws the complete UI layout onto the terminal frame.
fn render_ui(frame: &mut Frame, state: &RpgState, menu: &MenuSystem) {
	let [menu_area, header, body, footer] = Layout::vertical([
		Constraint::Length(3),
		Constraint::Length(5),
		Constraint::Min(5),
		Constraint::Length(3),
	])
	.margin(1)
	.areas(frame.area());
	let [items, logs] = Layout::horizontal([Constraint::Length(28), Constraint::Min(10)]).areas(body);

	frame.render_widget(menu.render_tabs(), menu_area);
	render_header(frame, header, state);
	render_menu_items(frame, items, menu);
	render_viewport(frame, logs, state);
	render_footer(frame, footer);
}

fn render_header(frame: &mut Frame, area: Rect, state: &RpgState) {
	let status_str = if state.is_paused { "PAUSED" } else { "RUNNING" };
	let status_color = if state.is_paused { Color::Yellow } else { Color::Green };

	let text = vec![
		Line::from(vec![
			Span::styled(&state.name, Style::default().add_modifier(Modifier::BOLD)),
			Span::raw(" v"),
			Span::raw(&state.version),
		]),
		Line::from(vec![
			Span::raw("Status: "),
			Span::styled(status_str, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
			Span::raw(format!(" | Tick: {} | Objects: {}", state.tick_count, state.world.count())),
		]),
	];

	let widget = Paragraph::new(text).block(
		Block::default()
			.title(" Engine Control ")
			.borders(Borders::ALL)
			.border_style(Style::default().fg(Color::Cyan)),
	);
	frame.render_widget(widget, area);
}

fn render_menu_items(frame: &mut Frame, area: Rect, menu: &MenuSystem) {
	let Some(category) = menu.categories.get(menu.selected_category) else { return };

	let items: Vec<ListItem> = category
		.items
		.iter()
		.map(|item| ListItem::new(item.label.as_str()))
		.collect();

	let list = List::new(items)
		.block(
			Block::default()
				.title(format!(" {} ", category.title))
				.borders(Borders::ALL)
				.border_style(Style::default().fg(Color::Gray)),
		)
		.highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
		.highlight_symbol("> ");

	let mut list_state = ListState::default().with_selected(Some(menu.selected_item));
	frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_viewport(frame: &mut Frame, area: Rect, state: &RpgState) {
	let mut log_lines: Vec<Line> = state
		.log_messages
		.iter()
		.map(|msg| Line::from(Span::raw(msg)))
		.collect();

	if log_lines.is_empty() {
		log_lines.push(Line::from("No log output..."));
	}

	let widget = Paragraph::new(log_lines).block(
		Block::default()
			.title(" Simulation Logs ")
			.borders(Borders::ALL)
			.border_style(Style::default().fg(Color::Gray)),
	);
	frame.render_widget(widget, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
	let key = Style::default().fg(Color::Yellow);
	let text = Line::from(vec![
		Span::raw("[ "),
		Span::styled("SPACE/P", key),
		Span::raw(" ] Pause/Resume | [ "),
		Span::styled("Q", Style::default().fg(Color::Red)),
		Span::raw(" ] Quit | [ "),
		Span::styled("\u{2190}\u{2192}", key),
		Span::raw(" ] Tabs | [ "),
		Span::styled("\u{2191}\u{2193}/ENTER", key),
		Span::raw(" ] Select"),
	]);

	let widget = Paragraph::new(text).block(
		Block::default()
			.title(" Controls ")
			.borders(Borders::ALL)
			.border_style(Style::default().fg(Color::DarkGray)),
	);
	frame.render_widget(widget, area);
}

#[derive(Debug, Clone)]
pub struct MenuItem {
	pub label: String,
	pub action: MenuAction,
}

/// Represents a loaded menu hierarchy with active navigation indices.
#[derive(Debug, Clone, Default)]
pub struct MenuCategory {
	pub title: String,
	pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone, Default)]
pub struct MenuSystem {
	pub categories: Vec<MenuCategory>,
	pub selected_category: usize,
	pub selected_item: usize,
}

/// Steps `index` by ±1 within `0..len`, wrapping at both ends.
fn wrap_step(index: usize, len: usize, forward: bool) -> usize {
	if len == 0 {
		0
	} else if forward {
		(index + 1) % len
	} else {
		(index + len - 1) % len
	}
}

impl MenuSystem {
	pub fn next_category(&mut self) {
		self.selected_category = wrap_step(self.selected_category, self.categories.len(), true);
		self.selected_item = 0;
	}

	pub fn previous_category(&mut self) {
		self.selected_category = wrap_step(self.selected_category, self.categories.len(), false);
		self.selected_item = 0;
	}

	pub fn next_item(&mut self) {
		self.selected_item = wrap_step(self.selected_item, self.current_items().len(), true);
	}

	pub fn previous_item(&mut self) {
		self.selected_item = wrap_step(self.selected_item, self.current_items().len(), false);
	}

	/// Returns the highlighted item, if any.
	pub fn selected(&self) -> Option<&MenuItem> {
		self.current_items().get(self.selected_item)
	}

	fn current_items(&self) -> &[MenuItem] {
		self.categories
			.get(self.selected_category)
			.map_or(&[], |c| c.items.as_slice())
	}

	/// Renders the menu category headers as a Ratatui Tabs widget.
	pub fn render_tabs(&self) -> Tabs<'_> {
		let titles: Vec<Line> = self
			.categories
			.iter()
			.map(|c| Line::from(c.title.as_str()))
			.collect();

		Tabs::new(titles)
			.block(Block::default().title(" RPG Engine Menu ").borders(Borders::ALL))
			.select(self.selected_category)
			.style(Style::default().fg(Color::Gray))
			.highlight_style(
				Style::default()
					.fg(Color::Yellow)
					.add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
			)
	}
}

/// Single function to instantiate and load the menu system from structured data definitions.
pub fn load_menus(definitions: &[(&str, &[(&str, MenuAction)])]) -> MenuSystem {
	let categories = definitions
		.iter()
		.map(|(title, items)| MenuCategory {
			title: title.to_string(),
			items: items
				.iter()
				.map(|&(label, action)| MenuItem { label: label.to_string(), action })
				.collect(),
		})
		.collect();

	MenuSystem { categories, ..Default::default() }
}
