use chrono::{Datelike, Local};
use memoize::memoize;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::{Config, utils};

fn topic_from_letter(letter: char) -> &'static str {
	match letter {
		'A' => "Algebra",
		'C' => "Combinatorics",
		'G' => "Geometry",
		'N' => "Number Theory",
		_ => "",
	}
}

/// The topic of a problem, derived from the letter prefix of its problem
/// number (`A` for Algebra, `C` for Combinatorics, ...). Only a leading
/// letter prefix yields a topic, like in the C++ version.
pub fn get_topic(source: &str) -> String {
	let problem = parsers::get_problem(source);
	let Some(first) = problem.chars().next() else {
		return String::new();
	};
	topic_from_letter(first).to_string()
}

fn get_contest_format(contest: &str, opts: &Config) -> Option<String> {
	if let Some(format) = opts.contest_format.get(contest) {
		return Some(format.clone());
	}
	for (k, v) in &opts.contest_format_prefix {
		if contest.starts_with(k) {
			return Some(v.clone());
		}
	}
	None
}

pub mod parsers {
	use super::*;

	pub fn get_contest(source: &str, opts: &Config) -> String {
		static CONTEST_REGEX: OnceLock<Regex> = OnceLock::new();
		let regex = CONTEST_REGEX
			.get_or_init(|| Regex::new(r"\b([^0-9/\-_ ](?:[^0-9/_]*)[^0-9/\-_ ])\b").unwrap());

		if let Some(caps) = regex.captures(source) {
			let mut contest = caps.get(1).unwrap().as_str().to_string();
			if let Some(abbr) = opts.abbreviations.get(&contest) {
				contest = abbr.clone();
			} else if get_contest_format(&contest, opts).is_none() {
				if contest.len() <= 4 && !contest.contains(' ') {
					contest = contest.to_uppercase();
				}
			}
			contest
		} else {
			String::new()
		}
	}

	pub fn get_year(source: &str) -> String {
		static YEAR_REGEX_4: OnceLock<Regex> = OnceLock::new();
		static YEAR_REGEX_2: OnceLock<Regex> = OnceLock::new();
		let r4 = YEAR_REGEX_4.get_or_init(|| Regex::new(r"(\b\d{4}\b)").unwrap());
		let r2 = YEAR_REGEX_2.get_or_init(|| Regex::new(r"(\b\d{2}\b)").unwrap());

		if let Some(caps) = r4.captures(source) {
			caps.get(1).unwrap().as_str().to_string()
		} else if let Some(caps) = r2.captures(source) {
			format!("20{}", caps.get(1).unwrap().as_str())
		} else {
			let now = Local::now();
			now.year().to_string()
		}
	}

	pub fn get_problem(source: &str) -> String {
		static PROBLEM_REGEX: OnceLock<Regex> = OnceLock::new();
		let regex =
			PROBLEM_REGEX.get_or_init(|| Regex::new(r"(^|\s)(([PACGN]|/)?(\d))(\s|$)").unwrap()); // TODO check removed lookaround still worky

		if let Some(caps) = regex.captures(source) {
			if caps.get(3).is_none() {
				format!("P{}", caps.get(4).unwrap().as_str())
			} else {
				let prefix = caps.get(3).unwrap().as_str();
				let digit = caps.get(4).unwrap().as_str();
				let problem = format!("{}{}", prefix, digit);
				problem
			}
		} else {
			String::new()
		}
	}

	pub fn get_date(source: &str) -> String {
		static DATE_REGEX: OnceLock<Regex> = OnceLock::new();
		let regex = DATE_REGEX
			.get_or_init(|| Regex::new(r"(\d{1,2})[/-](\d{1,2})(?:[/-](\d{2,4}))?").unwrap());

		let (day, month, year) = if let Some(caps) = regex.captures(source) {
			let d = caps.get(1).unwrap().as_str().parse::<u32>().unwrap_or(1);
			let m = caps.get(2).unwrap().as_str().parse::<u32>().unwrap_or(1);
			let mut y = if let Some(y_match) = caps.get(3) {
				y_match.as_str().parse::<i32>().unwrap_or(2000)
			} else {
				Local::now().year()
			};

			if y < 100 {
				y += 2000;
			}
			(d, m, y)
		} else {
			let now = Local::now();
			(now.day(), now.month(), now.year())
		};

		format!("{:02}-{:02}-{:04}", day, month, year)
	}
}

fn get_relative_path(source: &str, opts: &Config) -> PathBuf {
	let contest = parsers::get_contest(source, opts);
	let contest_format = get_contest_format(&contest, opts);

	let path = if let Some(format_str) = contest_format {
		// Precompute values to avoid closure borrow issues with `self.shared` mutability
		let date_val = parsers::get_date(source);
		let contest_val = contest.clone();
		let year_val = parsers::get_year(source);
		let problem_val = parsers::get_problem(source);
		let source_val = source.to_string();

		let expander = |var: &str| -> String {
			match var {
				"date" => date_val.clone(),
				"contest" => contest_val.clone(),
				"year" => year_val.clone(),
				"problem" => problem_val.clone(),
				"source" => source_val.clone(),
				_ => String::new(),
			}
		};
		// TODO: pass topic
		PathBuf::from(utils::base_expand_vars(&format_str, expander))
	} else if !contest.is_empty() {
		let year = parsers::get_year(source);
		if !year.is_empty() {
			let problem = parsers::get_problem(source);
			if !problem.is_empty() {
				PathBuf::from(format!(
					"{}/{contest} {year}/{contest} {year} {problem}",
					contest
				))
			} else {
				PathBuf::from(format!("{}/{}", contest, source))
			}
		} else {
			PathBuf::from(format!("{}/{}", contest, source))
		}
	} else {
		PathBuf::from(source)
	};

	path
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ConfigSnapshot {
	base_path: PathBuf,
	lang: crate::config::Lang,
	abbreviations: Vec<(String, String)>,
	contest_format: Vec<(String, String)>,
	contest_format_prefix: Vec<(String, String)>,
}

fn sorted_map_entries(map: &HashMap<String, String>) -> Vec<(String, String)> {
	let mut entries: Vec<_> = map
		.iter()
		.map(|(key, value)| (key.clone(), value.clone()))
		.collect();
	entries.sort_by(|(left, _), (right, _)| left.cmp(right));
	entries
}

fn snapshot(opts: &Config) -> ConfigSnapshot {
	ConfigSnapshot {
		base_path: opts.base_path.clone(),
		lang: opts.lang,
		abbreviations: sorted_map_entries(&opts.abbreviations),
		contest_format: sorted_map_entries(&opts.contest_format),
		contest_format_prefix: sorted_map_entries(&opts.contest_format_prefix),
	}
}

fn snapshot_config(snapshot: &ConfigSnapshot) -> Config {
	let mut opts = Config::default();
	opts.base_path = snapshot.base_path.clone();
	opts.lang = snapshot.lang;
	opts.abbreviations = snapshot.abbreviations.iter().cloned().collect();
	opts.contest_format = snapshot.contest_format.iter().cloned().collect();
	opts.contest_format_prefix = snapshot.contest_format_prefix.iter().cloned().collect();
	opts
}

#[memoize]
fn get_path_memoized(source: String, snapshot: ConfigSnapshot) -> PathBuf {
	let opts = snapshot_config(&snapshot);
	let base_path = PathBuf::from(utils::expand_env_vars(opts.base_path.to_str().unwrap()));
	let source_path = get_relative_path(&source, &opts);
	base_path.join(source_path)
}

pub fn get_path(source: &str, opts: &Config) -> PathBuf {
	get_path_memoized(source.to_string(), snapshot(opts))
}

#[memoize]
fn get_solution_path_memoized(source: String, snapshot: ConfigSnapshot) -> PathBuf {
	let opts = snapshot_config(&snapshot);
	let path = get_path(&source, &opts);
	let ext = opts.lang.ext();
	path.join(format!("solution{}", ext))
}

pub fn get_solution_path(source: &str, opts: &Config) -> PathBuf {
	get_solution_path_memoized(source.to_string(), snapshot(opts))
}

#[memoize]
fn get_name_memoized(source: String, snapshot: ConfigSnapshot) -> String {
	let opts = snapshot_config(&snapshot);
	get_name_uncached(&source, &opts)
}

pub fn get_name(source: &str, opts: &Config) -> String {
	get_name_memoized(source.to_string(), snapshot(opts))
}

fn get_name_uncached(source: &str, opts: &Config) -> String {
	let contest = parsers::get_contest(source, opts);
	let contest_format = get_contest_format(&contest, opts);

	if let Some(format_str) = contest_format {
		let mut name = contest.clone();
		let mut ignored = HashSet::new();

		if format_str.contains("${date}") {
			ignored.insert("year".to_string());
		}

		static VAR_REGEX: OnceLock<Regex> = OnceLock::new();
		let var_regex = VAR_REGEX.get_or_init(|| Regex::new(r"\$\{([^}]+)\}").unwrap());

		// Emulating the C++ regex iterator loop
		for caps in var_regex.captures_iter(&format_str) {
			let str_val = caps.get(1).unwrap().as_str();

			if ignored.contains(str_val) {
				continue;
			} else {
				ignored.insert(str_val.to_string());
			}

			match str_val {
				"date" => {
					name.push_str(&format!(" {}", parsers::get_date(source)));
				}
				"year" => {
					name.push_str(&format!(" {}", parsers::get_year(source)));
				}
				"problem" => {
					name.push_str(&format!(" {}", parsers::get_problem(source)));
				}
				"source" => {
					name = source.to_string();
					break;
				}
				_ => {}
			}
		}
		name
	} else if !contest.is_empty() {
		let year = parsers::get_year(source);
		if !year.is_empty() {
			let problem = parsers::get_problem(source);
			if !problem.is_empty() {
				let name = format!("{} {} {}", contest, year, problem);
				return name;
			}
		}
		source.to_string()
	} else {
		source.to_string()
	}
}
