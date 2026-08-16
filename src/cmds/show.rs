use crate::{config::Config, config::Lang, contest, utils};
use clap::Args;
use regex::Regex;
use std::sync::OnceLock;
use std::{fs, io, io::IsTerminal};
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

const LATEX_QUERIES: &str = include_str!("../../assets/latex/highlights.scm");
const TYPST_QUERIES: &str = include_str!("../../assets/typst/highlights.scm");

static SET_REGEX: OnceLock<Regex> = OnceLock::new();

/// Remove nvim-specific `(#set! @capture key value)` predicates, which the
/// tree-sitter core query engine cannot parse but which only carry metadata.
fn sanitize_query(source: &str) -> String {
	let regex = SET_REGEX.get_or_init(|| Regex::new(r"\(\s*#set![^)]*\)").unwrap());
	regex.replace_all(source, "").into_owned()
}

static LATEX_QUERIES_SAN: OnceLock<String> = OnceLock::new();
static TYPST_QUERIES_SAN: OnceLock<String> = OnceLock::new();

fn get_queries(opts: &Config) -> &'static str {
	match opts.lang {
		Lang::Latex => LATEX_QUERIES_SAN.get_or_init(|| sanitize_query(LATEX_QUERIES)),
		Lang::Typst => TYPST_QUERIES_SAN.get_or_init(|| sanitize_query(TYPST_QUERIES)),
	}
}

static LATEX_LANG: OnceLock<Language> = OnceLock::new();
static TYPST_LANG: OnceLock<Language> = OnceLock::new();

fn get_language(opts: &Config) -> &'static Language {
	match opts.lang {
		Lang::Latex => LATEX_LANG.get_or_init(|| codebook_tree_sitter_latex::LANGUAGE.into()),
		Lang::Typst => TYPST_LANG.get_or_init(|| codebook_tree_sitter_typst::LANGUAGE.into()),
	}
}

#[derive(Args)]
pub struct Arguments {
	/// When to use terminal colours (always, auto, never)
	#[arg(long, default_value = "auto")]
	pub color: String,

	pub problems: Vec<String>,
}

fn to_ansi(color: i32) -> String {
	let r = ((color >> 16) & 0xFF) as u8;
	let g = ((color >> 8) & 0xFF) as u8;
	let b = (color & 0xFF) as u8;
	format!("\x1b[38;2;{};{};{}m", r, g, b)
}

fn map_capture(name: &str, opts: &Config) -> String {
	match name {
		"markup.strong" => "\x1b[1m".to_string(),
		"markup.italic" => "\x1b[3m".to_string(),
		_ => opts
			.colorscheme
			.get(name)
			.map(|color| to_ansi(*color))
			.unwrap_or_default(),
	}
}

fn colorize(input: &str, opts: &Config) -> String {
	if input.is_empty() {
		return input.to_string();
	}

	let language = get_language(opts);
	let queries = get_queries(opts);
	let query = match Query::new(language, queries) {
		Ok(query) => query,
		Err(err) => {
			log::error!("Could not compile syntax query: {}", err);
			return input.to_string();
		}
	};

	let mut parser = Parser::new();
	if let Err(err) = parser.set_language(language) {
		log::error!("Could not load the parser: {}", err);
		return input.to_string();
	}
	let Some(tree) = parser.parse(input, None) else {
		return input.to_string();
	};
	let root = tree.root_node();

	let mut color_buffer: Vec<String> = vec![String::new(); input.len()];
	let mut cursor = QueryCursor::new();
	let mut matches = cursor.matches(&query, root, input.as_bytes());
	while let Some(m) = matches.next() {
		for capture in m.captures {
			let name = query.capture_names()[capture.index as usize];
			let esc = map_capture(name, opts);
			if esc.is_empty() {
				continue;
			}
			let node = capture.node;
			let end = node.end_byte().min(input.len());
			for slot in &mut color_buffer[node.start_byte()..end] {
				*slot = esc.clone();
			}
		}
	}

	let mut result = String::with_capacity(input.len() + input.len() / 5);
	let mut last_color = "";
	for (index, ch) in input.char_indices() {
		let byte_color = color_buffer.get(index).map(String::as_str).unwrap_or("");
		if byte_color != last_color {
			result.push_str(if byte_color.is_empty() {
				"\x1b[0m"
			} else {
				byte_color
			});
			last_color = byte_color;
		}
		result.push(ch);
	}
	result.push_str("\x1b[0m");
	result
}

fn process(input: &str, color: &str, opts: &Config) -> String {
	let mut color_opt = color.to_string();
	if color_opt == "auto" {
		color_opt = if io::stdout().is_terminal() {
			"always".to_string()
		} else {
			"never".to_string()
		};
	}
	let trimmed = utils::trim_newlines(input);
	if color_opt == "always" {
		format!("{}\n", colorize(&trimmed, opts))
	} else {
		if color_opt != "never" {
			log::warn!(
				"Invalid value for --color: should be one of auto, never or always (received {})",
				color
			);
		}
		format!("{trimmed}\n")
	}
}

fn get_statement(path: &std::path::Path, opts: &Config, color: &str) -> Result<String, io::Error> {
	let contents = fs::read_to_string(path)?;
	let mut lines = contents.lines();
	let mut statement = String::new();

	for line in lines.by_ref() {
		if !utils::is_yaml(line)
			&& !utils::should_ignore(line, opts)
			&& !utils::is_package_import(line, opts)
		{
			if !utils::is_separator(line, opts) {
				statement.push_str(line);
				statement.push('\n');
			}
			break;
		}
	}

	for line in lines {
		if utils::is_separator(line, opts) {
			return Ok(process(&statement, color, opts));
		}
		statement.push_str(line);
		statement.push('\n');
	}

	Ok(process(&statement, color, opts))
}

fn print_statement(path: &std::path::Path, opts: &Config, color: &str) -> bool {
	if !path.exists() {
		log::error!("{} does not exist !", path.display());
		return false;
	}
	match get_statement(path, opts, color) {
		Ok(statement) => {
			print!("{}", statement);
			true
		}
		Err(err) => {
			log::error!("Could not open {}: {}", path.display(), err);
			false
		}
	}
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	let problems = if args.problems.is_empty() {
		utils::prompt_user_for_problems()
	} else {
		args.problems.clone()
	};

	let mut success = true;
	for (index, problem) in problems.iter().enumerate() {
		success = print_statement(
			&contest::get_solution_path(problem, opts),
			opts,
			&args.color,
		) && success;
		if index + 1 != problems.len() {
			println!();
		}
	}
	if success {
		Ok(())
	} else {
		Err("show failed".into())
	}
}
