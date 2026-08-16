use ::log::{Level, LevelFilter, Metadata, Record};
use std::io::{self, IsTerminal, Write};
use std::process::Command;
use std::sync::Mutex;

static SCHEME: Mutex<bool> = Mutex::new(false);
static LEVEL: Mutex<LevelFilter> = Mutex::new(LevelFilter::Info);

pub fn set_scheme() {
	if let Ok(mut guard) = SCHEME.lock() {
		*guard = true;
	}
}

pub fn is_scheme() -> bool {
	match SCHEME.lock() {
		Ok(guard) => *guard,
		Err(_) => false,
	}
}

pub fn set_level(level: LevelFilter) {
	if let Ok(mut guard) = LEVEL.lock() {
		*guard = level;
	}
}

pub fn current_level() -> LevelFilter {
	match LEVEL.lock() {
		Ok(guard) => *guard,
		Err(poisoned) => *poisoned.into_inner(),
	}
}

pub fn init(level: LevelFilter) {
	set_level(level);
	let _ = ::log::set_boxed_logger(Box::new(OlyLogger));
	::log::set_max_level(LevelFilter::Trace);
}

struct OlyLogger;

impl ::log::Log for OlyLogger {
	fn enabled(&self, metadata: &Metadata) -> bool {
		metadata.level() <= current_level()
	}

	fn log(&self, record: &Record) {
		if !self.enabled(record.metadata()) {
			return;
		}
		if is_scheme() {
			notify_send(record);
		} else {
			log_stderr(record);
		}
	}

	fn flush(&self) {
		let _ = io::stderr().flush();
	}
}

fn severity_color(level: Level) -> &'static str {
	if !io::stderr().is_terminal() {
		return "";
	}
	match level {
		Level::Error => "\x1b[31m",
		Level::Warn => "\x1b[33m",
		Level::Info => "\x1b[34m",
		Level::Debug => "\x1b[90m",
		Level::Trace => "\x1b[2m",
	}
}

fn log_stderr(record: &Record) {
	let mut stderr = io::stderr().lock();
	let _ = write!(
		stderr,
		"{}{}\x1b[0m: {}\n",
		severity_color(record.level()),
		record.level(),
		record.args()
	);
}

fn notify_send(record: &Record) {
	let severity = record.level().to_string().to_lowercase();
	let data_home = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
		std::env::var("HOME")
			.map(|home| format!("{home}/.local/share"))
			.unwrap_or_default()
	});
	let icon = format!("{data_home}/icons/hicolor/48x48/apps/oly.png");
	let _ = Command::new("notify-send")
		.args(["--app-name", "oly", "--icon", icon.as_str()])
		.args(["--category", severity.as_str()])
		.args([severity.as_str(), &record.args().to_string()])
		.status();
}

/// Print the 'use oly ... --help for more information' hint to stderr.
pub fn help(cmd: Option<&str>) {
	match cmd {
		Some(cmd) => eprintln!("use oly {cmd} --help for more information"),
		None => eprintln!("use oly --help for more information"),
	}
}
