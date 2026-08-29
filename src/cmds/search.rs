use crate::{config::Config, utils};
use clap::Args;
use noyalib as Yaml;
use std::{fs, path::Path};

#[derive(Args)]
pub struct Arguments {
	#[arg(help = "Queries to filter problems (key:value or plain text, ANDed)")]
	pub queries: Vec<String>,
}

fn is_yaml(line: &str) -> bool {
	utils::is_yaml(line)
}

fn matches_queries(path: &Path, queries: &[String]) -> bool {
	let contents = match fs::read_to_string(path) {
		Ok(c) => c,
		Err(_) => return false,
	};
	let yaml_str = contents
		.lines()
		.skip(1)
		.take_while(|l| is_yaml(l))
		.collect::<Vec<_>>()
		.join("\n");
	let meta: Option<Yaml::Value> = Yaml::from_str(&yaml_str).ok();
	let haystack = {
		let mut h = String::new();
		if let Some(m) = &meta {
			// flatten metadata values into haystack
			h.push_str(&format!("{m:?}").to_lowercase());
			if let Some(s) = m.get("source").and_then(|v| v.as_str()) {
				h.push(' ');
				h.push_str(&s.to_lowercase());
			}
			if let Some(tags) = m.get("tags") {
				h.push(' ');
				h.push_str(&format!("{tags:?}").to_lowercase());
			}
		}
		h.push(' ');
		h.push_str(&contents.to_lowercase());
		h
	};

	for q in queries {
		let ql = q.to_lowercase();
		if let Some((field, val)) = ql.split_once(':') {
			let val = val.trim();
			let field_match = if let Some(m) = &meta {
				if let Some(v) = m.get(field.trim()) {
					format!("{v:?}").to_lowercase().contains(val)
				} else {
					// field not in metadata -> check haystack
					haystack.contains(&ql)
				}
			} else {
				haystack.contains(&ql)
			};
			if !field_match {
				return false;
			}
		} else if !haystack.contains(&ql) {
			return false;
		}
	}
	true
}

fn visit_and_search(
	path: &Path,
	queries: &[String],
	opts: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
	for entry in fs::read_dir(path)? {
		let entry = entry?;
		let p = entry.path();
		if p.is_dir() {
			visit_and_search(&p, queries, opts)?;
		} else if p.is_file() {
			// Only consider solution files to avoid noise from figures etc.
			let stem_ok = p.file_stem().and_then(|s| s.to_str()) == Some("solution");
			let ext_ok = p.extension().and_then(|e| e.to_str()).map(|e| {
				let lang_ext = opts.lang.ext().trim_start_matches('.');
				// If filter_lang is set, respect it; otherwise accept both
				if opts.filter_lang {
					e == lang_ext
				} else {
					e == "tex" || e == "typ"
				}
			}).unwrap_or(false);
			if !stem_ok {
				continue;
			}
			if !ext_ok {
				continue;
			}
			if matches_queries(&p, queries) {
				// extract source name for output (consistent with `list`)
				let yaml_str = fs::read_to_string(&p)
					.unwrap_or_default()
					.lines()
					.skip(1)
					.take_while(|l| is_yaml(l))
					.collect::<Vec<_>>()
					.join("\n");
				if let Ok(meta) = Yaml::from_str::<Yaml::Value>(&yaml_str) {
					if let Some(src) = meta.get("source").and_then(|v| v.as_str()) {
						println!("{src}");
						continue;
					}
				}
				// fallback: print path
				println!("{}", p.display());
			}
		}
	}
	Ok(())
}

pub fn run(args: &Arguments, opts: &Config) -> Result<(), Box<dyn std::error::Error>> {
	if args.queries.is_empty() {
		for problem in utils::prompt_user_for_problems() {
			println!("{problem}");
		}
		return Ok(());
	}
	if !opts.base_path.exists() {
		log::error!("{} does not exist !", opts.base_path.display());
		return Ok(());
	}
	visit_and_search(&opts.base_path, &args.queries, opts)?;
	Ok(())
}
