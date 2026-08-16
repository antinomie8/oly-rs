use crate::{config::Config, utils};
use clap::Args;

#[derive(Args)]
pub struct Arguments {
	pub queries: Vec<String>,
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	if args.queries.is_empty() {
		for problem in utils::prompt_user_for_problems() {
			println!("{}", problem);
		}
	} else {
		if !opts.base_path.exists() {
			log::error!("{} does not exist !", opts.base_path.display());
		}
		// todo: implement actual metadata-based query system (skip for now)
	}
	Ok(())
}
