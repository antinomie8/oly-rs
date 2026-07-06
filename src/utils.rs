use log;
use noyalib as Yaml;
use regex::Regex;
use std::{
	collections::HashMap,
	fs::{self, File},
	io::{self, Write},
	path::PathBuf,
	process::Command,
	sync::OnceLock,
};

use crate::config::Config;

/// The underlying, generic string interpolation engine.
pub fn base_expand_vars<F>(str: &str, formatter: F) -> String
where
	F: Fn(&str) -> String,
{
	static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
	let regex = VAR_REGEX.get_or_init(|| Regex::new(r"\$\{([^}]+)\}").unwrap());

	// replace_all automatically passes the capture groups to the closure
	regex
		.replace_all(str, |caps: &regex::Captures| {
			let var_name = caps.get(1).unwrap().as_str();
			formatter(var_name)
		})
		.into_owned()
}

/// Expands configuration variables and/or environment variables inside a string.
pub fn expand_vars(
	source_str: &str,
	expand_config_vars: bool,
	expand_env_vars: bool,
	metadata: Option<Yaml::Value>,
	opts: Option<&Config>,
	shared: Option<&HashMap<&str, String>>,
) -> String {
	let formatter = |match_str: &str| -> String {
		if expand_config_vars {
			if let Some(metadata) = &metadata {
				if let Some(val) = metadata.get(match_str) {
					return val.clone().to_string();
				}
			}
			if let Some(opts) = &opts {
				if let Some(val) = opts.get(match_str) {
					return val;
				}
			}
			if let Some(shared) = &shared {
				if let Some(val) = shared.get(match_str) {
					return val.clone();
				}
			}
		}

		if expand_env_vars {
			if let Ok(val) = std::env::var(match_str) {
				return val;
			}
		}

		String::new()
	};

	let mut fmt = base_expand_vars(source_str, formatter);

	if expand_env_vars && fmt.starts_with("~/") {
		let home = std::env::var("HOME").unwrap_or_default();
		if let Some(suffix) = fmt.strip_prefix('~') {
			fmt = format!("{}{}", home, suffix);
		}
	}

	fmt
}

/// Convenience wrapper to expand only environment variables.
pub fn expand_env_vars(source_str: &str) -> String {
	expand_vars(source_str, false, true, None, None, None)
}

/// Create a file with intial contents
pub fn create(path: &PathBuf, contents: &String) {
	if let Some(parent) = &path.parent() {
		let _ = fs::create_dir_all(parent);
	}
	if let Some(err) = fs::write(&path, &contents).err() {
		log::error!("Cound't create {:?}: {}", path, err);
	}
}
/// edit a file by spawning an editor
pub fn edit(path: &PathBuf, editor: &String) {
	Command::new(&editor).arg(&path).status(); // TODO error handling
}

/// create a preview file for source in opts.tmpdir
pub fn create_preview_file(source: &String, opts: &Config) {
	let preview_path = opts
		.tmpdir
		.join(source)
		.join(format!("preview{}", opts.lang.ext()));
	let preview_contents = opts.preview.get(opts);
	fs::create_dir_all(
		&preview_path
			.parent()
			.expect("preview_path can't be filesystem root"),
	);
	fs::write(preview_path, preview_contents);
	// TODO: logging ?
}

/// prompt the user to press Enter to continue
pub fn wait() {
	let _ = io::stderr().write_all(b"\x1b[0;90mPress Enter to continue...\x1b[0m");
	let _ = io::stderr().flush();
	let mut buffer = String::new();
	let _ = io::stdin().read_line(&mut buffer);
}

pub mod yaml {
	use super::*;

	/// load a Yaml file, displaying any error to the user
	pub fn load(path: &PathBuf, errmsg: &str) -> Option<Yaml::Value> {
		let metadata: Result<Yaml::Value, Yaml::Error> = Yaml::from_reader(File::open(&path).ok()?);
		match metadata {
			Ok(data) => Some(data),
			Err(e) => {
				log::error!("{errmsg}: {e}");
				wait();
				None
			}
		}
	}
}

pub mod figures {}
