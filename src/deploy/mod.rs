//! Install and packaging tasks, run from the command line only.
//!
//! None of this runs during a normal session; each task needs an
//! explicit flag (see [`run_flag`]). Each task lives in its own file:
//!
//! | flag                | file          | writes                      |
//! |---------------------|---------------|-----------------------------|
//! | `--install`         | `local_bin.rs`| a copy of this executable   |
//! | `--dist`            | `dist.rs`     | `./dist/` release folder    |
//! | `--install-wezterm` | `wezterm.rs`  | a WezTerm config, if absent |

mod dist;
mod local_bin;
mod wezterm;

use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;

/// What every task returns: the path it wrote, or an error.
///
/// `Box<dyn Error>` can hold any error type, which lets one function
/// use `?` on `io::Error`, `env::VarError` and plain `&str` messages
/// alike (the standard library converts each into the box).
type TaskResult = Result<PathBuf, Box<dyn Error>>;

/// Runs the task for `flag` and returns the process exit code:
/// success (0) if the task worked, failure (1) otherwise.
pub fn run_flag(flag: &str) -> ExitCode {
	let result = match flag {
		"--install" => local_bin::install(),
		"--dist" => dist::package(),
		"--install-wezterm" => wezterm::install(),
		_ => {
			eprintln!("Unknown argument: {flag}");
			eprintln!("Usage: [--install | --dist | --install-wezterm]");
			return ExitCode::FAILURE;
		}
	};
	match result {
		Ok(path) => {
			// `{:?}` on a path prints it quoted, with escapes.
			println!("Wrote {path:?}");
			ExitCode::SUCCESS
		}
		Err(err) => {
			eprintln!("Deployment failed: {err}");
			ExitCode::FAILURE
		}
	}
}
