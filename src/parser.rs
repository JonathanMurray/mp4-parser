use chrono::{Duration, NaiveDate, NaiveDateTime};

use crate::logger::Logger;
use crate::reader::Reader;

enum HandleUnknown {
    Skip,
    Panic,
}

pub fn parse_mp4(buf: &mut Vec<u8>, mut logger: &mut Logger) {
    let mut reader = Reader::new(buf);

    while reader.position() < buf.len() as u64 {
        parse_box(&mut reader, &mut logger, HandleUnknown::Panic);
    }

    logger.debug(format!("[{}]", reader.position()));
    logger.debug("Reached end of file");
}

fn parse_box(reader: &mut Reader, logger: &mut Logger, handle_unknown: HandleUnknown) {
    let start_offset = reader.position();
    logger.log_start_of_box(start_offset);

    let mut size = reader.read_u32() as u64;
    let box_type = reader.read_string(4);
    logger.debug_box(format!("{:?} ({} bytes)", box_type, size));

    if size == 1 {
        // largesize
        size = reader.read_u64();
    } else if size == 0 {
        println!("DEBUG: {:?}", reader.read_string_inexact(256));
        todo!("Handle box with size=0 (box '{}' extends to EOF)", box_type)
    }

    assert!(
        size >= 8,
        "Box {} (at {}) has invalid size: {}",
        box_type,
        start_offset,
        size
    );

    let inner_size = size - 8;

    match box_type.as_ref() {
        "ftyp" => {
            logger.log_box_title("File Type Box");
            let major_brand = reader.read_string(4);
            let minor_version = reader.read_u32();
            let remaining = inner_size - 8;
            let mut compatible_brands = Vec::new();
            for _ in 0..remaining / 4 {
                compatible_brands.push(reader.read_string(4));
            }
            logger.debug_box_attr("Major brand", &major_brand);
            logger.debug_box_attr("Minor version", &minor_version);
            logger.debug_box(format!("Compatible: {:?}", compatible_brands));

            if major_brand == "qt  " {
                println!("WARN: Apple QuickTime is not supported.");
            }
        }
        "free" => {
            logger.log_box_title("Free Space Box");
            assert_eq!(inner_size, 0);
        }
        "mdat" => {
            logger.log_box_title("Media Data Box");
            logger.debug_box("(skipping)"); //TODO
            reader
                .skip_bytes(inner_size as u32)
                .expect("Truncated 'mdat' box");
        }
        "moov" => {
            logger.log_box_title("Movie Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "mvhd" => {
            logger.log_box_title("Movie Header Box");
            let version = reader.read_u8();
            let _flags = reader.read_bytes(3);

            if version == 1 {
                todo!("mvhd version 1")
            } else {
                let creation_time = as_timestamp(reader.read_u32());
                let modification_time = as_timestamp(reader.read_u32());
                let timescale = reader.read_u32();
                let duration = reader.read_u32();
                logger.debug_box_attr("Created", &creation_time);
                logger.debug_box_attr("Modified", &modification_time);
                logger.debug_box_attr("Timescale", &timescale);
                logger.debug_box_attr("Duration", &duration);
                let rate = reader.read_fixed_point_16_16();
                let volume = reader.read_fixed_point_8_8();
                let _reserved = reader.read_string(2);
                let _reserved = reader.read_string(8);
                let mut matrix = Vec::new();
                for _ in 0..9 {
                    matrix.push(reader.read_u32());
                }
                let mut _pre_defined = Vec::new();
                for _ in 0..6 {
                    _pre_defined.push(reader.read_u32());
                }
                let next_track_id = reader.read_u32();

                logger.debug_box_attr("Rate", &rate);
                logger.debug_box_attr("Volume", &volume);
                logger.debug_box_attr("Matrix", &format!("{:?}", matrix));
                logger.debug_box_attr("Next track ID", &next_track_id);
            }
        }
        "trak" => {
            logger.log_box_title("Track Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "tkhd" => {
            logger.log_box_title("Track Header Box");
            let version = reader.read_u8();
            let flags = reader.read_bytes(3);
            let track_enabled = (flags[2] & 1) != 0;
            let track_in_movie = (flags[2] & 2) != 0;
            let track_in_preview = (flags[2] & 4) != 0;
            logger.debug_box(format!(
                "Enabled: {}, in movie: {}, in preview: {}",
                track_enabled, track_in_movie, track_in_preview
            ));

            if version == 1 {
                todo!("tkhd version 1")
            } else {
                let creation_time = as_timestamp(reader.read_u32());
                logger.debug_box_attr("Created", &creation_time);
                let modification_time = as_timestamp(reader.read_u32());
                logger.debug_box_attr("Modified", &modification_time);
                let track_id = reader.read_u32();
                logger.debug_box_attr("Track ID", &track_id);
                let reserved = reader.read_string(4);
                logger.debug_box_attr("(reserved)", &reserved);
                let duration = reader.read_u32();
                logger.debug_box_attr("Duration", &duration);

                let reserved = reader.read_string(4 * 2);
                logger.debug_box_attr("(reserved)", &reserved);
                let layer = reader.read_u16();
                let alternate_group = reader.read_u16();
                let volume = reader.read_fixed_point_8_8();
                let reserved = reader.read_string(2);
                logger.debug_box_attr("(reserved)", &reserved);
                let mut matrix = Vec::new();
                for _ in 0..9 {
                    matrix.push(reader.read_u32());
                }
                let width = reader.read_u32();
                let height = reader.read_u32();
                logger.debug_box_attr("Layer", &layer);
                logger.debug_box_attr("Alternate group", &alternate_group);
                logger.debug_box_attr("Volume", &volume);
                logger.debug_box(format!("Matrix: {:?}", matrix));
                logger.debug_box_attr("Dimension", &format!("{} x {}", width, height));
            }
        }
        "edts" => {
            logger.log_box_title("Edit Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "elst" => {
            logger.log_box_title("Edit List Box");

            let version = reader.read_u8();
            let _flags = reader.read_bytes(3);

            let entry_count = reader.read_u32();
            for _ in 0..entry_count {
                if version == 1 {
                    todo!("elst version 1")
                } else {
                    let segment_duration = reader.read_u32();
                    let media_time = reader.read_i32();
                    let media_rate_integer = reader.read_i16();
                    let _media_rate_fraction = reader.read_i16();
                    logger.debug_box_attr("Segment duration", &segment_duration);
                    logger.debug_box_attr("Media time", &media_time);
                    logger.debug_box_attr("Media rate", &media_rate_integer);
                }
            }
        }
        "mdia" => {
            logger.log_box_title("Media Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "mdhd" => {
            logger.log_box_title("Media Header Box");

            let version = reader.read_u8();
            let _flags = reader.read_bytes(3);

            if version == 1 {
                todo!("mdhd version 1")
            } else {
                let creation_time = as_timestamp(reader.read_u32());
                let modification_time = as_timestamp(reader.read_u32());
                let timescale = reader.read_u32();
                let duration = reader.read_u32();
                logger.debug_box_attr("Created", &creation_time);
                logger.debug_box_attr("Modified", &modification_time);
                logger.debug_box_attr("Timescale", &timescale);
                logger.debug_box_attr("Duration", &duration);
            }

            let language = reader.read_bytes(2);
            // Each char is stored as 5bit ascii - 0x60
            let c1 = ((language[0] & 0b0111_1100) >> 2) + 0x60;
            let c2 = ((language[0] & 0b0000_0011) << 3) + ((language[1] & 0b1110_0000) >> 5) + 0x60;
            let c3 = (language[1] & 0b0001_1111) + 0x60;
            let language = String::from_utf8(vec![c1, c2, c3]).unwrap();
            logger.debug_box_attr("Language", &language);

            let _pre_defined = reader.read_bytes(2);
        }
        "hdlr" => {
            logger.log_box_title("Handler Reference Box");

            let _version = reader.read_u8();
            let _flags = reader.read_bytes(3);

            let predefined = reader.read_string(4);
            logger.debug_box_attr("(predefined)", &predefined);
            let handler_type = reader.read_string(4);
            logger.debug_box_attr("Handler type", &handler_type);
            let _reserved = reader.read_string(4 * 3);
            let remaining = inner_size - 24;
            let name = reader.read_string(remaining as usize);
            logger.debug_box_attr("Name", &name);
        }
        "minf" => {
            logger.log_box_title("Media Information Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "vmhd" => {
            logger.log_box_title("Video media header");

            let _version = reader.read_u8();
            let _flags = reader.read_bytes(3);

            let graphicsmode = reader.read_u16();
            let opcolor = reader.read_bytes(2 * 3);
            logger.debug_box_attr("Graphics mode", &graphicsmode);
            logger.debug_box(format!("Opcolor: {:?}", &opcolor));
        }
        "smhd" => {
            logger.log_box_title("Sound media header");

            let _version = reader.read_u8();
            let _flags = reader.read_bytes(3);

            let balance = reader.read_fixed_point_8_8();
            logger.debug_box_attr("Balance", &balance);
            let _reserved = reader.read_bytes(2);
        }
        "dinf" => {
            logger.log_box_title("Data Information Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "dref" => {
            logger.log_box_title("Data Reference Box");

            let _version = reader.read_u8();
            let _flags = reader.read_bytes(3);

            let entry_count = reader.read_u32();
            logger.increase_indent();
            for _ in 0..entry_count {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "url " => {
            logger.log_box_title("Data Entry URL Box");
            let _version = reader.read_u8();
            let flags = reader.read_bytes(3);
            if flags == [0, 0, 1] {
                logger.debug_box("Self-contained")
            } else {
                todo!("Handle external media URL")
            }
        }
        "stbl" => {
            logger.log_box_title("Sample Table Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "stsd" => {
            logger.log_box_title("Sample Description Box");

            let _version = reader.read_u8();
            let _flags = reader.read_bytes(3);

            let entry_count = reader.read_u32();
            logger.debug_box_attr("# entries", &entry_count);
            logger.increase_indent();
            for _ in 0..entry_count {
                // Technically not boxes, but can be parsed as if they are
                parse_box(reader, logger, HandleUnknown::Skip);
            }
            logger.decrease_indent();
        }
        "avc1" | "mp4a" => {
            logger.log_box_title(format!("SampleEntry({})", box_type));
            //Technically not a box, but can be parsed as if it was
            let _reserved = reader.read_string(6);
            let data_reference_index = reader.read_u16();
            logger.debug_box_attr("Data reference index", &data_reference_index);
            let remaining = inner_size - 8;
            reader.skip_bytes(remaining as u32).unwrap();
            logger.debug_box(format!("(skipping {} bytes of data)", remaining));
        }
        "stts" => {
            logger.log_box_title("Decoding Time to Sample Box");
            let _version = reader.read_u8();
            let _flags = reader.read_bytes(3);
            let entry_count = reader.read_u32();
            logger.debug_box_attr("# entries", &entry_count);
            reader.skip_bytes((4 + 4) * entry_count).unwrap();
            logger.debug_box(format!(
                "(skipping {} bytes of entries)",
                (4 + 4) * entry_count
            ));
        }
        "stss" => {
            logger.log_box_title("Sync Sample Box");
            let _version = reader.read_u8();
            let _flags = reader.read_bytes(3);
            let entry_count = reader.read_u32();
            logger.debug_box_attr("# entries", &entry_count);
            reader.skip_bytes(4 * entry_count).unwrap();
            logger.debug_box(format!("(skipping {} bytes of entries)", 4 * entry_count));
        }
        "ctts" => {
            logger.log_box_title("Composition Time to Sample Box");
            logger.debug_box("(skipping)"); //TODO
            reader.skip_bytes(inner_size as u32).unwrap();
        }
        "stsc" => {
            logger.log_box_title("Sample to Chunk Box");
            logger.debug_box("(skipping)"); //TODO
            reader.skip_bytes(inner_size as u32).unwrap();
        }
        "stsz" => {
            logger.log_box_title("Sample Table Box");
            logger.debug_box("(skipping)"); //TODO
            reader.skip_bytes(inner_size as u32).unwrap();
        }
        "stco" => {
            logger.log_box_title("Chunk Offset Box");
            logger.debug_box("(skipping)"); //TODO
            reader.skip_bytes(inner_size as u32).unwrap();
        }
        "sgpd" => {
            logger.log_box_title("Sample Group Description Box");
            logger.debug_box("(skipping)"); //TODO
            reader.skip_bytes(inner_size as u32).unwrap();
        }
        "sbgp" => {
            logger.log_box_title("Sample to Group Box");
            logger.debug_box("(skipping)"); //TODO
            reader.skip_bytes(inner_size as u32).unwrap();
        }
        "sdtp" => {
            logger.log_box_title("Sample Dependency Type Box");
            logger.debug_box("(skipping)"); //TODO
            reader.skip_bytes(inner_size as u32).unwrap();
        }

        "udta" => {
            logger.log_box_title("User Data Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Skip);
            }
            logger.decrease_indent();
        }
        "meta" => {
            logger.log_box_title("The Meta Box (container)");

            let _version = reader.read_u8();
            let _flags = reader.read_bytes(3);
            let remaining = inner_size - 4;
            logger.increase_indent();
            let end_offset = reader.position() + remaining;
            while reader.position() < end_offset {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "ilst" => {
            logger.log_box_title("Unknown: 'ilst'");
            logger.debug_box("(skipping)"); //TODO
            reader
                .skip_bytes(inner_size as u32)
                .expect("Truncated 'ilst' box");
        }

        _ => match handle_unknown {
            HandleUnknown::Skip => {
                logger.log_box_title(format!("Skipping unknown: '{}' ({} bytes)", box_type, size));
                reader
                    .skip_bytes(inner_size as u32)
                    .unwrap_or_else(|e| panic!("Truncated '{}' box: {}", box_type, e));
            }
            HandleUnknown::Panic => {
                todo!("Unhandled box: {:?} (inner size: {})", box_type, inner_size);
            }
        },
    }
}

fn as_timestamp(epoch_secs: u32) -> NaiveDateTime {
    let epoch_1904: NaiveDateTime = NaiveDate::from_ymd(1904, 1, 1).and_hms(0, 0, 0);
    epoch_1904 + Duration::seconds(epoch_secs as i64)
}
