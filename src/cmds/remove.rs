use crate::{config::Config, contest, utils};
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct Arguments {
	/// Prompt before deleting file
	#[arg(short = 'i', long, default_value_t = false)]
	pub confirm: bool,

	/// Do not prompt before deleting file
	#[arg(short = 'f', long, default_value_t = false)]
	pub force: bool,

	pub problems: Vec<String>,
}

fn delete_problem(path: &std::path::Path, confirm: bool, opts: &Config) {
	if !path.exists() {
		log::error!("{} doesn't exist !", path.display());
		return;
	}
	if confirm && !utils::prompt_before_deletion(path) {
		return;
	}
	if let Err(err) = fs::remove_dir_all(path).or_else(|_| fs::remove_file(path)) {
		log::error!("{} couldn't be removed... ({})", path.display(), err);
	} else if let Some(parent) = path.parent() {
		utils::remove_empty_parents(parent.to_path_buf(), &opts.base_path);
	}
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	let problems = if args.problems.is_empty() {
		utils::prompt_user_for_problems()
	} else {
		args.problems.clone()
	};
	let confirm = if args.force {
		false
	} else {
		args.confirm || opts.confirm
	};
	for problem in problems {
		delete_problem(&contest::get_path(&problem, opts), confirm, opts);
	}
	Ok(())
}
