use crate::{config::Config, contest};
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct Arguments {
	pub problems: Vec<String>,
}

fn link(from: &std::path::Path, to: &std::path::Path) {
	if let Some(parent) = to.parent() {
		if let Err(err) = fs::create_dir_all(parent) {
			log::error!("{}", err);
			return;
		}
	}
	if let Err(err) = std::os::unix::fs::symlink(from, to) {
		log::error!("{}", err);
	}
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	if args.problems.is_empty() {
		log::error!("Expected source and at least one destination");
		crate::logger::help(Some("alias"));
		return Err("Expected source and at least one destination".into());
	}
	if args.problems.len() == 1 {
		log::error!("No destination provided");
		crate::logger::help(Some("alias"));
		return Err("No destination provided".into());
	}

	let target = contest::get_path(&args.problems[0], opts);
	if !target.exists() {
		log::error!(
			"cannot find {}: no such file or directory",
			target.display()
		);
		return Ok(()); // TODO: return an error
	}

	for destination in args.problems.iter().skip(1) {
		link(&target, &contest::get_path(destination, opts));
	}
	Ok(())
}
