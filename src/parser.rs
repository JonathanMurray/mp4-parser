use std::time::{Duration, UNIX_EPOCH};

use chrono::{DateTime, Utc};

use crate::logger::Logger;
use crate::reader::Reader;

pub fn parse_mp4(buf: &mut Vec<u8>, mut logger: &mut Logger) {
    let mut reader = Reader::new(&buf);

    while reader.position() < buf.len() as u64 {
        parse_box(&mut reader, &mut logger);
    }

    logger.debug(format!("[{}]", reader.position()));
    logger.debug("Reached end of file");
}

fn parse_box(reader: &mut Reader, logger: &mut Logger) {
    let start_offset = reader.position();
    logger.log_start_of_box(start_offset);

    let mut size = reader.read_u32() as u64;
    let box_type = reader.read_string(4);
    //logger.log(format!("{:?} ({} bytes)", box_type, size));

    if size == 1 {
        // largesize
        size = reader.read_u64();
    } else if size == 0 {
        todo!("Handle box with size=0 (box extends to EOF)")
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

            logger.debug_box(format!(
                "{:?}, {}, {:?}",
                major_brand, minor_version, compatible_brands
            ));
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
                parse_box(reader, logger);
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
                reader.skip_bytes(2).unwrap(); // reserved
                reader.skip_bytes(8).unwrap(); // reserved
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
                parse_box(reader, logger);
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
                let modification_time = as_timestamp(reader.read_u32());
                let track_id = reader.read_u32();
                reader.skip_bytes(4).unwrap(); // reserved
                let duration = reader.read_u32();
                logger.debug_box_attr("Created", &creation_time);
                logger.debug_box_attr("Modified", &modification_time);
                logger.debug_box_attr("Track ID", &track_id);
                logger.debug_box_attr("Duration", &duration);
                reader.skip_bytes(4 * 2).unwrap(); // reserved
                let layer = reader.read_u16();
                let alternate_group = reader.read_u16();
                let volume = reader.read_fixed_point_8_8();
                reader.skip_bytes(2).unwrap(); // reserved
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
                parse_box(reader, logger);
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
            logger.log_box_title("Media Box");
            logger.debug_box("(skipping)"); //TODO
            reader
                .skip_bytes(inner_size as u32)
                .expect("Truncated 'mdia' box");
        }
        "udta" => {
            logger.log_box_title("User Data Box (container)");
            logger.increase_indent();
            let end_offset = reader.position() + inner_size as u64;
            while reader.position() < end_offset {
                parse_box(reader, logger);
            }
            logger.decrease_indent();
        }
        "meta" => {
            logger.log_box_title("The Meta Box");
            logger.debug_box("(skipping)"); //TODO
            reader
                .skip_bytes(inner_size as u32)
                .expect("Truncated 'meta' box");
        }
        _ => {
            todo!("Unhandled box: {:?} (inner size: {})", box_type, inner_size);
        }
    }
}

fn as_timestamp(epoch_secs: u32) -> DateTime<Utc> {
    let system_time = UNIX_EPOCH + Duration::from_secs(epoch_secs as u64);
    system_time.into()
}
