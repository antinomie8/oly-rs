mod cmds;
mod config;
mod contest;
mod logger;
mod utils;

use crate::config::{Config, Lang};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oly", about, long_about = None, disable_version_flag = true)]
pub struct Arguments {
	/// Specify config file to use
	#[arg(short = 'c', long = "config-file", value_name = "FILE", global = true)]
	pub config_file: Option<PathBuf>,

	/// Specify log level
	#[arg(long = "log-level", default_value_t = log::LevelFilter::Info, global = true)]
	pub log_level: log::LevelFilter,

	/// Choose markup language to use
	#[arg(long = "lang", global = true)]
	pub lang: Option<String>,

	/// Choose which language to use
	#[arg(long = "language", global = true)]
	pub language: Option<String>,

	/// Print program version and related info
	#[arg(short = 'v', long = "version", global = true)]
	pub version: bool,

	/// Check whether the config has any errors
	#[arg(long = "verify-config", global = true)]
	pub verify_config: bool,

	/// Use the scheme handler
	#[arg(long = "scheme", num_args = 0..=1, default_missing_value = "", global = true)]
	pub scheme: Option<String>,

	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
	/// Add a problem to the database
	Add(cmds::add::Arguments),
	/// Edit an entry from the database
	Edit(cmds::edit::Arguments),
	/// Generate a pdf for a problem
	Gen(cmds::generate::Arguments),
	/// Search problems by contest, metadata...
	Search(cmds::search::Arguments),
	/// Print a problem statement
	Show(cmds::show::Arguments),
	/// List problems in the database
	List(cmds::list::Arguments),
	/// Link a problem to another one
	Alias(cmds::alias::Arguments),
	/// Remove a problem and its solution file
	Rm(cmds::remove::Arguments),
	/// Rename a problem
	Mv(cmds::rename::Arguments),
	/// Generate shell completion scripts
	#[command(hide = true)]
	Completions(cmds::completions::Arguments),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Arguments::parse();
	logger::init(cli.log_level);

	let mut config = Config::load(&cli.config_file);

	if let Some(lang) = &cli.lang {
		config.lang = match lang.as_str() {
			"latex" => Lang::Latex,
			"typst" => Lang::Typst,
			_ => {
				log::error!("lang needs to be one of latex or typst !");
				std::process::exit(1);
			}
		};
	}
	if let Some(language) = &cli.language {
		config.language = language.clone();
	}

	unsafe {
		std::env::set_var(
			"OLY",
			match &cli.command {
				Some(Commands::Add(_)) => "add",
				Some(Commands::Edit(_)) => "edit",
				Some(Commands::Gen(_)) => "gen",
				Some(Commands::Search(_)) => "search",
				Some(Commands::Show(_)) => "show",
				Some(Commands::List(_)) => "list",
				Some(Commands::Alias(_)) => "alias",
				Some(Commands::Rm(_)) => "rm",
				Some(Commands::Mv(_)) => "mv",
				Some(Commands::Completions(_)) => "completions",
				None => "default",
			},
		);
	}

	match &cli.command {
		Some(Commands::Add(args)) => cmds::add::run(args, &config),
		Some(Commands::Edit(args)) => cmds::edit::run(args, &config),
		Some(Commands::Gen(args)) => cmds::generate::run(args, &config),
		Some(Commands::Search(args)) => cmds::search::run(args, &config),
		Some(Commands::Show(args)) => cmds::show::run(args, &config),
		Some(Commands::List(args)) => cmds::list::run(args, &config),
		Some(Commands::Alias(args)) => cmds::alias::run(args, &config),
		Some(Commands::Rm(args)) => cmds::remove::run(args, &config),
		Some(Commands::Mv(args)) => cmds::rename::run(args, &config),
		Some(Commands::Completions(args)) => cmds::completions::run(args, &config),
		None => cmds::default::run(&cli, &config),
	}
}
