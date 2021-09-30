use crate::reader::Reader;
use chrono::{Duration, NaiveDate, NaiveDateTime};

/// Box (abstract)
#[derive(Debug)]
pub struct BoxHeader {
    pub start_offset: u64,
    pub box_size: u64,
    pub box_type: String,
    pub inner_size: u64,
}

impl BoxHeader {
    pub fn parse(reader: &mut Reader) -> Self {
        let start_offset = reader.position();

        let mut size = reader.read_u32() as u64;
        let box_type = reader.read_string(4);

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

        Self {
            start_offset,
            box_size: size,
            box_type,
            inner_size,
        }
    }
}

/// FullBox (abstract)
#[derive(Debug)]
pub struct FullBoxHeader {
    pub version: u8,
    pub flags: [u8; 3],
}

impl FullBoxHeader {
    pub fn parse(reader: &mut Reader) -> Self {
        let version = reader.read_u8();
        let mut flags = [0; 3];
        reader.read_exact(&mut flags);

        Self { version, flags }
    }
}

/// ftyp
#[derive(Debug)]
pub struct FileTypeBox {
    pub major_brand: String,
    pub minor_version: u32,
    pub compatible_brands: Vec<String>,
}

impl FileTypeBox {
    pub fn parse(reader: &mut Reader, inner_size: u64) -> Self {
        let major_brand = reader.read_string(4);
        let minor_version = reader.read_u32();
        let remaining = inner_size - 8;
        let mut compatible_brands = Vec::new();
        for _ in 0..remaining / 4 {
            compatible_brands.push(reader.read_string(4));
        }

        Self {
            major_brand,
            minor_version,
            compatible_brands,
        }
    }
}

/// mdat
#[derive(Debug)]
pub struct MediaDataBox;

impl MediaDataBox {
    pub fn parse(reader: &mut Reader, inner_size: u64) -> Self {
        reader
            .skip_bytes(inner_size as u32)
            .expect("Truncated 'mdat' box");

        Self
    }
}

/// free
#[derive(Debug)]
pub struct FreeSpaceBox;

impl FreeSpaceBox {
    pub fn parse(_reader: &mut Reader, inner_size: u64) -> Self {
        assert_eq!(inner_size, 0);
        Self
    }
}

/// mvhd
#[derive(Debug)]
pub struct MovieHeaderBox {
    pub creation_time: NaiveDateTime,
    pub modification_time: NaiveDateTime,
    pub timescale: u32,
    pub duration: u32,
    pub rate: f32,
    pub volume: f32,
    pub matrix: Vec<u32>,
    pub next_track_id: u32,
}

impl MovieHeaderBox {
    pub fn parse(reader: &mut Reader, _inner_size: u64) -> Self {
        let full_box = FullBoxHeader::parse(reader);

        if full_box.version == 1 {
            todo!("mvhd version 1")
        } else {
            let creation_time = as_timestamp(reader.read_u32());
            let modification_time = as_timestamp(reader.read_u32());
            let timescale = reader.read_u32();
            let duration = reader.read_u32();
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

            Self {
                creation_time,
                modification_time,
                timescale,
                duration,
                rate,
                volume,
                matrix,
                next_track_id,
            }
        }
    }
}

/// tkhd
#[derive(Debug)]
pub struct TrackHeaderBox {
    pub track_enabled: bool,
    pub track_in_movie: bool,
    pub track_in_preview: bool,
    pub creation_time: NaiveDateTime,
    pub modification_time: NaiveDateTime,
    pub track_id: u32,
    pub duration: u32,
    pub layer: u16,
    pub alternate_group: u16,
    pub volume: f32,
    pub matrix: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

impl TrackHeaderBox {
    pub fn parse(reader: &mut Reader, _inner_size: u64) -> Self {
        let full_box = FullBoxHeader::parse(reader);
        let track_enabled = (full_box.flags[2] & 1) != 0;
        let track_in_movie = (full_box.flags[2] & 2) != 0;
        let track_in_preview = (full_box.flags[2] & 4) != 0;

        if full_box.version == 1 {
            todo!("tkhd version 1")
        } else {
            let creation_time = as_timestamp(reader.read_u32());
            let modification_time = as_timestamp(reader.read_u32());
            let track_id = reader.read_u32();
            let _reserved = reader.read_string(4);
            let duration = reader.read_u32();
            let _reserved = reader.read_string(4 * 2);
            let layer = reader.read_u16();
            let alternate_group = reader.read_u16();
            let volume = reader.read_fixed_point_8_8();
            let _reserved = reader.read_string(2);
            let mut matrix = Vec::new();
            for _ in 0..9 {
                matrix.push(reader.read_u32());
            }
            let width = reader.read_u32();
            let height = reader.read_u32();

            Self {
                track_enabled,
                track_in_movie,
                track_in_preview,
                creation_time,
                modification_time,
                track_id,
                duration,
                layer,
                alternate_group,
                volume,
                matrix,
                width,
                height,
            }
        }
    }
}

/// mdhd
#[derive(Debug)]
pub struct MediaHeaderBox {
    pub creation_time: NaiveDateTime,
    pub modification_time: NaiveDateTime,
    pub timescale: u32,
    pub duration: u32,
    pub language: String,
}

impl MediaHeaderBox {
    pub fn parse(reader: &mut Reader, _inner_size: u64) -> Self {
        let full_box = FullBoxHeader::parse(reader);

        if full_box.version == 1 {
            todo!("mdhd version 1")
        }

        let creation_time = as_timestamp(reader.read_u32());
        let modification_time = as_timestamp(reader.read_u32());
        let timescale = reader.read_u32();
        let duration = reader.read_u32();

        let language = reader.read_bytes(2);
        // Each char is stored as 5bit ascii - 0x60
        let c1 = ((language[0] & 0b0111_1100) >> 2) + 0x60;
        let c2 = ((language[0] & 0b0000_0011) << 3) + ((language[1] & 0b1110_0000) >> 5) + 0x60;
        let c3 = (language[1] & 0b0001_1111) + 0x60;
        let language = String::from_utf8(vec![c1, c2, c3]).unwrap();
        let _pre_defined = reader.read_bytes(2);

        Self {
            creation_time,
            modification_time,
            timescale,
            duration,
            language,
        }
    }
}

/// hdlr
#[derive(Debug)]
pub struct HandlerReferenceBox {
    pub handler_type: String,
    pub name: String,
}

impl HandlerReferenceBox {
    pub fn parse(reader: &mut Reader, inner_size: u64) -> Self {
        FullBoxHeader::parse(reader);

        let _predefined = reader.read_string(4);
        let handler_type = reader.read_string(4);
        let _reserved = reader.read_string(4 * 3);
        let remaining = inner_size - 24;
        let name = reader.read_string(remaining as usize);

        Self { handler_type, name }
    }
}

/// vmhd
#[derive(Debug)]
pub struct VideoMediaHandler {
    pub graphicsmode: u16,
    pub opcolor: Vec<u8>,
}

impl VideoMediaHandler {
    pub fn parse(reader: &mut Reader, _inner_size: u64) -> Self {
        FullBoxHeader::parse(reader);
        let graphicsmode = reader.read_u16();
        let opcolor = reader.read_bytes(2 * 3);
        Self {
            graphicsmode,
            opcolor,
        }
    }
}

/// smhd
#[derive(Debug)]
pub struct SoundMediaHandler {
    pub balance: f32,
}

impl SoundMediaHandler {
    pub fn parse(reader: &mut Reader, _inner_size: u64) -> Self {
        FullBoxHeader::parse(reader);
        let balance = reader.read_fixed_point_8_8();
        let _reserved = reader.read_bytes(2);
        Self { balance }
    }
}

/// elst
#[derive(Debug)]
pub struct EditListBox {
    pub entry_count: u32,
}

#[derive(Debug)]
pub struct EditListEntry {
    pub segment_duration: u32,
    pub media_time: i32,
    pub media_rate_integer: i16,
    pub media_rate_fraction: i16,
}

impl EditListBox {
    pub fn parse_header(reader: &mut Reader) -> Self {
        let full_box = FullBoxHeader::parse(reader);
        if full_box.version == 1 {
            todo!("elst version 1")
        }
        let entry_count = reader.read_u32();
        Self { entry_count }
    }

    pub fn parse_entry(reader: &mut Reader) -> EditListEntry {
        let segment_duration = reader.read_u32();
        let media_time = reader.read_i32();
        let media_rate_integer = reader.read_i16();
        let media_rate_fraction = reader.read_i16();
        EditListEntry {
            segment_duration,
            media_time,
            media_rate_integer,
            media_rate_fraction,
        }
    }
}

/// stts
#[derive(Debug)]
pub struct DecodingTimeToSampleBox {
    pub entry_count: u32,
}

#[derive(Debug)]
pub struct DecodingTimeToSampleEntry {
    pub sample_count: u32,
    pub sample_delta: u32,
}

impl DecodingTimeToSampleBox {
    pub fn parse_header(reader: &mut Reader) -> Self {
        let full_box = FullBoxHeader::parse(reader);
        if full_box.version == 1 {
            todo!("elst version 1")
        }
        let entry_count = reader.read_u32();
        Self { entry_count }
    }

    pub fn parse_entry(reader: &mut Reader) -> DecodingTimeToSampleEntry {
        let sample_count = reader.read_u32();
        let sample_delta = reader.read_u32();
        DecodingTimeToSampleEntry {
            sample_count,
            sample_delta,
        }
    }
}

/// stss
#[derive(Debug)]
pub struct SyncSampleBox {
    pub entry_count: u32,
}

impl SyncSampleBox {
    pub fn parse_header(reader: &mut Reader) -> Self {
        FullBoxHeader::parse(reader);
        let entry_count = reader.read_u32();
        Self { entry_count }
    }

    pub fn skip_entries(&self, reader: &mut Reader) {
        reader.skip_bytes(4 * self.entry_count).unwrap();
    }
}

fn as_timestamp(epoch_secs: u32) -> NaiveDateTime {
    let epoch_1904: NaiveDateTime = NaiveDate::from_ymd(1904, 1, 1).and_hms(0, 0, 0);
    epoch_1904 + Duration::seconds(epoch_secs as i64)
}
