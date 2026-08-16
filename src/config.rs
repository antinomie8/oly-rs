use crate::utils;
use etcetera::{BaseStrategy, choose_base_strategy};
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, env, fs, path::PathBuf};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
	Latex,
	Typst,
}

impl Lang {
	pub fn ext(&self) -> &'static str {
		match self {
			Lang::Latex => ".tex",
			Lang::Typst => ".typ",
		}
	}

	pub fn comment_open(&self) -> &'static str {
		match self {
			Lang::Latex => "\\iffalse",
			Lang::Typst => "/*",
		}
	}
	pub fn comment_close(&self) -> &'static str {
		match self {
			Lang::Latex => "\\fi",
			Lang::Typst => "*/",
		}
	}
}

#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(default)]
pub struct LangMap {
	latex: String,
	typst: String,
}
impl LangMap {
	pub fn get(&self, opts: &Config) -> &String {
		match &opts.lang {
			Lang::Latex => &self.latex,
			Lang::Typst => &self.typst,
		}
	}
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
	// the author's name in the preview and generated pdfs
	pub author: String,

	// the base path where solutions are stored
	pub base_path: PathBuf,

	// the path where figures are stored, inside of base_path
	pub figures_dir: PathBuf,

	// the markup language the solutions are typed in
	// must be one of latex or typst (default latex)
	pub lang: Lang,

	// language setting passed to the typst preview and preamble files.
	// must be a two or three letters language code
	pub language: String,

	// the text editor to use
	// autodected from $EDITOR and $VISUAL, fallback to xdg-open
	pub editor: String,

	// the pdf viewer used for 'oly gen'
	pub pdf_viewer: String,

	// whether to open the generated pdf with pdf_viewer when running 'oly gen'
	pub open: bool,

	// whether to confirm deletion or not in 'oly rm'
	pub confirm: bool,

	// whether to only output the entries written in the current markup language in 'oly list'
	pub filter_lang: bool,

	// map contest names to other ones
	pub abbreviations: HashMap<String, String>,

	// directory structure to store contests
	// available variables are: date, contest, year, problem and source
	pub contest_format: HashMap<String, String>,

	pub contest_format_prefix: HashMap<String, String>,

	// customize the colors used by the show command
	#[serde(default = "default_colorscheme", deserialize_with = "deserialize_colorscheme")]
	pub colorscheme: HashMap<String, i32>,

	// where to output the generated pdf
	// defaults to ${XDG_CACHE_HOME:-~/.cache}/oly/${source}
	pub output_directory: PathBuf,

	// where temporary files are written
	// defaults to ${TMPDIR:-/tmp}/oly
	pub tmpdir: PathBuf,

	// packages to import
	pub packages: LangMap,

	// what gets put in the preview file
	pub preview: LangMap,

	// the initial content in a file opened through 'oly add'
	pub contents: LangMap,

	// the preamble used by 'oly gen'
	pub preamble: LangMap,

	// metadata you get prompted for in 'oly add'
	pub metadata: String,
}

fn default_colorscheme() -> HashMap<String, i32> {
	let mut colors = HashMap::new();
	colors.insert("punctuation.special".into(), 0x7fb4ca);
	colors.insert("punctuation.delimiter".into(), 0x9cabca);
	colors.insert("punctuation.bracket".into(), 0x9cabca);
	colors.insert("operator".into(), 0xc0a36e);
	colors.insert("keyword.import".into(), 0xe46876);
	colors.insert("keyword".into(), 0x957fb8);
	colors.insert("keyword.repeat".into(), 0x957fb8);
	colors.insert("keyword.conditional".into(), 0x957fb8);
	colors.insert("number".into(), 0xd27e99);
	colors.insert("string".into(), 0x98bb6c);
	colors.insert("boolean".into(), 0xffa066);
	colors.insert("constant".into(), 0xffa066);
	colors.insert("variable.member".into(), 0xe6c384);
	colors.insert("function.call".into(), 0x7e9cd8);
	colors.insert("markup.heading.1".into(), 0x7e9cd8);
	colors.insert("markup.heading.2".into(), 0x7e9cd8);
	colors.insert("markup.heading.3".into(), 0x7e9cd8);
	colors.insert("markup.heading.4".into(), 0x7e9cd8);
	colors.insert("markup.heading.5".into(), 0x7e9cd8);
	colors.insert("markup.heading.6".into(), 0x7e9cd8);
	colors.insert("markup.link.url".into(), 0x7fb4ca);
	colors.insert("markup.raw".into(), 0x98bb6c);
	colors.insert("label".into(), 0x957fb8);
	colors.insert("markup.raw.block".into(), 0x98bb6c);
	colors.insert("markup.link.label".into(), 0x7fb4ca);
	colors.insert("markup.link".into(), 0x7fb4ca);
	colors.insert("markup.math".into(), 0xffa066);
	colors
}

fn deserialize_colorscheme<'de, D>(deserializer: D) -> Result<HashMap<String, i32>, D::Error>
where
	D: Deserializer<'de>,
{
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum ColorValue {
		Int(i32),
		Str(String),
	}

	let raw_map: HashMap<String, ColorValue> = HashMap::deserialize(deserializer)?;
	let mut map = default_colorscheme();

	for (k, v) in raw_map {
		let color_int = match v {
			ColorValue::Int(i) => i,
			ColorValue::Str(s) => match i32::from_str_radix(s.trim_start_matches('#'), 16) {
				Ok(color) => color,
				Err(_) => {
					log::error!("{} is not a valid hex color code !", s);
					continue;
				}
			},
		};
		map.insert(k, color_int);
	}

	Ok(map)
}

fn get_editor() -> String {
	env::var("EDITOR").unwrap_or(env::var("VISUAl").unwrap_or("xdg-open".into())) // TODO cross platform
}

impl ::std::default::Default for Config {
	fn default() -> Self {
		let strategy = choose_base_strategy().unwrap();
		Self {
			author: "".into(),
			base_path: strategy.data_dir().join("oly"),
			figures_dir: PathBuf::from("figures"),
			lang: Lang::Latex,
			language: "en".into(),
			editor: get_editor(),
			pdf_viewer: "xdg-open".into(), // TODO cross platform
			open: true,
			confirm: true,
			filter_lang: false,
			abbreviations: {
				let mut abbr = HashMap::new();
				abbr.insert("Shortlist".into(), "ISl".into());
				abbr
			},
			contest_format: HashMap::new(),
			contest_format_prefix: HashMap::new(),
			colorscheme: default_colorscheme(),
			output_directory: strategy.cache_dir().join("oly"),
			tmpdir: PathBuf::from(env::var("TMPDIR").unwrap_or("/tmp".into())).join("oly"),
			packages: LangMap {
				latex: "".into(),
				typst: "#import \"@local/oly:1.0.0\": *".into(),
			}, // TODO configure only one of them
			preview: LangMap {
				latex: include_str!("../assets/latex/preview.tex").into(),
				typst: include_str!("../assets/typst/preview.typ").into(),
			}, // TODO configure only one of them
			contents: LangMap {
				latex: include_str!("../assets/latex/contents.tex").into(),
				typst: include_str!("../assets/typst/contents.typ").into(),
			}, // TODO configure only one of them
			preamble: LangMap {
				latex: include_str!("../assets/latex/preamble.tex").into(),
				typst: include_str!("../assets/typst/preamble.typ").into(),
			}, // TODO configure only one of them
			metadata: include_str!("../assets/metadata.yaml").into(),
		}
	}
}

impl Config {
	pub fn load(path: &Option<PathBuf>) -> Self {
		let config_path = if let Some(path) = path {
			path
		} else {
			&choose_base_strategy()
				.unwrap()
				.config_dir()
				.join("oly/config.yaml")
		};
		let editor = get_editor();
		if !fs::exists(config_path).unwrap_or(false) {
			let default_config = include_str!("../assets/config.yaml");
			utils::create(config_path, &default_config.to_string());
			utils::edit(config_path, &editor);
		}

		loop {
			if let Some(config) = utils::yaml::load(config_path, "Couldn't load config") {
				match Config::deserialize(&config) {
					Ok(mut parsed_config) => {
						parsed_config.base_path = PathBuf::from(utils::expand_env_vars(
							parsed_config.base_path.to_str().unwrap_or(""),
						));
						parsed_config.output_directory = PathBuf::from(utils::expand_env_vars(
							parsed_config.output_directory.to_str().unwrap_or(""),
						));
						parsed_config.tmpdir = PathBuf::from(utils::expand_env_vars(
							parsed_config.tmpdir.to_str().unwrap_or(""),
						));
						break parsed_config;
					}
					Err(err) => {
						log::error!("Failed to deserialize config: {err}");
						utils::wait();
						utils::edit(config_path, &editor);
					}
				}
			} else {
				utils::edit(config_path, &editor);
			}
		}
	}

	pub fn get(&self, key: &str) -> Option<String> {
		match key {
			"author" => Some(self.author.clone()),
			"base_path" => Some(self.base_path.to_str().unwrap().to_string()),
			"figures_dir" => Some(self.figures_dir.to_str().unwrap().to_string()),
			"language" => Some(self.language.clone()),
			"editor" => Some(self.editor.clone()),
			"pdf_viewer" => Some(self.pdf_viewer.clone()),
			"packages" => Some(self.packages.get(self).clone()),
			"preview" => Some(self.preview.get(self).clone()),
			"contents" => Some(self.contents.get(self).clone()),
			"preamble" => Some(self.preamble.get(self).clone()),
			"metadata" => Some(self.metadata.clone()),
			_ => None,
		}
	}
}
