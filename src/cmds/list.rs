use crate::{config::Config, contest};
use clap::{ArgAction, Args};
use noyalib as Yaml;
use regex::Regex;
use std::{error::Error, fs, path::Path, sync::OnceLock};

#[derive(Args)]
pub struct Arguments {
	/// Test whether get_problem_name output matches the source
	#[arg(short, long, default_value_t = false)]
	pub test: bool,

	/// Output entries separated by NUL bytes
	#[arg(long, default_value_t = false)]
	pub print0: bool,

	/// Only output the entries written in the current markup language
	#[arg(long, overrides_with = "no_filter_lang", action = ArgAction::SetTrue, default_value_t = false)]
	pub filter_lang: bool,
	/// Output entries regardless of the markup language used
	#[arg(long, overrides_with = "filter_lang", action = ArgAction::SetTrue, default_value_t = false)]
	pub no_filter_lang: bool,
}

fn is_yaml(line: &str) -> bool {
	static YAML_REGEX: OnceLock<Regex> = OnceLock::new();
	YAML_REGEX
		.get_or_init(|| Regex::new(r"^[A-Za-z]+:\s*.+$").unwrap())
		.is_match(line)
}

fn parse_metadata_from_file(solution_path: &Path) -> Option<String> {
	let contents = fs::read_to_string(solution_path)
		.map_err(|err| {
			log::error!("Could not open {}: {}", solution_path.display(), err);
			err
		})
		.ok()?;

	let metadata = contents
		.lines()
		.skip(1)
		.take_while(|line| is_yaml(line))
		.collect::<Vec<_>>()
		.join("\n");

	let yaml: Yaml::Value = match Yaml::from_str(&metadata) {
		Ok(yaml) => yaml,
		Err(err) => {
			log::error!("{}: {}", solution_path.display(), err);
			return None;
		}
	};

	yaml.get("source")
		.and_then(|source| source.as_str())
		.map(str::to_string)
}

fn visit_dir(
	path: &Path,
	args: &Arguments,
	opts: &Config,
	separator: char,
	filter_lang: bool,
) -> Result<(), Box<dyn Error>> {
	for entry in fs::read_dir(path)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			visit_dir(&path, args, opts, separator, filter_lang)?;
		} else if path.is_file() {
			if filter_lang
				&& path.extension().and_then(|ext| ext.to_str())
					!= opts.lang.ext().strip_prefix('.')
			{
				continue;
			}

			if let Some(source) = parse_metadata_from_file(&path) {
				if !args.test || source != contest::get_name(&source, opts) {
					print!("{source}{separator}");
				}
			}
		}
	}
	Ok(())
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn Error>> {
	let separator = if args.print0 { '\0' } else { '\n' };
	let filter_lang = args.filter_lang || (opts.filter_lang && !args.no_filter_lang);
	if let Err(err) = visit_dir(&opts.base_path, args, opts, separator, filter_lang) {
		log::error!("{}", err);
	}
	Ok(())
}
