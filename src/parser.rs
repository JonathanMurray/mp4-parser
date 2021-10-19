use crate::boxes::*;
use crate::logger::Logger;
use crate::quicktime;
use crate::quicktime::EncoderTag;
use crate::reader::Reader;

#[derive(Copy, Clone)]
enum HandleUnknown {
    Skip,
    Panic,
}

pub fn parse_mp4(buf: &mut Vec<u8>, mut logger: &mut Logger) {
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
