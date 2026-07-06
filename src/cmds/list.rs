use crate::{config::Config, utils};
use clap::{ArgAction, Args};
use log;
use noyalib as Yaml;
use std::{error::Error, fs};

#[derive(Args)]
pub struct Arguments {
	/// Test whether get_problem_name output matches the source
	#[arg(short, long, default_value_t = false)]
	pub test: bool,

	/// Output entries separated by NUL bytes
	#[arg(default_value_t = false)]
	pub print0: bool,

	/// Only output the entries written in the current markup language
	#[arg(long, overrides_with = "no_filter_lang", action = ArgAction::SetTrue, default_value_t = false)]
	pub filter_lang: bool,
	/// Output entries regardless of the markup language used
	#[arg(long, overrides_with = "filter_lang", action = ArgAction::SetFalse)]
	pub no_filter_lang: bool,
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn Error>> {
	let sep: char = if args.print0 { '\0' } else { '\n' };
	let paths = fs::read_dir(&opts.base_path).unwrap();
	for entry in fs::read_dir(".")? {
		let dir = entry?;
		println!("{:?}", dir.path());
	}
	Ok(())
}
