use crate::{config::Config, contest, utils};
use clap::Args;
use noyalib as Yaml;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Args)]
pub struct Arguments {
	/// Overwrite previous database entry for the problem
	#[arg(short, long, default_value_t = false)]
	pub overwrite: bool,

	pub problems: Vec<String>,
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	if args.problems.is_empty() {
		log::error!("Expected problem name !");
		crate::logger::help(Some("add"));
		return Err("Expected problem name".into());
	}

	for source in &args.problems {
		add_problem(source, args, opts);
	}
	Ok(())
}

fn add_problem(source: &String, args: &Arguments, opts: &Config) {
	let pb_path = contest::get_path(source, opts);
	let pb_name = contest::get_name(source, opts);

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
	let body = get_solution_body(&tmp_path, &pb_name, opts);
	let metadata = get_solution_metadata(&tmp_path, &pb_name, opts);
	utils::figures::save(&tmp_path, &pb_path, opts);
	create_solution_file(
		contest::get_solution_path(source, opts),
		body,
		metadata,
		opts,
	);
}

fn get_solution_body(base_path: &PathBuf, source: &String, opts: &Config) -> String {
	let mut shared = HashMap::new();
	shared.insert("source", source.clone());
	shared.insert("packages", opts.packages.get(opts).clone());
	utils::create_preview_file(source, opts, None, Some(&shared));
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
	match fs::read_to_string(&path) {
		Ok(content) => content.trim_start().to_string(),
		Err(e) => {
			log::error!(
				"failed to read {} file contents: {}",
				path.to_str().unwrap(),
				e
			);
			String::new()
		}
	}
}

fn get_solution_metadata(path: &PathBuf, source: &String, opts: &Config) -> Yaml::Value {
	let mut shared = HashMap::new();
	shared.insert("source", source.clone());
	shared.insert("packages", opts.packages.get(opts).clone());
	let topic = contest::get_topic(source);
	if !topic.is_empty() {
		shared.insert("topic", topic);
	}
	let path = path.join("metadata.yaml");
	let metadata = utils::expand_vars(
		opts.metadata.as_str(),
		true,
		true,
		None,
		Some(opts),
		Some(&shared),
	);
	utils::create(&path, &metadata);
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
		if let Err(err) = fs::create_dir_all(parent) {
			log::error!("Couldn't create solution directory: {}", err);
			return;
		}
	}
	if let Err(err) = fs::write(path, &contents) {
		log::error!("Couldn't create solution file: {}", err);
	};
}
