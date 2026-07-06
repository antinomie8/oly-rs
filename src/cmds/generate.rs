use crate::{config::Config, utils};
use clap::{ArgAction, Args};
use log;
use noyalib as Yaml;

#[derive(Args)]
pub struct Arguments {}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	Ok(())
}
