use crate::{config::Config, contest, utils};
use clap::Args;
use log;
use noyalib as Yaml;
use std::{collections::HashMap, fs, io, path::PathBuf};

#[derive(Args)]
pub struct Arguments {
	/// Overwrite previous database entry for the problem
	#[arg(short, long, default_value_t = false)]
	pub overwrite: bool,

	pub problems: Vec<String>,
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	if args.problems.is_empty() {
		log::error!("Expected problem name !") // TODO: logopt::HELP
	}

	for source in &args.problems {
		add_problem(source, args, opts);
	}
	Ok(())
}

fn add_problem(source: &String, args: &Arguments, opts: &Config) {
	let pb_path = contest::get_path(&source, opts);
	let pb_name = contest::get_name(&source, opts);

	if !args.overwrite && pb_path.exists() {
		log::error!(
			"Cannot add {0}: entry already present in database\n\
			Use 'oly edit {0}' to edit it\n\
			Or use --overwrite/-o to overwrite it",
			pb_name
		);
		return;
	}

	let tmp_path = opts.tmpdir.join(&pb_name);
	let body = get_solution_body(&tmp_path, &pb_name, &opts).unwrap();
	let metadata = get_solution_metadata(&tmp_path, &opts);
	// utils::figures::save(tmp_path, pb_path); // TODO
	create_solution_file(
		contest::get_solution_path(source, opts),
		body,
		metadata,
		opts,
	);
}

fn get_solution_body(
	base_path: &PathBuf,
	source: &String,
	opts: &Config,
) -> Result<String, io::Error> {
	utils::create_preview_file(source, opts);
	let mut shared = HashMap::new();
	shared.insert("source", source.clone());
	let contents = utils::expand_vars(
		opts.contents.get(opts),
		true,
		true,
		None,
		Some(opts),
		Some(&shared),
	);
	let path = base_path.join(format!("solution{}", opts.lang.ext()));
	utils::create(&path, &contents);
	utils::edit(&path, &opts.editor);
	Ok(fs::read_to_string(path)?.trim_start().to_string()) // TODO handle error
}

fn get_solution_metadata(path: &PathBuf, opts: &Config) -> Yaml::Value {
	let path = path.join("metadata.yaml");
	utils::create(&path, &opts.metadata); // TODO expand vars with TOPIC and source
	loop {
		utils::edit(&path, &opts.editor);
		if let Some(metadata) = utils::yaml::load(&path, "Couldn't get solution metadata") {
			return metadata;
		}
	}
}

fn create_solution_file(path: PathBuf, body: String, metadata: Yaml::Value, opts: &Config) {
	let metadata =
		Yaml::to_string(&metadata).expect("get_solution_metadata() should return valid yaml");
	let (open, close) = (opts.lang.comment_open(), opts.lang.comment_close());
	let contents = format!(
		"{open}\n\
		{metadata}\n\
		{close}\n\
		\n\n\
		{body}",
	);
	if let Some(parent) = &path.parent() {
		fs::create_dir_all(parent);
	}
	if let Err(e) = fs::write(path, &contents) {
		// dump contents to stdout so they aren't lost
		// (even though the files should remain in opts.tmpdir, but you know, just in case)
		log::error!("Couldn't create solution file: {}", e);
		print!("{}", contents);
	};
}
