use std::fs::File;
use std::io::Read;

use clap::{arg_enum, App, Arg};

use logger::{Logger, LOG_LEVEL_DEBUG, LOG_LEVEL_INFO, LOG_LEVEL_NONE, LOG_LEVEL_TRACE};

mod boxes;
mod logger;
mod parser;
mod quicktime;
mod reader;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum LogLevelArg {
        None,
        Info,
        Debug,
        Trace,
    }
}

fn main() {
    let matches = App::new("mp4-parser")
        .about("Parse an MP4 file")
        .arg(
            Arg::with_name("FILE")
                .help("The mp4 file that should be parsed")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("loglevel")
                .short("l")
                .long("loglevel")
                .value_name("VERBOSITY")
                .possible_values(&LogLevelArg::variants())
                .case_insensitive(true)
                .help("Chooses the verbosity of the tool's output"),
        )
        .get_matches();

    let log_level = matches.value_of("loglevel").map(|v| v.to_lowercase());
    let path = matches.value_of("FILE").unwrap();
    let verbosity = match log_level.as_ref().map(|v| &v[..]) {
        Some("none") => LOG_LEVEL_NONE,
        Some("info") => LOG_LEVEL_INFO,
        Some("debug") => LOG_LEVEL_DEBUG,
        Some("trace") => LOG_LEVEL_TRACE,
        None => LOG_LEVEL_DEBUG,
        _ => panic!("Unhandled log level: {:?}", log_level),
    };
    let mut f = File::open(&path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let mut logger = Logger::new(verbosity);
    logger.debug(format!("Read {} bytes", buf.len()));

    parser::parse_mp4(&mut buf, &mut logger);
}
