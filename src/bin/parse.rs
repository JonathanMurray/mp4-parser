use std::fs::File;
use std::io::Read;

use clap::{arg_enum, App, Arg};

use mp4_parser::boxes::{BoxHeader, Mp4Box};
use mp4_parser::logger::{
    Logger, LOG_LEVEL_DEBUG, LOG_LEVEL_INFO, LOG_LEVEL_NONE, LOG_LEVEL_TRACE,
};
use mp4_parser::quicktime::EncoderTag;
use mp4_parser::reader::Reader;

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

    parse_mp4(&mut buf, &mut logger);
}

#[derive(Copy, Clone)]
enum HandleUnknown {
    Skip,
    Panic,
}

fn parse_mp4(buf: &mut Vec<u8>, mut logger: &mut Logger) {
    let mut reader = Reader::new(buf);

    _parse(
        &mut reader,
        &mut logger,
        HandleUnknown::Panic,
        buf.len() as u64,
    );

    logger.debug(format!("[{}]", reader.position()));
    logger.debug("Reached end of file");
}

fn _parse(
    reader: &mut Reader,
    logger: &mut Logger,
    handle_unknown: HandleUnknown,
    end_offset: u64,
) {
    while reader.position() < end_offset {
        let box_start_offset = reader.position();

        let header = BoxHeader::parse(reader);

        logger.log_start_of_box(header.start_offset);
        logger.debug_box(format!("{:?} ({} bytes)", header.box_type, header.box_size));

        let box_ = Mp4Box::parse_contents(reader, &header.box_type, header.inner_size);
        // println!("DEBUG: Parsed box: {:?}", box_);

        let box_ = match box_ {
            Some(b) => b,
            None => match handle_unknown {
                HandleUnknown::Skip => {
                    logger.log_box_title(format!(
                        "Skipping unknown: '{}' ({} bytes)",
                        header.box_type, header.box_size
                    ));
                    reader
                        .skip_bytes(header.inner_size as u32)
                        .unwrap_or_else(|e| panic!("Truncated '{}' box: {}", header.box_type, e));
                    continue;
                }
                HandleUnknown::Panic => {
                    todo!(
                        "Unhandled box: {:?} (inner size: {})",
                        header.box_type,
                        header.inner_size
                    );
                }
            },
        };

        logger.log_box_title(box_.name());
        box_.print_attributes(|k, v| logger.debug_box_attr(k, v));

        let box_end_offset = box_start_offset + header.box_size;
        match box_ {
            Mp4Box::Container(_) => {
                logger.increase_indent();
                //println!("DEBUG: It's a container. Will jump into it");
                _parse(reader, logger, HandleUnknown::Skip, box_end_offset);
                logger.decrease_indent();
            }
            Mp4Box::QuickTimeMetadataItemList(metadata_item_list) => {
                logger.increase_indent();
                while reader.position() < box_end_offset {
                    let tag: EncoderTag = metadata_item_list.parse_entry(reader);
                    logger.debug_box(format!("{:?}", tag));
                }
                logger.decrease_indent();
            }
            Mp4Box::Stsd(sample_description_box) => {
                logger.increase_indent();
                for _ in 0..sample_description_box.entry_count {
                    let entry = sample_description_box.parse_entry(reader);
                    logger.debug_box(entry.name());
                    entry.print_attributes(|k, v| logger.debug_box_attr(k, v));
                }
                logger.decrease_indent();
            }
            _ => {}
        }

        let remaining = (box_end_offset - reader.position()) as u32;
        if remaining > 0 {
            // println!("DEBUG: Skipping {} bytes of {}", remaining, header.box_type);
            reader.skip_bytes(remaining).unwrap();
        }
    }
}
