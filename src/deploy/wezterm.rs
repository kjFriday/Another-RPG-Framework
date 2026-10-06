//! `--install-wezterm`: write a starter config for the WezTerm
//! terminal at `~/.config/wezterm/wezterm.lua`, one of the locations
//! WezTerm searches for its config file (see "Configuration Files" in
//! the WezTerm documentation). On Windows `~` is `%USERPROFILE%`.
//!
//! An existing config is never overwritten.

use super::TaskResult;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::{env, path::PathBuf};

/// The config, embedded in the executable at compile time. Each
/// setting is commented in the Lua itself, so the written file
/// explains itself too.
const WEZTERM_LUA: &str = r#"-- Written by: extensible_rpg_framework
--             --install-wezterm
local wezterm = require 'wezterm'
local config = wezterm.config_builder()

-- Draw with the WebGPU renderer.
config.front_end = "WebGpu"
-- One of WezTerm's built-in colour schemes.
config.color_scheme = 'Catppuccin Mocha'
-- 1.0 is opaque; 0.95 lets a little of the desktop show through.
config.window_background_opacity = 0.95
-- Consolas ships with Windows.
config.font = wezterm.font('Consolas')
config.font_size = 11.0
-- Starting window size, in character cells.
config.initial_cols = 100
config.initial_rows = 30

-- Space between the window edge and the text.
config.window_padding = {
  left = 12,
  right = 12,
  top = 12,
  bottom = 12,
}

-- Environment variables for programs started inside WezTerm.
-- COLORTERM=truecolor advertises 24-bit colour support.
config.set_environment_variables = {
  TERM = 'xterm-256color',
  COLORTERM = 'truecolor',
}

return config
"#;

pub fn install() -> TaskResult {
	let config_dir = home_dir()?.join(".config").join("wezterm");
	fs::create_dir_all(&config_dir)?;

	let target = config_dir.join("wezterm.lua");
	// `create_new(true)` fails with `AlreadyExists` if the file is
	// there, and the check and the creation happen in one step. A
	// separate `exists()` check could be beaten by another program
	// creating the file in between.
	let mut file = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(&target)
		.map_err(|e| format!("{}: {e}", target.display()))?;
	file.write_all(WEZTERM_LUA.as_bytes())?;
	Ok(target)
}

#[cfg(target_os = "windows")]
fn home_dir() -> Result<PathBuf, env::VarError> {
	env::var("USERPROFILE").map(PathBuf::from)
}

#[cfg(not(target_os = "windows"))]
fn home_dir() -> Result<PathBuf, env::VarError> {
	env::var("HOME").map(PathBuf::from)
}
