use crate::boxes::{
    BoxHeader, DecodingTimeToSampleBox, EditListBox, FileTypeBox, FreeSpaceBox, FullBoxHeader,
    HandlerReferenceBox, MediaDataBox, MediaHeaderBox, MovieFragmentHeaderBox, MovieHeaderBox,
    SoundMediaHandler, SyncSampleBox, TrackExtendsBox, TrackHeaderBox, VideoMediaHandler,
};
use crate::logger::Logger;
use crate::reader::Reader;

#[derive(Copy, Clone)]
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
    let header = BoxHeader::parse(reader);
    logger.log_start_of_box(header.start_offset);
    logger.debug_box(format!("{:?} ({} bytes)", header.box_type, header.box_size));
    let size = header.box_size;
    let box_type = header.box_type;
    let inner_size = header.inner_size;

    match box_type.as_ref() {
        "ftyp" => {
            logger.log_box_title("File Type Box");
            let b = FileTypeBox::parse(reader, inner_size);
            logger.debug_box_attr("Major brand", &b.major_brand);
            logger.debug_box_attr("Minor version", &b.minor_version);
            logger.debug_box(format!("Compatible: {:?}", b.compatible_brands));

            if b.major_brand == "qt  " {
                println!("WARN: Apple QuickTime is not supported.");
            }
        }
        "free" => {
            logger.log_box_title("Free Space Box");
            FreeSpaceBox::parse(reader, inner_size);
        }
        "mdat" => {
            logger.log_box_title("Media Data Box");
            MediaDataBox::parse(reader, inner_size);
            logger.debug_box("(skipping)"); //TODO
        }
        "moov" => {
            logger.log_box_title("Movie Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "mvhd" => {
            logger.log_box_title("Movie Header Box");
            let b = MovieHeaderBox::parse(reader, inner_size);
            logger.debug_box_attr("Created", &b.creation_time);
            logger.debug_box_attr("Modified", &b.modification_time);
            logger.debug_box_attr("Timescale", &b.timescale);
            logger.debug_box_attr("Duration", &b.duration);
            logger.debug_box_attr("Rate", &b.rate);
            logger.debug_box_attr("Volume", &b.volume);
            logger.debug_box_attr("Matrix", &format!("{:?}", b.matrix));
            logger.debug_box_attr("Next track ID", &b.next_track_id);
        }
        "trak" => {
            logger.log_box_title("Track Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "tkhd" => {
            logger.log_box_title("Track Header Box");
            let b = TrackHeaderBox::parse(reader, inner_size);
            logger.debug_box(format!(
                "Enabled: {}, in movie: {}, in preview: {}",
                b.track_enabled, b.track_in_movie, b.track_in_preview
            ));
            logger.debug_box_attr("Created", &b.creation_time);
            logger.debug_box_attr("Modified", &b.modification_time);
            logger.debug_box_attr("Track ID", &b.track_id);
            logger.debug_box_attr("Duration", &b.duration);
            logger.debug_box_attr("Layer", &b.layer);
            logger.debug_box_attr("Alternate group", &b.alternate_group);
            logger.debug_box_attr("Volume", &b.volume);
            logger.debug_box(format!("Matrix: {:?}", b.matrix));
            logger.debug_box_attr("Dimension", &format!("{} x {}", b.width, b.height));
        }
        "edts" => {
            logger.log_box_title("Edit Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "elst" => {
            logger.log_box_title("Edit List Box");

            let h = EditListBox::parse_header(reader);
            logger.debug_box_attr("# entries", &h.entry_count);
            for _ in 0..h.entry_count {
                let e = EditListBox::parse_entry(reader);
                logger.debug_box_attr("Segment duration", &e.segment_duration);
                logger.debug_box_attr("Media time", &e.media_time);
                logger.debug_box_attr("Media rate", &e.media_rate_integer);
            }
        }
        "mdia" => {
            logger.log_box_title("Media Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "mdhd" => {
            logger.log_box_title("Media Header Box");
            let b = MediaHeaderBox::parse(reader, inner_size);
            logger.debug_box_attr("Created", &b.creation_time);
            logger.debug_box_attr("Modified", &b.modification_time);
            logger.debug_box_attr("Timescale", &b.timescale);
            logger.debug_box_attr("Duration", &b.duration);
            logger.debug_box_attr("Language", &b.language);
        }
        "hdlr" => {
            logger.log_box_title("Handler Reference Box");
            let b = HandlerReferenceBox::parse(reader, inner_size);
            logger.debug_box_attr("Handler type", &b.handler_type);
            logger.debug_box_attr("Name", &b.name);
        }
        "minf" => {
            logger.log_box_title("Media Information Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "vmhd" => {
            logger.log_box_title("Video media header");
            let b = VideoMediaHandler::parse(reader, inner_size);
            logger.debug_box_attr("Graphics mode", &b.graphicsmode);
            logger.debug_box(format!("Opcolor: {:?}", &b.opcolor));
        }
        "smhd" => {
            logger.log_box_title("Sound media header");
            let b = SoundMediaHandler::parse(reader, inner_size);
            logger.debug_box_attr("Balance", &b.balance);
        }
        "dinf" => {
            logger.log_box_title("Data Information Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "dref" => {
            logger.log_box_title("Data Reference Box");

            FullBoxHeader::parse(reader);

            let entry_count = reader.read_u32();
            logger.increase_indent();
            for _ in 0..entry_count {
                parse_box(reader, logger, HandleUnknown::Panic);
            }
            logger.decrease_indent();
        }
        "url " => {
            logger.log_box_title("Data Entry URL Box");
            let full_box = FullBoxHeader::parse(reader);
            if full_box.flags == [0, 0, 1] {
                logger.debug_box("Self-contained")
            } else {
                todo!("Handle external media URL")
            }
        }
        "stbl" => {
            logger.log_box_title("Sample Table Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "stsd" => {
            logger.log_box_title("Sample Description Box");

            FullBoxHeader::parse(reader);

            let entry_count = reader.read_u32();
            logger.debug_box_attr("# entries", &entry_count);
            logger.increase_indent();
            for _ in 0..entry_count {
                // Technically not boxes, but can be parsed as if they are
                parse_box(reader, logger, HandleUnknown::Skip);
            }
            logger.decrease_indent();
        }
        "mp4a" => {
            logger.log_box_title(format!("AudioSampleEntry({})", box_type));
            //Technically not a box, but can be parsed as if it was
            let _reserved = reader.read_string(6);
            let data_reference_index = reader.read_u16();
            logger.debug_box_attr("Data reference index", &data_reference_index);

            let mut remaining = inner_size - 8;

            // https://www.fatalerrors.org/a/analysis-of-mp4-file-format.html

            let _reserved = reader.read_bytes(4 * 2);
            let channel_count = reader.read_u16();
            let sample_size = reader.read_u16();
            let _predefined = reader.read_bytes(2);
            let _reserved = reader.read_bytes(2);
            let sample_rate = reader.read_fixed_point_16_16();

            remaining -= 20;

            logger.debug_box_attr("Channel count", &channel_count);
            logger.debug_box_attr("Sample size", &sample_size);
            logger.debug_box_attr("Sample rate", &sample_rate);

            parse_container_sub_boxes(reader, remaining, logger, HandleUnknown::Skip);
        }
        "avc1" => {
            logger.log_box_title(format!("VisualSampleEntry({})", box_type));
            //Technically not a box, but can be parsed as if it was
            let _reserved = reader.read_string(6);
            let data_reference_index = reader.read_u16();
            logger.debug_box_attr("Data reference index", &data_reference_index);
            let mut remaining = inner_size - 8;

            // https://www.fatalerrors.org/a/analysis-of-mp4-file-format.html

            reader.skip_bytes(2).unwrap(); // predefined
            reader.skip_bytes(2).unwrap(); // reserved
            reader.skip_bytes(4 * 3).unwrap(); // predefined
            let width = reader.read_u16();
            let height = reader.read_u16();
            let hor_resolution = reader.read_fixed_point_16_16();
            let ver_resolution = reader.read_fixed_point_16_16();
            reader.skip_bytes(4).unwrap(); // reserved
            let frame_count = reader.read_u16();
            let compressor_name = reader.read_string(32);
            let depth = reader.read_u16();
            reader.skip_bytes(2).unwrap(); // predefined

            remaining -= 70;

            logger.debug_box_attr("Width", &width);
            logger.debug_box_attr("Height", &height);
            logger.debug_box_attr("Horizontal resolution", &hor_resolution);
            logger.debug_box_attr("Vertical resolution", &ver_resolution);
            logger.debug_box_attr("Frame count", &frame_count);
            logger.debug_box_attr("Compressor name", &compressor_name);
            logger.debug_box_attr("Depth", &depth);

            parse_container_sub_boxes(reader, remaining, logger, HandleUnknown::Skip);
        }
        "stts" => {
            logger.log_box_title("Decoding Time to Sample Box");
            let h = DecodingTimeToSampleBox::parse_header(reader);
            logger.debug_box_attr("# entries", &h.entry_count);
            for i in 0..h.entry_count {
                let e = DecodingTimeToSampleBox::parse_entry(reader);
                logger.trace_box(format!(
                    "({}) Sample count: {}, sample delta: {}",
                    i, e.sample_count, e.sample_delta,
                ));
            }
        }
        "stss" => {
            logger.log_box_title("Sync Sample Box");
            let h = SyncSampleBox::parse_header(reader);
            logger.debug_box_attr("# entries", &h.entry_count);
            h.skip_entries(reader);
            logger.debug_box(format!("(skipping {} entries)", h.entry_count));
        }
        "ctts" => {
            logger.log_box_title("Composition Time to Sample Box");
            let full_box = FullBoxHeader::parse(reader);
            let entry_count = reader.read_u32();
            logger.debug_box_attr("# entries", &entry_count);
            if full_box.version == 0 {
                for i in 0..entry_count {
                    let sample_count = reader.read_u32();
                    let sample_offset = reader.read_u32();
                    logger.trace_box(format!(
                        "({}) Sample count: {}, sample offset: {}",
                        i, sample_count, sample_offset,
                    ));
                }
            } else {
                todo!("ctts version 1")
            }
        }
        "stsc" => {
            logger.log_box_title("Sample to Chunk Box");
            FullBoxHeader::parse(reader);
            let entry_count = reader.read_u32();
            logger.debug_box_attr("# entries", &entry_count);
            for i in 0..entry_count {
                let first_chunk = reader.read_u32();
                let samples_per_chunk = reader.read_u32();
                let sample_description_index = reader.read_u32();
                logger.trace_box(format!(
                    "({}) First chunk: {}, smpls/chunk: {}, smpl dscr idx: {}",
                    i, first_chunk, samples_per_chunk, sample_description_index,
                ));
            }
        }
        "stsz" => {
            logger.log_box_title("Sample Size Box");
            FullBoxHeader::parse(reader);
            let sample_size = reader.read_u32();
            if sample_size != 0 {
                logger.debug_box_attr("Sample size", &sample_size);
            }
            let sample_count = reader.read_u32();
            logger.debug_box_attr("# samples", &sample_count);
            if sample_size == 0 {
                for i in 0..sample_count {
                    let sample_size = reader.read_u32();
                    logger.trace_box(format!("({}) Sample size: {}", i, sample_size));
                }
            }
        }
        "stco" => {
            logger.log_box_title("Chunk Offset Box");
            FullBoxHeader::parse(reader);
            let entry_count = reader.read_u32();
            logger.debug_box_attr("# entries", &entry_count);
            for i in 0..entry_count {
                let chunk_offset = reader.read_u32();
                logger.trace_box(format!("({}) Chunk offset: {}", i, chunk_offset))
            }
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

        "mvex" => {
            logger.log_box_title("Movie Extends Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "trex" => {
            logger.log_box_title("Track Extends Box");
            let b = TrackExtendsBox::parse(reader, inner_size);

            logger.debug_box_attr("Track ID", &b.track_id);
            logger.debug_box_attr(
                "Default smpl. descr. index",
                &b.default_sample_description_index,
            );
            logger.debug_box_attr("Default sample duration", &b.default_sample_duration);
            logger.debug_box_attr("Default sample size", &b.default_sample_size);
            logger.debug_box_attr("Default sample flags", &b.default_sample_flags);
        }
        "moof" => {
            logger.log_box_title("Movie Fragment Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Panic);
        }
        "mfhd" => {
            logger.log_box_title("Movie Fragment Header Box");
            let b = MovieFragmentHeaderBox::parse(reader, inner_size);
            logger.debug_box_attr("Sequence number", &b.sequence_number);
        }
        "traf" => {
            logger.log_box_title("Track Fragment Box (container)");
            //TODO don't skip over sub boxes
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Skip);
        }
        "mfra" => {
            logger.log_box_title("Movie Fragment Random Access Box (container)");
            //TODO don't skip over sub boxes
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Skip);
        }
        "udta" => {
            logger.log_box_title("User Data Box (container)");
            parse_container_sub_boxes(reader, inner_size as u64, logger, HandleUnknown::Skip);
        }
        "meta" => {
            logger.log_box_title("The Meta Box (container)");
            FullBoxHeader::parse(reader);
            parse_container_sub_boxes(reader, inner_size - 4, logger, HandleUnknown::Panic);
        }
        // QuickTime
        "ilst" => {
            logger.log_box_title("QuickTime Metadata Item List");
            parse_container_sub_boxes(reader, inner_size - 4, logger, HandleUnknown::Panic);
        }
        // QuickTime
        "©too" => {
            logger.log_box_title("QuickTime Metadata Item List Encoder Tag");
            let remaining = inner_size;
            let encoder_tag_content = reader.read_string(remaining as usize);
            logger.debug_box_attr("Encoder Tag", &encoder_tag_content);
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

fn parse_container_sub_boxes(
    reader: &mut Reader,
    n_bytes: u64,
    logger: &mut Logger,
    handle_unknown: HandleUnknown,
) {
    logger.increase_indent();
    let end_offset = reader.position() + n_bytes;
    while reader.position() < end_offset {
        parse_box(reader, logger, handle_unknown);
    }
    logger.decrease_indent();
}
