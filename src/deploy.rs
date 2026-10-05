use std::{
	env,
	fs,
	path::{Path, PathBuf},
};

pub struct Project {
	pub name: String,
	pub version: String,
	pub description: String,
}

/// Deploy Option 1: Installs the compiled binary into the user's system PATH
pub fn deploy_local_bin() -> Result<PathBuf, Box<dyn std::error::Error>> {
	println!("Deploying binary to user local path...");

	let current_exe = env::current_exe()?;
	let exe_name = current_exe
		.file_name()
		.ok_or("Failed to determine binary filename")?;

	// Determine OS-specific install folder
	#[cfg(target_os = "windows")]
	let deploy_dir = {
		let local_app_data = env::var("LOCALAPPDATA")?;
		PathBuf::from(local_app_data).join("Programs").join("MyRPG")
	};

	#[cfg(not(target_os = "windows"))]
	let deploy_dir = {
		let home = env::var("HOME")?;
		PathBuf::from(home).join(".local").join("bin")
	};

	// Ensure target folder exists
	fs::create_dir_all(&deploy_dir)?;

	let target_path = deploy_dir.join(exe_name);
	fs::copy(&current_exe, &target_path)?;

	println!("Successfully installed binary to: {target_path:?}");
	Ok(target_path)
}

/// Deploy Option 2: Packages executable & assets into a clean `./dist` release directory
pub fn deploy_to_dist(project: &Project) -> Result<PathBuf, Box<dyn std::error::Error>> {
	println!("Packaging release build to ./dist directory...");

	let current_exe = env::current_exe()?;
	let exe_name = current_exe
		.file_name()
		.ok_or("Failed to determine binary filename")?;

	// Set target deployment folder relative to workspace root
	let dist_dir = Path::new("dist");
	if dist_dir.exists() {
		fs::remove_dir_all(dist_dir)?; // Clean previous build
	}
	fs::create_dir_all(dist_dir)?;

	// 1. Copy Executable
	let target_exe = dist_dir.join(exe_name);
	fs::copy(&current_exe, &target_exe)?;

	// 2. Create default metadata / config file in dist
	let config_content = format!(
		"name = \"{}\"\nversion = \"{}\"\ndescription = \"{}\"\n",
		project.name, project.version, project.description
	);
	fs::write(dist_dir.join("project_info.toml"), config_content)?;

	println!("Deployment payload ready at: {:?}", dist_dir.canonicalize()?);
	Ok(dist_dir.to_path_buf())
}

/// Embeds and deploys the WezTerm Lua configuration file to the OS-specific WezTerm config path.
/// Refuses to overwrite an existing config.
pub fn deploy_wezterm_config() -> Result<PathBuf, Box<dyn std::error::Error>> {
	// 1. Embed the Lua configuration directly into the compiled binary at compile-time
	const WEZTERM_LUA_CONTENT: &str = r#"
local wezterm = require 'wezterm'
local config = wezterm.config_builder()

config.front_end = "WebGpu"
config.color_scheme = 'Catppuccin Mocha'
config.window_background_opacity = 0.95
config.font = wezterm.font('Consolas')
config.font_size = 11.0
config.initial_cols = 100
config.initial_rows = 30

config.window_padding = {
left = 12,
right = 12,
top = 12,
bottom = 12,
}

config.set_environment_variables = {
TERM = 'xterm-256color',
COLORTERM = 'truecolor',
}

return config
"#;

	// 2. Resolve the OS-specific target directory for WezTerm
	#[cfg(target_os = "windows")]
	let config_dir = {
		let user_profile = env::var("USERPROFILE")?;
		PathBuf::from(user_profile).join(".config").join("wezterm")
	};

	#[cfg(not(target_os = "windows"))]
	let config_dir = {
		let home = env::var("HOME")?;
		PathBuf::from(home).join(".config").join("wezterm")
	};

	// 3. Create target directory structure if it doesn't exist
	fs::create_dir_all(&config_dir)?;

	// 4. Write the Lua script file, leaving any existing config alone
	let target_file = config_dir.join("wezterm.lua");
	if target_file.exists() {
		return Err(format!("{target_file:?} already exists; not overwriting").into());
	}
	fs::write(&target_file, WEZTERM_LUA_CONTENT)?;

	println!("Successfully deployed WezTerm config to: {target_file:?}");
	Ok(target_file)
}
