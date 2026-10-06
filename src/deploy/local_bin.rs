//! `--install`: copy the running executable into a per-user folder.
//!
//! | OS      | destination                              |
//! |---------|------------------------------------------|
//! | Windows | `%LOCALAPPDATA%\Programs\MyRPG\`         |
//! | others  | `$HOME/.local/bin/`                      |
//!
//! This only copies the file. It does not change `PATH`; whether you
//! can then run the program by name depends on whether that folder
//! is already on your `PATH`.

use super::TaskResult;
use std::{env, fs, path::PathBuf};

pub fn install() -> TaskResult {
	// The path of the executable that is running right now.
	let current_exe = env::current_exe()?;
	let exe_name = current_exe
		.file_name()
		.ok_or("Failed to determine binary filename")?;

	let deploy_dir = install_dir()?;
	fs::create_dir_all(&deploy_dir)?; // no error if it already exists

	// `fs::copy` replaces any file already at the destination.
	let target = deploy_dir.join(exe_name);
	fs::copy(&current_exe, &target)?;
	Ok(target)
}

/// `#[cfg(...)]` compiles exactly one of these two functions,
/// depending on the operating system being built for.
#[cfg(target_os = "windows")]
fn install_dir() -> Result<PathBuf, env::VarError> {
	let local_app_data = env::var("LOCALAPPDATA")?;
	Ok(PathBuf::from(local_app_data).join("Programs").join("MyRPG"))
}

#[cfg(not(target_os = "windows"))]
fn install_dir() -> Result<PathBuf, env::VarError> {
	let home = env::var("HOME")?;
	Ok(PathBuf::from(home).join(".local").join("bin"))
}
