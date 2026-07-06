mod cmds;
mod config;
mod contest;
mod utils;

use crate::config::Config;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Arguments {
	/// Sets a custom config file
	#[arg(short, long = "config-file", value_name = "FILE")]
	config_file: Option<PathBuf>,

	/// Set the log level
	#[arg(long = "log-level", default_value_t = simplelog::LevelFilter::Info)]
	log_level: simplelog::LevelFilter,

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// logging // TODO implement my own with notify-send on --scheme and --log-level arg
	simplelog::CombinedLogger::init(vec![simplelog::TermLogger::new(
		simplelog::LevelFilter::Warn,
		simplelog::Config::default(),
		simplelog::TerminalMode::Mixed,
		simplelog::ColorChoice::Auto,
	)])
	.unwrap();

	// parse cmdline arguments
	let cli = Arguments::parse();

	// load config file
	let config = Config::load(&cli.config_file);

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
		None => cmds::default::run(&cli, &config),
	}
}
