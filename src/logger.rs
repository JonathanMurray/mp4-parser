use std::fmt::Display;

pub type LogLevel = u32;
pub const LOG_LEVEL_NONE: LogLevel = 0;
pub const LOG_LEVEL_INFO: LogLevel = 1;
pub const LOG_LEVEL_DEBUG: LogLevel = 2;

pub struct Logger {
    verbosity: LogLevel,
    indent: usize,
}

impl Logger {
    pub fn new(verbosity: LogLevel) -> Self {
        Self {
            verbosity,
            indent: 4,
        }
    }

    pub fn debug(&self, text: impl Display) {
        if self.verbosity >= LOG_LEVEL_DEBUG {
            println!("{}", text);
        }
    }

    pub fn log_start_of_box(&self, file_offset: u64) {
        if self.verbosity >= LOG_LEVEL_DEBUG {
            println!("[{}]", file_offset);
            println!(
                "{:indent$}+----------------------------",
                "",
                indent = self.indent
            );
        }
    }

    pub fn log_box_title(&self, text: impl AsRef<str>) {
        if self.verbosity >= LOG_LEVEL_INFO {
            println!("{:indent$}| {}", "", text.as_ref(), indent = self.indent);
        }
    }

    pub fn debug_box(&self, text: impl AsRef<str>) {
        if self.verbosity >= LOG_LEVEL_DEBUG {
            println!("{:indent$}| {}", "", text.as_ref(), indent = self.indent);
        }
    }

    pub fn debug_box_attr(&self, label: &str, value: &impl Display) {
        if self.verbosity >= LOG_LEVEL_DEBUG {
            println!("{:indent$}| {}: {}", "", label, value, indent = self.indent);
        }
    }

    pub fn increase_indent(&mut self) {
        self.indent += 2;
    }

    pub fn decrease_indent(&mut self) {
        self.indent -= 2;
    }
}
