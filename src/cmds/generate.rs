use crate::{config::Config, config::Lang, contest, utils};
use clap::Args;
use noyalib as Yaml;
use std::{
	collections::HashMap,
	error::Error,
	fs,
	path::{Path, PathBuf},
};

#[derive(Args, Clone)]
pub struct Arguments {
	/// Open the generated pdf
	#[arg(long, default_value_t = false)]
	pub open: bool,

	/// Do not open the generated pdf
	#[arg(long, default_value_t = false)]
	pub no_open: bool,

	/// Remove auxiliary files
	#[arg(long, default_value_t = false)]
	pub clean: bool,

	/// Only generate a source file
	#[arg(long, default_value_t = false)]
	pub no_pdf: bool,

	/// Remove the source file and induce --clean
	#[arg(long, default_value_t = false)]
	pub no_source: bool,

	/// Create the pdf in the current directory
	#[arg(long, default_value_t = false)]
	pub cwd: bool,

	/// Print the path of the directory where the pdf will be generated
	#[arg(short = 'p', long, default_value_t = false)]
	pub print_path: bool,

	/// Clear the cache
	#[arg(long, default_value_t = false)]
	pub clear_cache: bool,

	/// Regenerate the pdf even if it was cached
	#[arg(long, default_value_t = false)]
	pub regen: bool,

	/// Generate a pdf for each solution file
	#[arg(long, default_value_t = false)]
	pub all: bool,

	pub problems: Vec<String>,
}

fn get_solution_bodies(source: &Path, opts: &Config) -> Result<Vec<String>, Box<dyn Error>> {
	let contents = fs::read_to_string(source)?;
	let mut bodies = Vec::new();
	let mut body = String::new();
	let mut lines = contents.lines();
	let mut current = String::new();

	for line in lines.by_ref() {
		current = line.to_string();
		if !utils::is_yaml(line) && !utils::should_ignore(line, opts) {
			if utils::is_package_import(line, opts) {
				body.push_str(line);
				body.push('\n');
			} else {
				bodies.push(body);
				body = String::new();
				break;
			}
		}
	}

	if !current.is_empty() {
		if utils::is_separator(&current, opts) {
			bodies.push(body);
			body = String::new();
		} else {
			body.push_str(&current);
			body.push('\n');
		}
	}

	for line in lines {
		if utils::is_separator(line, opts) {
			bodies.push(body);
			body = String::new();
		} else {
			body.push_str(line);
			body.push('\n');
		}
	}
	bodies.push(body);
	Ok(bodies)
}

fn get_solution_metadata(source: &Path) -> Result<Yaml::Value, Box<dyn Error>> {
	let contents = fs::read_to_string(source)?;
	let yaml = contents
		.lines()
		.skip(1)
		.take_while(|line| utils::is_yaml(line))
		.collect::<Vec<_>>()
		.join("\n");
	Ok(Yaml::from_str(&yaml)?)
}

fn yaml_str<'a>(metadata: &'a Yaml::Value, key: &str) -> Option<&'a str> {
	metadata.get(key).and_then(|value| value.as_str())
}

fn output_source_path(source: &str, opts: &Config) -> PathBuf {
	let shared = HashMap::from([("source", source.to_string())]);
	let output_directory = utils::expand_vars(
		opts.output_directory.to_str().unwrap(),
		true,
		true,
		None,
		Some(opts),
		Some(&shared),
	);
	PathBuf::from(output_directory)
		.join(source)
		.join(format!("{}{}", source, opts.lang.ext()))
}

fn create_latex_file(
	problems: &[String],
	source: &str,
	output_path: &Path,
	opts: &Config,
) -> Result<(), Box<dyn Error>> {
	if let Some(parent) = output_path.parent() {
		fs::create_dir_all(parent)?;
	}
	let mut out = String::new();
	let shared = HashMap::from([
		("source", source.to_string()),
		("packages", opts.packages.get(opts).clone()),
	]);
	out.push_str(&utils::expand_vars(
		opts.preamble.get(opts),
		true,
		true,
		None,
		Some(opts),
		Some(&shared),
	));

	for problem in problems {
		let path = contest::get_solution_path(problem, opts);
		let bodies = get_solution_bodies(&path, opts)?;
		let metadata = get_solution_metadata(&path)?;
		if let Some(packages) = bodies.first() {
			out.push_str(packages);
		}
		out.push_str(if problems.len() > 1 {
			"\\begin{problem}"
		} else {
			"\\begin{problem*}"
		});
		if let Some(source) = yaml_str(&metadata, "source") {
			out.push_str(&format!(" [{}]", source));
		}
		out.push('\n');
		if let Some(statement) = bodies.get(1) {
			out.push_str(statement);
		}
		out.push_str(if problems.len() > 1 {
			"\\end{problem}"
		} else {
			"\\end{problem*}"
		});
		out.push_str("\n\n");
		if let Some(url) = yaml_str(&metadata, "url") {
			out.push_str(&format!("\\noindent\\emph{{Link}}: \\url{{{}}}\n\n", url));
		}
		for body in bodies.iter().skip(2) {
			out.push_str("\\hrulebar\n\n");
			out.push_str(body);
		}
		out.push_str("\n\\pagebreak\n\n");
	}
	out.push_str("\\end{document}\n");
	fs::write(output_path, out)?;
	Ok(())
}

fn create_typst_file(
	problems: &[String],
	source: &str,
	output_path: &Path,
	opts: &Config,
) -> Result<(), Box<dyn Error>> {
	if let Some(parent) = output_path.parent() {
		fs::create_dir_all(parent)?;
	}
	let shared = HashMap::from([
		("source", source.to_string()),
		("packages", opts.packages.get(opts).clone()),
	]);
	let mut out = String::new();
	if problems.len() != 1 {
		out.push_str(&utils::expand_vars(
			opts.preamble.get(opts),
			true,
			true,
			None,
			Some(opts),
			Some(&shared),
		));
	}
	for (index, problem) in problems.iter().enumerate() {
		let path = contest::get_solution_path(problem, opts);
		let bodies = get_solution_bodies(&path, opts)?;
		let metadata = get_solution_metadata(&path)?;
		if problems.len() == 1 {
			out.push_str(&utils::expand_vars(
				opts.preamble.get(opts),
				true,
				true,
				Some(metadata.clone()),
				Some(opts),
				Some(&shared),
			));
		}
		if let Some(packages) = bodies.first() {
			out.push_str(packages);
		}
		let is_problem = bodies.len() > 2;
		if !is_problem {
			if let Some(url) = yaml_str(&metadata, "url") {
				out.push_str(&format!("#link(\"{}\")[🔗_{} _]\n\n", url, url));
			}
		}
		if is_problem {
			out.push_str(if problems.len() > 1 {
				"#problem"
			} else {
				"#_problem"
			});
			match (yaml_str(&metadata, "source"), yaml_str(&metadata, "url")) {
				(Some(source), Some(url)) => {
					out.push_str(&format!("(\"{}\", link: \"{}\")", source, url))
				}
				(Some(source), None) => out.push_str(&format!("(\"{}\")", source)),
				(None, Some(url)) => out.push_str(&format!("(link: \"{}\")", url)),
				(None, None) => {}
			}
			out.push_str("[\n");
		}
		if let Some(statement) = bodies.get(1) {
			out.push_str(statement);
		}
		if is_problem {
			out.push(']');
		}
		out.push_str("\n\n");
		for (body_index, body) in bodies.iter().enumerate().skip(2) {
			if body_index == 2 {
				out.push_str(&format!("#solution[\n{}\n]", utils::trim_newlines(body)));
			} else {
				out.push_str("#divider()\n\n");
				out.push_str(body);
			}
		}
		if index + 1 != problems.len() {
			out.push_str("\n#pagebreak()\n\n");
		}
	}
	fs::write(output_path, out)?;
	Ok(())
}

fn compile_output(path: &Path, args: &Arguments, opts: &Config) {
	if args.no_pdf {
		return;
	}
	let open = if args.no_open {
		false
	} else {
		args.open || opts.open
	};
	if open && !utils::is_executable(&opts.pdf_viewer) {
		log::error!("{} is not executable", opts.pdf_viewer);
	}
	match opts.lang {
		Lang::Latex => {
			if !utils::is_executable("latexmk") {
				log::error!("latexmk is not executable");
				return;
			}
			let outdir = if args.cwd {
				std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
			} else {
				path.parent().unwrap_or(Path::new(".")).to_path_buf()
			};
			let mut cmd = vec![
				"latexmk".to_string(),
				"-pdf".to_string(),
				"-silent".to_string(),
			];
			if open {
				cmd.push("-e".to_string());
				cmd.push(format!("'$pdf_previewer=q[{} %S];'", opts.pdf_viewer));
			}
			cmd.push(format!("-outdir={}", outdir.display()));
			cmd.push(path.display().to_string());
			if let Err(err) = utils::run_command(&cmd, false, false) {
				log::error!("{}", err);
			}
			if args.clean || args.cwd || args.no_source {
				let mut exts = vec!["aux", "fdb_latexmk", "fls", "log", "pre"];
				if args.no_source {
					exts.push("tex");
				}
				for ext in exts {
					let mut aux = PathBuf::from(path.file_name().unwrap_or_default());
					aux.set_extension(ext);
					let _ = fs::remove_file(outdir.join(aux));
				}
			}
		}
		Lang::Typst => {
			if !utils::is_executable("typst") {
				log::error!("typst is not executable");
				return;
			}
			let root = path.parent().unwrap_or(Path::new("."));
			for problem in &args.problems {
				utils::figures::copy(root, &contest::get_path(problem, opts), opts);
			}
			let mut cmd = vec![
				"typst".to_string(),
				"compile".to_string(),
				"--root".to_string(),
				root.display().to_string(),
			];
			if open {
				cmd.push("--open".to_string());
				cmd.push(opts.pdf_viewer.clone());
			}
			cmd.push(path.display().to_string());
			if args.cwd {
				let pdf = path
					.file_name()
					.map(PathBuf::from)
					.unwrap_or_else(|| PathBuf::from("out.pdf"))
					.with_extension("pdf");
				cmd.push(
					std::env::current_dir()
						.unwrap_or_else(|_| PathBuf::from("."))
						.join(pdf)
						.display()
						.to_string(),
				);
			}
			if let Err(err) = utils::run_command(&cmd, false, false) {
				log::error!("{}", err);
			}
		}
	}
}

fn create_pdf(problems: &[String], source: &str, args: &Arguments, opts: &Config) {
	let output_path = output_source_path(source, opts);

	let regenerate = {
		let mut regenerate = args.regen;
		if !regenerate {
			match fs::metadata(&output_path) {
				Ok(output_meta) => {
					let output_time = output_meta.modified().unwrap_or(std::time::UNIX_EPOCH);
					let mut input_time = output_time;
					for problem in problems {
						if let Ok(meta) = fs::metadata(contest::get_solution_path(problem, opts)) {
							let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
							input_time = input_time.max(modified);
						}
					}
					regenerate = input_time > output_time;
				}
				Err(_) => regenerate = true,
			}
		}
		regenerate
	};

	if !regenerate {
		let pdf = output_path.with_extension("pdf");
		if let Err(err) = utils::run_command(
			&[opts.pdf_viewer.clone(), pdf.display().to_string()],
			true,
			true,
		) {
			log::error!("{}", err);
		}
		return;
	}

	let result = match opts.lang {
		Lang::Latex => create_latex_file(problems, source, &output_path, opts),
		Lang::Typst => create_typst_file(problems, source, &output_path, opts),
	};
	if let Err(err) = result {
		log::error!("Error generating {}: {}", source, err);
		return;
	}
	compile_output(&output_path, args, opts);
	if args.no_source {
		if let Err(err) = fs::remove_file(&output_path) {
			log::error!("{}", err);
		}
	}
	if args.print_path {
		let outdir = if args.cwd {
			std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
		} else {
			output_path.parent().unwrap_or(Path::new(".")).to_path_buf()
		};
		println!("{}", outdir.display());
	}
}

fn generate_all_in_dir(path: &Path, args: &Arguments, opts: &Config) -> Result<(), Box<dyn Error>> {
	for entry in fs::read_dir(path)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() {
			generate_all_in_dir(&path, args, opts)?;
		} else if path.is_file()
			&& path.file_stem().and_then(|stem| stem.to_str()) == Some("solution")
		{
			let metadata = get_solution_metadata(&path)?;
			let Some(source) = yaml_str(&metadata, "source") else {
				log::error!("No source entry found in {}", path.display());
				continue;
			};
			let mut file_opts = opts.clone();
			file_opts.lang = match path.extension().and_then(|ext| ext.to_str()) {
				Some("typ") => Lang::Typst,
				Some("tex") => Lang::Latex,
				_ => continue,
			};
			let mut file_args = args.clone();
			file_args.open = false;
			file_args.regen = true;
			create_pdf(&[source.to_string()], source, &file_args, &file_opts);
		}
	}
	Ok(())
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn Error>> {
	if args.clear_cache {
		let path = PathBuf::from(utils::expand_env_vars(
			opts.output_directory.to_str().unwrap(),
		));
		if utils::prompt_before_deletion(&path) {
			fs::remove_dir_all(path)?;
		}
		return Ok(());
	}

	if args.all {
		generate_all_in_dir(&opts.base_path, args, opts)?;
		return Ok(());
	}

	let problems = if args.problems.is_empty() {
		utils::prompt_user_for_problems()
	} else {
		args.problems.clone()
	};
	let source = problems
		.iter()
		.map(|problem| contest::get_name(problem, opts))
		.collect::<Vec<_>>()
		.join(" - ");
	create_pdf(&problems, &source, args, opts);
	Ok(())
}
