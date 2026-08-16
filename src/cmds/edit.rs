use crate::{config::Config, contest, utils};
use clap::Args;
use noyalib as Yaml;
use std::path::Path;

#[derive(Args)]
pub struct Arguments {
	pub problems: Vec<String>,
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	let problems = if !args.problems.is_empty() {
		&args.problems
	} else {
		&utils::prompt_user_for_problems()
	};

	for source in problems {
		edit_problem(source, opts);
	}
	Ok(())
}

fn edit_problem(source: &String, opts: &Config) {
	let solution_path = contest::get_solution_path(source, opts);
	let pb_name = contest::get_name(source, opts);
	let contents = match get_solution(&solution_path, &pb_name, opts) {
		Ok(contents) => contents,
		Err(err) => {
			log::error!("{}", err);
			return;
		}
	};
	utils::create(&solution_path, &contents);
}

fn get_solution(solution_path: &Path, source: &String, opts: &Config) -> Result<String, std::io::Error> {
	let (metadata, solution) = parse_metadata_and_return_content(solution_path)?;
	let mut shared = std::collections::HashMap::new();
	shared.insert("source", source.clone());
	shared.insert("packages", opts.packages.get(opts).clone());
	utils::create_preview_file(source, opts, Some(metadata), Some(&shared));

	let tmp_path = opts.tmpdir.join(source);
	utils::figures::copy(&tmp_path, solution_path.parent().unwrap_or(Path::new(".")), opts);

	let tmp_file = tmp_path.join(format!("solution{}", opts.lang.ext()));
	utils::create(&tmp_file, &solution);
	utils::edit(&tmp_file, &opts.editor);
	utils::figures::save(&tmp_path, solution_path.parent().unwrap_or(Path::new(".")), opts);

	Ok(skip_leading_blank_lines(&std::fs::read_to_string(&tmp_file)?))
}

fn skip_leading_blank_lines(content: &str) -> String {
	let mut lines = content.lines();
	let mut out = String::new();
	while let Some(line) = lines.next() {
		if !line.trim().is_empty() {
			out.push_str(line);
			out.push('\n');
			for line in lines {
				out.push_str(line);
				out.push('\n');
			}
			return out;
		}
	}
	out
}

fn parse_metadata_and_return_content(
	solution_path: &Path,
) -> Result<(Yaml::Value, String), std::io::Error> {
	let content = std::fs::read_to_string(solution_path)?;
	let mut lines = content.lines();

	let mut solution = String::new();
	if let Some(first) = lines.next() {
		solution.push_str(first);
		solution.push('\n');
	}

	let mut metadata_str = String::new();
	for line in lines.by_ref() {
		solution.push_str(line);
		solution.push('\n');
		if !utils::is_yaml(line) {
			break;
		}
		metadata_str.push_str(line);
		metadata_str.push('\n');
	}

	for line in lines {
		solution.push_str(line);
		solution.push('\n');
	}

	let metadata = match Yaml::from_str(&metadata_str) {
		Ok(metadata) => metadata,
		Err(err) => {
			log::error!("Couldn't parse metadata from {}: {}", solution_path.display(), err);
			Yaml::from_str("").expect("empty string is valid yaml")
		}
	};
	Ok((metadata, solution))
}
