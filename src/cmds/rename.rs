use crate::{config::Config, contest, utils};
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct Arguments {
	/// Alias the new file to the old one
	#[arg(short, long, default_value_t = false)]
	pub alias: bool,

	pub problems: Vec<String>,
}

fn move_problem(from: &std::path::Path, to: &std::path::Path, alias: bool, opts: &Config) {
	if let Some(parent) = to.parent() {
		if let Err(err) = fs::create_dir_all(parent) {
			log::error!("{}", err);
			return;
		}
	}
	if let Err(err) = fs::rename(from, to) {
		log::error!("{}", err);
		return;
	}
	if let Some(parent) = from.parent() {
		utils::remove_empty_parents(parent.to_path_buf(), &opts.base_path);
	}
	if to.exists() {
		if alias {
			if let Err(err) = std::os::unix::fs::symlink(to, from) {
				log::error!("{}", err);
			}
		}
	} else {
		log::error!("cannot rename {} to {}", from.display(), to.display());
	}
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	match args.problems.len() {
		0 => {
			log::error!("missing file operand");
			crate::logger::help(Some("mv"));
			return Err("missing file operand".into());
		}
		1 => {
			log::error!(
				"missing destination file operand after '{}'",
				args.problems[0]
			);
			crate::logger::help(Some("mv"));
			return Err("missing destination file operand".into());
		}
		2 => {}
		_ => {
			log::error!("too many arguments provided: expected exactly two");
			crate::logger::help(Some("mv"));
			return Err("too many arguments provided".into());
		}
	}

	let target = contest::get_path(&args.problems[0], opts);
	if !target.exists() {
		log::error!(
			"cannot find {}: no such file or directory",
			target.display()
		);
		return Ok(());
	}
	move_problem(
		&target,
		&contest::get_path(&args.problems[1], opts),
		args.alias,
		opts,
	);
	Ok(())
}
