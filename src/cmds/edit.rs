use crate::{config::Config, contest, utils};
use clap::Args;
use log;
use noyalib as Yaml;
use std::{fs, io, path::PathBuf};

#[derive(Args)]
pub struct Arguments {
	pub problems: Vec<String>,
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	if args.problems.is_empty() {
		// TODO: prompt user for problems
		log::error!("Expected problem name !")
	}

	for source in &args.problems {
		edit_problem(source, args, opts);
	}
	Ok(())
}

fn edit_problem(source: &String, _args: &Arguments, opts: &Config) {
	let pb_path = contest::get_path(&source, opts);
	let pb_name = contest::get_name(&source, opts);

	let contents = get_solution(&pb_path, &pb_name, &opts).unwrap();
	utils::create(&pb_path, &contents);
}

fn get_solution(base_path: &PathBuf, source: &String, opts: &Config) -> Result<String, io::Error> {
	let (metadata, solution) = get_metadata_and_content(base_path, opts)?; // TODO use metadata
	utils::create_preview_file(source, opts);
	let tmp_path = opts.tmpdir.join(&source);
	// TODO utils::figures::copy
	let path = base_path.join(format!("solution{}", opts.lang.ext()));
	utils::create(&tmp_path, &solution);
	utils::edit(&tmp_path, &opts.editor);
	// TODO utils::figures::save
	Ok(fs::read_to_string(path)?.trim_start().to_string()) // TODO handle error
}

fn get_metadata_and_content(
	base_path: &PathBuf,
	opts: &Config,
) -> Result<(Yaml::Value, String), io::Error> {
	let content = fs::read_to_string(base_path)?;
	let metadata = content
		.lines()
		.enumerate()
		.find(|(_, l)| l.starts_with(opts.lang.comment_close()));
	if let Some((i, metadata)) = metadata {
		Ok((
			Yaml::from_str(metadata)
				.unwrap_or(Yaml::from_str("").expect("empty string is valid yaml")), // TODO log error
			content.lines().skip(i).fold("".into(), |acc, l| acc + l),
		))
	} else {
		Ok((
			Yaml::from_str("").expect("empty string is valid yaml"),
			content,
		))
	}
}
