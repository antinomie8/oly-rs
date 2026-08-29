use crate::{Arguments, config::Config, contest};
use std::io::{self, Write};

fn print_version() {
	println!("oly version {}", env!("CARGO_PKG_VERSION"));
	println!(
		"Build type: {}",
		if cfg!(debug_assertions) {
			"Debug"
		} else {
			"Release"
		}
	);
}

pub fn print_help() {
	println!("usage: oly <cmd> [args [...]].\n");
	println!("Available subcommands:");
	println!("    add                          - add a problem to the database");
	println!("    edit                         - edit an entry from the database");
	println!("    gen                          - generate a PDF from a problem");
	println!("    search                       - search problems by contest, metadata...");
	println!("    show                         - print a problem statement");
	println!("    list                         - list problems in the database");
	println!("    alias                        - link a problem to another one");
	println!("    rm                           - remove a problem and its solution file");
	println!("    mv                           - rename a problem");
	println!("  Run oly <cmd> --help for more information regarding a specific subcommand");
	println!();
	println!("Arguments:");
	println!("    --help              -h       - Show this help message");
	println!("    --config-file FILE  -c FILE  - Specify config file to use");
	println!("    --verify-config              - Check whether the config has any errors");
	println!("    --version           -v       - Print this binary's version");
	println!("    --log-level LEVEL            - Set the log level");
}

fn handle_scheme(request: &str, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	let request = if request.is_empty() {
		let mut pb_name = String::new();
		print!("Enter problem name: ");
		io::stdout().flush()?;
		io::stdin().read_line(&mut pb_name)?;
		let pb_name = pb_name.trim().to_string();
		if contest::get_path(&pb_name, opts).exists() {
			format!("oly://edit?name={pb_name}")
		} else {
			format!("oly://add?name={pb_name}")
		}
	} else {
		log::info!("received request: {request}");
		request.to_string()
	};

	let url = request.get(6..).unwrap_or("");
	let Some((cmd_name, query)) = url.split_once('?') else {
		log::error!("malformed query: expected format oly://cmd?name=<problem name>");
		return Err("malformed query".into());
	};
	let mut params = std::collections::HashMap::new();
	for pair in query.split('&') {
		if let Some((k, v)) = pair.split_once('=') {
			// Basic URL decode for common cases (%20 -> space)
			let decoded = v.replace("%20", " ").replace('+', " ");
			params.insert(k, decoded);
		}
	}
	let Some(pb_name) = params.get("name").cloned() else {
		log::error!("malformed query: expected format oly://cmd?name=<problem name>");
		return Err("malformed query".into());
	};
	let page = params.get("page").and_then(|v| v.parse::<u32>().ok());

	crate::logger::set_level(log::LevelFilter::Warn);
	crate::logger::set_scheme();
	unsafe { std::env::set_var("OLY", cmd_name) };

	match cmd_name {
		"add" => crate::cmds::add::run(
			&crate::cmds::add::Arguments {
				overwrite: false,
				problems: vec![pb_name],
			},
			opts,
		),
		"edit" => crate::cmds::edit::run(
			&crate::cmds::edit::Arguments {
				problems: vec![pb_name],
			},
			opts,
		),
		"show" => crate::cmds::show::run(
			&crate::cmds::show::Arguments {
				color: "auto".to_string(),
				problems: vec![pb_name],
			},
			opts,
		),
		"gen" => crate::cmds::generate::run(
			&crate::cmds::generate::Arguments {
				open: false,
				no_open: false,
				clean: false,
				no_pdf: false,
				no_source: false,
				cwd: false,
				print_path: false,
				clear_cache: false,
				regen: false,
				all: false,
				page,
				problems: vec![pb_name],
			},
			opts,
		),
		"rm" => crate::cmds::remove::run(
			&crate::cmds::remove::Arguments {
				confirm: false,
				force: false,
				problems: vec![pb_name],
			},
			opts,
		),
		_ => {
			log::error!("unknown command: {cmd_name}");
			Err("unknown command".into())
		}
	}
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	if args.version {
		print_version();
		return Ok(());
	}
	if args.verify_config {
		println!("All good !");
		return Ok(());
	}
	if let Some(request) = &args.scheme {
		return handle_scheme(request, opts);
	}
	print_help();
	Ok(())
}
