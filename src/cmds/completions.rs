use crate::config::Config;
use clap::{Args, CommandFactory};
use clap_complete::Shell;
use regex::Regex;
use std::{
	fs,
	io::{self, Write},
	path::PathBuf,
};

#[derive(Args, Clone)]
pub struct Arguments {
	/// The shell to generate completions for
	#[arg(value_enum)]
	pub shell: Shell,

	/// Write the completion script to a file instead of stdout
	#[arg(short, long)]
	pub output: Option<PathBuf>,
}

pub fn run(args: &Arguments, _opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	let mut cmd = crate::Arguments::command();
	cmd.set_bin_name("oly");

	let mut buf = Vec::new();
	clap_complete::generate(args.shell, &mut cmd, "oly", &mut buf);

	let script = String::from_utf8(buf)?;
	let script = match args.shell {
		Shell::Zsh => inject_problems(&script),
		_ => script,
	};

	match &args.output {
		Some(path) => {
			if let Some(parent) = path.parent() {
				fs::create_dir_all(parent)?;
			}
			fs::write(path, script)?;
		}
		None => io::stdout().write_all(script.as_bytes())?,
	}
	Ok(())
}

fn inject_problems(script: &str) -> String {
	let script = script.replace(
		"'*::problems:_default'",
		"'*::problems:_oly_problems'",
	);

	let re_describe = Regex::new(r"_describe -t commands 'oly[^']*'").unwrap();
	let script = re_describe
		.replace_all(&script, "_describe -t subcommand 'subcommand'")
		.to_string();

	let script = strip_completions_subcommand(&script);

	let problems = "\n\
(( $+functions[_oly_problems] )) ||\n\
_oly_problems() {\n\
    local -a problems\n\
    problems=(\"${(@f)$(oly list 2>/dev/null)}\")\n\
    _describe -t problem 'problem' problems\n\
}\n";

	// Insert before the dispatch block so _oly_problems is defined before
	// _oly runs on first autoload (otherwise the first `oly edit <tab>`
	// fails with "command not found: _oly_problems").
	let re_dispatch =
		Regex::new(r#"\nif \[ "\$funcstack\[1\]" = "_oly" \]"#).unwrap();
	match re_dispatch.find(&script).map(|m| m.start()) {
		Some(pos) => {
			let mut script = script;
			script.insert_str(pos, problems);
			script
		}
		None => format!("{script}{problems}"),
	}
}

fn strip_completions_subcommand(script: &str) -> String {
	let re_case = Regex::new(r"(?s)\(completions\)\n.*?;;\n").unwrap();
	let re_list = Regex::new(r"'completions:[^']*' \\\n").unwrap();
	let re_func =
		Regex::new(r"(?s)\(\( \$\+functions\[_oly[^]]*completions[^]]*\] \)\) \|\|\n_oly[^()]*\(\) \{\n[^}]*\}\n")
			.unwrap();

	let script = re_case.replace_all(script, "").to_string();
	let script = re_list.replace_all(&script, "").to_string();
	re_func.replace_all(&script, "").to_string()
}
