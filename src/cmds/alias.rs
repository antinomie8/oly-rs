use crate::{config::Config, contest};
use clap::Args;
use std::fs;

#[derive(Args)]
pub struct Arguments {
	pub problems: Vec<String>,
}

fn symlink(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
	#[cfg(unix)]
	{
		std::os::unix::fs::symlink(from, to)
	}
	#[cfg(windows)]
	{
		if from.is_dir() {
			std::os::windows::fs::symlink_dir(from, to)
		} else {
			std::os::windows::fs::symlink_file(from, to)
		}
	}
	#[cfg(not(any(unix, windows)))]
	{
		let _ = (from, to);
		Err(std::io::Error::new(
			std::io::ErrorKind::Unsupported,
			"symlinks not supported on this platform",
		))
	}
}

fn link(from: &std::path::Path, to: &std::path::Path) {
	if from == to {
		log::error!("{} and {} are the same file", from.display(), to.display());
		return;
	}
	if to.exists() || to.is_symlink() {
		log::error!("{} already exists", to.display());
		return;
	}
	if let Some(parent) = to.parent() {
		if let Err(err) = fs::create_dir_all(parent) {
			log::error!("{}", err);
			return;
		}
	}
	if let Err(err) = symlink(from, to) {
		log::error!("failed to create symlink {} -> {}: {}", to.display(), from.display(), err);
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
		return Err(format!("cannot find {}", target.display()).into());
	}

	for destination in args.problems.iter().skip(1) {
		link(&target, &contest::get_path(destination, opts));
	}
	Ok(())
}
