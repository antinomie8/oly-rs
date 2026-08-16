use noyalib as Yaml;
use regex::Regex;
use std::{
	collections::HashMap,
	fs::{self, File},
	io::{self, Write},
	path::{Path, PathBuf},
	process::{Command, Stdio},
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
					if let Some(s) = val.as_str() {
						return s.to_string();
					}
					// scalars that aren't strings (numbers, booleans)
					if val.is_number() || val.is_bool() {
						return val.to_string();
					}
					// null / sequences / mappings are skipped, like the C++ version
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

pub fn expand_env_vars(source_str: &str) -> String {
	expand_vars(source_str, false, true, None, None, None)
}

// TODO put in a file module
pub fn create(path: &PathBuf, contents: &String) {
	if let Some(parent) = path.parent() {
		if let Err(err) = fs::create_dir_all(parent) {
			log::error!("Cound't create {:?}: {}", parent, err);
			return;
		}
	}
	if let Err(err) = fs::write(path, contents) {
		log::error!("Cound't create {:?}: {}", path, err);
	}
}

// TODO put in a file module
pub fn edit(path: &PathBuf, editor: &String) {
	if let Err(err) = Command::new(editor).arg(path).status() {
		log::error!("Couldn't run {}: {}", editor, err);
	}
}

pub fn create_preview_file(
	source: &String,
	opts: &Config,
	metadata: Option<Yaml::Value>,
	shared: Option<&HashMap<&str, String>>,
) {
	let preview_path = opts
		.tmpdir
		.join(source)
		.join(format!("preview{}", opts.lang.ext()));
	let preview_contents = opts.preview.get(opts);
	let mut shared = match shared {
		Some(shared) => shared.clone(),
		None => HashMap::new(),
	};
	shared.insert("packages", opts.packages.get(opts).clone());
	let contents =
		expand_vars(&preview_contents, true, true, metadata, Some(opts), Some(&shared));
	if let Some(parent) = preview_path.parent() {
		if let Err(err) = fs::create_dir_all(parent) {
			log::error!("Couldn't create {}: {}", parent.display(), err);
			return;
		}
	}
	if let Err(err) = fs::write(&preview_path, contents) {
		log::error!("Couldn't create {}: {}", preview_path.display(), err);
	}
}

pub fn wait() {
	if io::stderr()
		.write_all(b"\x1b[0;90mPress Enter to continue...\x1b[0m")
		.is_err()
	{
		return;
	}
	if io::stderr().flush().is_err() {
		return;
	}
	let mut buffer = String::new();
	if io::stdin().read_line(&mut buffer).is_err() {
		log::error!("Couldn't read input");
	}
}

pub fn is_yaml(line: &str) -> bool {
	static YAML_REGEX: OnceLock<Regex> = OnceLock::new();
	YAML_REGEX
		.get_or_init(|| Regex::new(r"^[A-Za-z]+:\s*.+$").unwrap())
		.is_match(line)
}

pub fn is_separator(line: &str, opts: &Config) -> bool {
	match opts.lang {
		crate::config::Lang::Latex => line.trim() == r"\hrulebar",
		crate::config::Lang::Typst => line.trim() == "#divider()",
	}
}

pub fn is_package_import(line: &str, opts: &Config) -> bool {
	match opts.lang {
		crate::config::Lang::Latex => line.starts_with(r"\usepackage"),
		crate::config::Lang::Typst => line.starts_with("#import "),
	}
}

pub fn should_ignore(line: &str, opts: &Config) -> bool {
	let comment = match opts.lang {
		crate::config::Lang::Latex => line == r"\iffalse" || line == r"\fi",
		crate::config::Lang::Typst => line == "/*" || line == "*/",
	};
	comment || line.trim().is_empty()
}

pub fn trim_newlines(str: &str) -> String {
	str.trim_matches('\n').to_string()
}

pub fn is_executable(cmd: &str) -> bool {
	if cmd.contains('/') {
		return Path::new(cmd).is_file();
	}
	std::env::var_os("PATH")
		.map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd).is_file()))
		.unwrap_or(false)
}

pub fn run_command(args: &[String], silent: bool, detach: bool) -> io::Result<i32> {
	if args.is_empty() {
		return Ok(-1);
	}
	let mut command = Command::new(&args[0]);
	command.args(&args[1..]);
	if silent || detach {
		command.stdout(Stdio::null()).stderr(Stdio::null());
	}
	if detach {
		command.stdin(Stdio::null());
		command
			.spawn()
			.map(|child| i32::try_from(child.id()).unwrap_or(0))
	} else {
		command.status().map(|status| status.code().unwrap_or(-1))
	}
}

pub fn prompt_user_for_problems() -> Vec<String> {
	if !is_executable("fzf") {
		log::error!("fzf is not executable");
		return Vec::new();
	}
	if !is_executable("oly") {
		log::error!("oly is not executable");
		return Vec::new();
	}

	let output = Command::new("sh")
		.arg("-c")
		.arg(
			"oly list --print0 | fzf --read0 --print0 --multi --preview 'oly show --color=always {}'",
		)
		.output();
	match output {
		Ok(output) => output
			.stdout
			.split(|byte| *byte == 0)
			.filter(|chunk| !chunk.is_empty())
			.filter_map(|chunk| String::from_utf8(chunk.to_vec()).ok())
			.collect(),
		Err(err) => {
			log::error!("popen() failed: {}", err);
			Vec::new()
		}
	}
}

pub fn prompt_before_deletion(path: &Path) -> bool {
	eprint!(
		"Are you sure you want to remove {} ? [y/n] ",
		path.display()
	);
	let mut input = String::new();
	if io::stdin().read_line(&mut input).is_err() {
		return false;
	}
	input
		.chars()
		.next()
		.map(|ch| ch.eq_ignore_ascii_case(&'y'))
		.unwrap_or(false)
}

pub fn remove_empty_parents(mut path: PathBuf, base_path: &Path) {
	while !path.as_os_str().is_empty() && path != base_path {
		if !path.is_dir() {
			break;
		}
		let mut entries = match fs::read_dir(&path) {
			Ok(entries) => entries,
			Err(_) => break,
		};
		if entries.next().is_some() {
			break;
		}
		if let Err(err) = fs::remove_dir(&path) {
			log::error!("Couldn't remove {}: {}", path.display(), err);
			break;
		}
		if let Some(parent) = path.parent() {
			path = parent.to_path_buf();
		} else {
			break;
		}
	}
}

pub mod yaml {
	use super::*;

	pub fn load(path: &PathBuf, errmsg: &str) -> Option<Yaml::Value> {
		let metadata: Result<Yaml::Value, Yaml::Error> = Yaml::from_reader(File::open(path).ok()?);
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

pub mod figures {
	use super::*;

	fn copy_dir(from: &Path, to: &Path) -> bool {
		if !from.is_dir() {
			if to.exists() {
				return fs::remove_file(to)
					.or_else(|_| fs::remove_dir_all(to))
					.is_ok();
			}
			return false;
		}
		if let Err(err) = fs::create_dir_all(to) {
			log::error!("Error copying: {}", err);
			return false;
		}
		let entries = match fs::read_dir(from) {
			Ok(entries) => entries,
			Err(err) => {
				log::error!("Error copying: {}", err);
				return false;
			}
		};
		for entry in entries.flatten() {
			let src = entry.path();
			let dst = to.join(entry.file_name());
			if src.is_dir() {
				if !copy_dir(&src, &dst) {
					return false;
				}
			} else if let Err(err) = fs::copy(&src, &dst) {
				log::error!("Error copying: {}", err);
				return false;
			}
		}
		true
	}

	pub fn copy(tmp_path: &Path, pb_path: &Path, opts: &Config) -> bool {
		copy_dir(
			&pb_path.join(&opts.figures_dir),
			&tmp_path.join(&opts.figures_dir),
		)
	}

	pub fn save(tmp_path: &Path, pb_path: &Path, opts: &Config) -> bool {
		copy_dir(
			&tmp_path.join(&opts.figures_dir),
			&pb_path.join(&opts.figures_dir),
		)
	}
}
