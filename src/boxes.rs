use chrono::{Duration, NaiveDate, NaiveDateTime};

use crate::quicktime::MetadataItemList;
use crate::reader::Reader;

#[derive(Debug)]
pub enum Mp4Box {
    QuickTimeMetadataItemList(MetadataItemList),
    Ftyp(FileTypeBox),
    Free,
    Mdat,
    Container(&'static str),
    Mvhd(MovieHeaderBox),
    Tkhd(TrackHeaderBox),
    Elst(EditListBox),
    Mdhd(MediaHeaderBox),
    Hdlr(HandlerReferenceBox),
    Vmhd(VideoMediaHandler),
    Smhd(SoundMediaHandler),
    Dref(DataReferenceBox),
    Stsd(SampleDescriptionBox),
    Stts(DecodingTimeToSampleBox),
    Stss(SyncSampleBox),
    Ctts(CompositionTimeToSampleBox),
    Stsc(SampleToChunkBox),
    Stsz(SampleSizeBox),
    Stco(ChunkOffsetBox),
    Sgpd(SampleGroupDescriptionBox),
    Sbgp(SampleToGroupBox),
    Sdtp(SampleDependencyTypeBox),
    Trex(TrackExtendsBox),
    Mfhd(MovieFragmentHeaderBox),
}

impl Mp4Box {
    pub fn parse_contents(reader: &mut Reader, box_type: &str, inner_size: u64) -> Option<Self> {
        match box_type {
            "ftyp" => {
                let b = FileTypeBox::parse(reader, inner_size);
                if b.major_brand == "qt  " {
                    println!("WARN: Apple QuickTime is not supported.");
                }
                Some(Mp4Box::Ftyp(b))
            }
            "free" => {
                FreeSpaceBox::parse(reader, inner_size);
                Some(Mp4Box::Free)
            }
            "mdat" => {
                MediaDataBox::parse(reader, inner_size);
                Some(Mp4Box::Mdat)
            }
            "moov" => Some(Mp4Box::Container("Movie Box (container)")),
            "mvhd" => {
                let b = MovieHeaderBox::parse(reader, inner_size);
                Some(Mp4Box::Mvhd(b))
            }
            "trak" => Some(Mp4Box::Container("Track Box (container)")),
            "tkhd" => {
                let b = TrackHeaderBox::parse(reader, inner_size);
                Some(Mp4Box::Tkhd(b))
            }
            "edts" => Some(Mp4Box::Container("Edit Box (container)")),
            "elst" => {
                let b = EditListBox::parse_header(reader);
                Some(Mp4Box::Elst(b))
            }
            "mdia" => Some(Mp4Box::Container("Media Box (container)")),
            "mdhd" => {
                let b = MediaHeaderBox::parse(reader, inner_size);
                Some(Mp4Box::Mdhd(b))
            }
            "hdlr" => {
                let b = HandlerReferenceBox::parse(reader, inner_size);
                Some(Mp4Box::Hdlr(b))
            }
            "minf" => Some(Mp4Box::Container("Media Information Box (container)")),
            "vmhd" => {
                let b = VideoMediaHandler::parse(reader, inner_size);
                Some(Mp4Box::Vmhd(b))
            }
            "smhd" => {
                let b = SoundMediaHandler::parse(reader, inner_size);
                Some(Mp4Box::Smhd(b))
            }
            "dinf" => Some(Mp4Box::Container("Data Information Box (container)")),
            "dref" => {
                let b = DataReferenceBox::parse(reader);
                Some(Mp4Box::Dref(b))
            }
            "stbl" => Some(Mp4Box::Container("Sample Table Box (container)")),
            "stsd" => {
                let b = SampleDescriptionBox::parse_header(reader, inner_size);
                Some(Mp4Box::Stsd(b))
            }
            "stts" => {
                let b = DecodingTimeToSampleBox::parse_header(reader);
                Some(Mp4Box::Stts(b))
            }
            "stss" => {
                let b = SyncSampleBox::parse_header(reader);
                Some(Mp4Box::Stss(b))
            }
            "ctts" => {
                let b = CompositionTimeToSampleBox::parse_header(reader);
                Some(Mp4Box::Ctts(b))
            }
            "stsc" => {
                let b = SampleToChunkBox::parse_header(reader);
                Some(Mp4Box::Stsc(b))
            }
            "stsz" => {
                let b = SampleSizeBox::parse_header(reader);
                Some(Mp4Box::Stsz(b))
            }
            "stco" => {
                let b = ChunkOffsetBox::parse_header(reader);
                Some(Mp4Box::Stco(b))
            }
            "sgpd" => {
                let b = SampleGroupDescriptionBox::parse_header(reader);
                Some(Mp4Box::Sgpd(b))
            }
            "sbgp" => {
                let b = SampleToGroupBox::parse_header(reader);
                Some(Mp4Box::Sbgp(b))
            }
            "sdtp" => {
                let b = SampleDependencyTypeBox::parse_header(reader);
                Some(Mp4Box::Sdtp(b))
            }
            "mvex" => Some(Mp4Box::Container("Movie Extends Box (container)")),
            "trex" => {
                let b = TrackExtendsBox::parse(reader, inner_size);
                Some(Mp4Box::Trex(b))
            }
            "moof" => Some(Mp4Box::Container("Movie Fragment Box (container)")),
            "mfhd" => {
                let b = MovieFragmentHeaderBox::parse(reader, inner_size);
                Some(Mp4Box::Mfhd(b))
            }
            "traf" => Some(Mp4Box::Container("Track Fragment Box (container)")),
            "mfra" => Some(Mp4Box::Container(
                "Movie Fragment Random Access Box (container)",
            )),
            "udta" => Some(Mp4Box::Container("User Data Box (container)")),
            "meta" => {
                FullBoxHeader::parse(reader);
                Some(Mp4Box::Container("The Meta Box (container)"))
            }
            "ilst" => Some(Mp4Box::QuickTimeMetadataItemList(MetadataItemList)),

            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        use Mp4Box::*;
        match self {
            QuickTimeMetadataItemList(_) => "QuickTime Metadata Item List",
            Container(title) => title,
            Ftyp(_) => "File Type Box",
            Mdat => "Media Data Box",
            Free => "Free Space Box",
            Mvhd(_) => "Movie Header Box",
            Tkhd(_) => "Track Header Box",
            Elst(_) => "Edit List Box",
            Mdhd(_) => "Media Header Box",
            Hdlr(_) => "Handler Reference Box",
            Vmhd(_) => "Video Media Header",
            Smhd(_) => "Sound Media Header",
            Dref(_) => "Data Reference Box",
            Stsd(_) => "Sample Description Box",
            Stts(_) => "Decoding Time to Sample Box",
            Stss(_) => "Sync Sample Box",
            Ctts(_) => "Composition Time to Sample Box",
            Stsc(_) => "Sample To Chunk Box",
            Stsz(_) => "Sample Size Box",
            Stco(_) => "Chunk Offset Box",
            Sgpd(_) => "Sample Group Description Box",
            Sbgp(_) => "Sample To Group Box",
            Sdtp(_) => "Sample Dependency Type Box",
            Trex(_) => "Track Extends Box",
            Mfhd(_) => "Movie Fragment Header Box",
        }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        use Mp4Box::*;
        match self {
            QuickTimeMetadataItemList(_) => {}
            Container(_) => {}
            Ftyp(b) => b.print_attributes(print),
            Mdat => {}
            Free => {}
            Mvhd(b) => b.print_attributes(print),
            Tkhd(b) => b.print_attributes(print),
            Elst(b) => b.print_attributes(print),
            Mdhd(b) => b.print_attributes(print),
            Hdlr(b) => b.print_attributes(print),
            Vmhd(b) => b.print_attributes(print),
            Smhd(b) => b.print_attributes(print),
            Dref(b) => b.print_attributes(print),
            Stsd(b) => b.print_attributes(print),
            Stts(b) => b.print_attributes(print),
            Stss(b) => b.print_attributes(print),
            Ctts(b) => b.print_attributes(print),
            Stsc(b) => b.print_attributes(print),
            Stsz(b) => b.print_attributes(print),
            Stco(b) => b.print_attributes(print),
            Sgpd(b) => b.print_attributes(print),
            Sbgp(b) => b.print_attributes(print),
            Sdtp(b) => b.print_attributes(print),
            Trex(b) => b.print_attributes(print),
            Mfhd(b) => b.print_attributes(print),
        }
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

    fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Major brand", &self.major_brand);
        print("Minor version", &self.minor_version);
        print("Compatible", &format!("{:?}", self.compatible_brands));
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

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Created", &self.creation_time);
        print("Modified", &self.modification_time);
        print("Timescale", &self.timescale);
        print("Duration", &self.duration);
        print("Rate", &self.rate);
        print("Volume", &self.volume);
        print("Matrix", &format!("{:?}", self.matrix));
        print("Next track ID", &self.next_track_id);
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

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Enabled", &self.track_enabled);
        print("In movie", &self.track_in_movie);
        print("In preview", &self.track_in_preview);
        print("Created", &self.creation_time);
        print("Modified", &self.modification_time);
        print("Track ID", &self.track_id);
        print("Duration", &self.duration);
        print("Layer", &self.layer);
        print("Alternate group", &self.alternate_group);
        print("Volume", &self.volume);
        print("Matrix", &format!("{:?}", self.matrix));
        print("Dimension", &format!("{} x {}", self.width, self.height));
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

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Created", &self.creation_time);
        print("Modified", &self.modification_time);
        print("Timescale", &self.timescale);
        print("Duration", &self.duration);
        print("Language", &self.language);
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

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Handler type", &self.handler_type);
        print("Name", &self.name);
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

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Graphics mode", &self.graphicsmode);
        print("Opcolor", &format!("{:?}", &self.opcolor));
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

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Balance", &self.balance);
    }
}

/// dref
#[derive(Debug)]
pub struct DataReferenceBox {
    pub entry_count: u32,
}

/// url
#[derive(Debug)]
pub struct DataEntryUrlBox {
    pub self_contained: bool,
}

impl DataReferenceBox {
    pub fn parse(reader: &mut Reader) -> Self {
        FullBoxHeader::parse(reader);
        let entry_count = reader.read_u32();
        Self { entry_count }
    }

    pub fn parse_entry(reader: &mut Reader) -> DataEntryUrlBox {
        let header = BoxHeader::parse(reader);
        if header.box_type != "url " {
            todo!("Handle DataEntryUrnBox");
        }
        let full_box = FullBoxHeader::parse(reader);
        if full_box.flags == [0, 0, 1] {
            DataEntryUrlBox {
                self_contained: true,
            }
        } else {
            todo!("Handle external media URL")
        }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("# entries", &self.entry_count);
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

impl EditListEntry {
    fn parse(reader: &mut Reader) -> Self {
        let segment_duration = reader.read_u32();
        let media_time = reader.read_i32();
        let media_rate_integer = reader.read_i16();
        let media_rate_fraction = reader.read_i16();
        Self {
            segment_duration,
            media_time,
            media_rate_integer,
            media_rate_fraction,
        }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Segment duration", &self.segment_duration);
        print("Media time", &self.media_time);
        print("Media rate", &self.media_rate_integer);
    }
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
        EditListEntry::parse(reader)
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("# entries", &self.entry_count);
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

impl DecodingTimeToSampleEntry {
    fn parse(reader: &mut Reader) -> Self {
        let sample_count = reader.read_u32();
        let sample_delta = reader.read_u32();
        Self {
            sample_count,
            sample_delta,
        }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Sample count", &self.sample_count);
        print("Sample delta", &self.sample_delta);
    }
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
        DecodingTimeToSampleEntry::parse(reader)
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("# entries", &self.entry_count);
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

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("# entries", &self.entry_count);
    }
}

/// ctts
#[derive(Debug)]
pub struct CompositionTimeToSampleBox {
    pub version: u8,
    pub entry_count: u32,
}

impl CompositionTimeToSampleBox {
    pub fn parse_header(reader: &mut Reader) -> Self {
        let full_box = FullBoxHeader::parse(reader);
        let entry_count = reader.read_u32();
        Self {
            version: full_box.version,
            entry_count,
        }

        // TODO: handle entries
        // for i in 0..entry_count {
        //     let sample_count = reader.read_u32();
        //     let sample_offset = reader.read_u32();
        //     logger.trace_box(format!(
        //         "({}) Sample count: {}, sample offset: {}",
        //         i, sample_count, sample_offset,
        //     ));
        // }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("# entries", &self.entry_count);
    }
}

/// stsc
#[derive(Debug)]
pub struct SampleToChunkBox {
    pub entry_count: u32,
}

impl SampleToChunkBox {
    pub fn parse_header(reader: &mut Reader) -> Self {
        FullBoxHeader::parse(reader);
        let entry_count = reader.read_u32();
        Self { entry_count }

        // TODO: handle entries
        // for i in 0..entry_count {
        //     let first_chunk = reader.read_u32();
        //     let samples_per_chunk = reader.read_u32();
        //     let sample_description_index = reader.read_u32();
        //     logger.trace_box(format!(
        //         "({}) First chunk: {}, smpls/chunk: {}, smpl dscr idx: {}",
        //         i, first_chunk, samples_per_chunk, sample_description_index,
        //     ));
        // }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("# entries", &self.entry_count);
    }
}

/// stsz
#[derive(Debug)]
pub struct SampleSizeBox {
    pub sample_size: u32,
    pub sample_count: u32,
}

impl SampleSizeBox {
    pub fn parse_header(reader: &mut Reader) -> Self {
        FullBoxHeader::parse(reader);

        let sample_size = reader.read_u32();
        let sample_count = reader.read_u32();
        Self {
            sample_size,
            sample_count,
        }

        // TODO: handle entries
        // if sample_size == 0 {
        //     for i in 0..sample_count {
        //         let sample_size = reader.read_u32();
        //         logger.trace_box(format!("({}) Sample size: {}", i, sample_size));
        //     }
        // }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Sample size", &self.sample_size);
        print("# samples", &self.sample_count);
    }
}

/// stco
#[derive(Debug)]
pub struct ChunkOffsetBox {
    pub entry_count: u32,
}

impl ChunkOffsetBox {
    pub fn parse_header(reader: &mut Reader) -> Self {
        FullBoxHeader::parse(reader);
        let entry_count = reader.read_u32();
        Self { entry_count }

        // TODO: handle entries
        // for i in 0..entry_count {
        //     let chunk_offset = reader.read_u32();
        //     logger.trace_box(format!("({}) Chunk offset: {}", i, chunk_offset))
        // }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("# entries", &self.entry_count);
    }
}

/// sgpd
#[derive(Debug)]
pub struct SampleGroupDescriptionBox {}

impl SampleGroupDescriptionBox {
    pub fn parse_header(_reader: &mut Reader) -> Self {
        // TODO
        SampleGroupDescriptionBox {}
    }

    pub fn print_attributes<F>(&self, _print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        // TODO
    }
}

/// sbgp
#[derive(Debug)]
pub struct SampleToGroupBox {}

impl SampleToGroupBox {
    pub fn parse_header(_reader: &mut Reader) -> Self {
        // TODO
        SampleToGroupBox {}
    }

    pub fn print_attributes<F>(&self, _print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        // TODO
    }
}

/// sdtp
#[derive(Debug)]
pub struct SampleDependencyTypeBox {}

impl SampleDependencyTypeBox {
    pub fn parse_header(_reader: &mut Reader) -> Self {
        // TODO
        SampleDependencyTypeBox {}
    }

    pub fn print_attributes<F>(&self, _print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        // TODO
    }
}

/// trex
#[derive(Debug)]
pub struct TrackExtendsBox {
    pub track_id: u32,
    pub default_sample_description_index: u32,
    pub default_sample_duration: u32,
    pub default_sample_size: u32,
    pub default_sample_flags: u32,
}

impl TrackExtendsBox {
    pub fn parse(reader: &mut Reader, _inner_size: u64) -> Self {
        FullBoxHeader::parse(reader);
        let track_id = reader.read_u32();
        let default_sample_description_index = reader.read_u32();
        let default_sample_duration = reader.read_u32();
        let default_sample_size = reader.read_u32();
        let default_sample_flags = reader.read_u32();
        Self {
            track_id,
            default_sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
        }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Track ID", &self.track_id);
        print(
            "Default smpl. descr. index",
            &self.default_sample_description_index,
        );
        print("Default sample duration", &self.default_sample_duration);
        print("Default sample size", &self.default_sample_size);
        print("Default sample flags", &self.default_sample_flags);
    }
}

/// mfhd
#[derive(Debug)]
pub struct MovieFragmentHeaderBox {
    pub sequence_number: u32,
}

impl MovieFragmentHeaderBox {
    pub fn parse(reader: &mut Reader, _inner_size: u64) -> Self {
        FullBoxHeader::parse(reader);
        let sequence_number = reader.read_u32();
        Self { sequence_number }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Sequence number", &self.sequence_number);
    }
}

/// stsd
#[derive(Debug)]
pub struct SampleDescriptionBox {
    pub entry_count: u32,
}

impl SampleDescriptionBox {
    pub fn parse_header(reader: &mut Reader, _inner_size: u64) -> Self {
        FullBoxHeader::parse(reader);

        let entry_count = reader.read_u32();
        Self { entry_count }
    }

    pub fn parse_entry(&self, reader: &mut Reader) -> SampleEntry {
        let header = BoxHeader::parse(reader);
        match header.box_type.as_ref() {
            "mp4a" => SampleEntry::Mp4a(Mp4aAudioSampleEntry::parse(reader)),
            "avc1" => SampleEntry::Avc1(Avc1VisualSampleEntry::parse(reader)),
            _ => panic!("Unhandled sample description entry: {}", header.box_type),
        }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("# entries", &self.entry_count);
    }
}

#[derive(Debug)]
pub enum SampleEntry {
    Mp4a(Mp4aAudioSampleEntry),
    Avc1(Avc1VisualSampleEntry),
}

impl SampleEntry {
    pub fn name(&self) -> &'static str {
        match self {
            SampleEntry::Mp4a(_) => "AudioSampleEntry(mp4a)",
            SampleEntry::Avc1(_) => "VisualSampleEntry(avc1)",
        }
    }

    pub fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        match self {
            SampleEntry::Mp4a(mp4a) => mp4a.print_attributes(print),
            SampleEntry::Avc1(avc1) => avc1.print_attributes(print),
        }
    }
}

/// mp4a
#[derive(Debug)]
pub struct Mp4aAudioSampleEntry {
    pub data_reference_index: u16,
    pub channel_count: u16,
    pub sample_size: u16,
    pub sample_rate: f32,
}

impl Mp4aAudioSampleEntry {
    fn parse(reader: &mut Reader) -> Self {
        let _reserved = reader.read_string(6);
        let data_reference_index = reader.read_u16();

        //let mut remaining = inner_size - 8;

        // https://www.fatalerrors.org/a/analysis-of-mp4-file-format.html

        let _reserved = reader.read_bytes(4 * 2);
        let channel_count = reader.read_u16();
        let sample_size = reader.read_u16();
        let _predefined = reader.read_bytes(2);
        let _reserved = reader.read_bytes(2);
        let sample_rate = reader.read_fixed_point_16_16();

        //remaining -= 20;

        // TODO ?
        // parse_container_sub_boxes(reader, remaining, logger, HandleUnknown::Skip);

        Self {
            data_reference_index,
            channel_count,
            sample_size,
            sample_rate,
        }
    }

    fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Data reference index", &self.data_reference_index);
        print("Channel count", &self.channel_count);
        print("Sample size", &self.sample_size);
        print("Sample rate", &self.sample_rate);
    }
}

/// avc1
#[derive(Debug)]
pub struct Avc1VisualSampleEntry {
    pub data_reference_index: u16,
    pub width: u16,
    pub height: u16,
    pub hor_resolution: f32,
    pub ver_resolution: f32,
    pub frame_count: u16,
    pub compressor_name: String,
    pub depth: u16,
}

impl Avc1VisualSampleEntry {
    fn parse(reader: &mut Reader) -> Self {
        let _reserved = reader.read_string(6);
        let data_reference_index = reader.read_u16();
        //let mut remaining = inner_size - 8;

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

        //remaining -= 70;

        Self {
            data_reference_index,
            width,
            height,
            hor_resolution,
            ver_resolution,
            frame_count,
            compressor_name,
            depth,
        }

        // TODO ?
        // parse_container_sub_boxes(reader, remaining, logger, HandleUnknown::Skip);
    }

    fn print_attributes<F>(&self, print: F)
    where
        F: Fn(&str, &dyn std::fmt::Display),
    {
        print("Data reference index", &self.data_reference_index);
        print("Width", &self.width);
        print("Height", &self.height);
        print("Hor. resolution", &self.hor_resolution);
        print("Ver. resolution", &self.ver_resolution);
        print("Frame count", &self.frame_count);
        print("Compressor name", &self.compressor_name);
        print("Depth", &self.depth);
    }
}

fn as_timestamp(epoch_secs: u32) -> NaiveDateTime {
    let epoch_1904: NaiveDateTime = NaiveDate::from_ymd(1904, 1, 1).and_hms(0, 0, 0);
    epoch_1904 + Duration::seconds(epoch_secs as i64)
}

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
        let box_type = reader.read_bytes(4);
        let box_type = String::from_utf8(box_type).unwrap_or_else(|e| {
            // QuickTime has boxes that begin with the copyright symbol ©, but it's
            // encoded as a single byte 0xA9. For these boxes the box_type is not valid UTF-8.
            let buf = e.into_bytes();
            let u16_buf = [buf[0] as u16, buf[1] as u16, buf[2] as u16, buf[3] as u16];
            String::from_utf16(&u16_buf).unwrap()
        });

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
