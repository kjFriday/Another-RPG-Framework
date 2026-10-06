//! `--dist`: build a `dist/` folder holding the executable and a
//! small metadata file, ready to zip and share.
//!
//! Caution: `dist` is relative to the *current working directory*
//! (where you ran the command), not to the project folder, and any
//! existing `dist` folder there is deleted first.

use super::TaskResult;
use std::{env, fs, path::Path};

pub fn package() -> TaskResult {
	let current_exe = env::current_exe()?;
	let exe_name = current_exe
		.file_name()
		.ok_or("Failed to determine binary filename")?;

	let dist_dir = Path::new("dist");
	if dist_dir.exists() {
		fs::remove_dir_all(dist_dir)?; // start clean
	}
	fs::create_dir_all(dist_dir)?;

	fs::copy(&current_exe, dist_dir.join(exe_name))?;

	// `env!` reads Cargo's variables at *compile* time, so these
	// come from Cargo.toml and are baked into the executable.
	let info = format!(
		"name = \"{}\"\nversion = \"{}\"\ndescription = \"{}\"\n",
		env!("CARGO_PKG_NAME"),
		env!("CARGO_PKG_VERSION"),
		"Extensible RPG Framework",
	);
	fs::write(dist_dir.join("project_info.toml"), info)?;

	// An absolute path for the message. On Windows `canonicalize`
	// returns the extended form, which starts with `\\?\`.
	Ok(dist_dir.canonicalize()?)
}
