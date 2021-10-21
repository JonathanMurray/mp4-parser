use std::fs::File;
use std::io::Read;

use clap::{App, Arg};

use mp4_parser::boxes::{BoxHeader, Mp4Box, SampleEntry};
use mp4_parser::reader::Reader;

fn main() {
    let matches = App::new("mp4-info")
        .about("Show high-level info of an MP4 file")
        .arg(
            Arg::with_name("FILE")
                .help("The mp4 file that should be analyzed")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = matches.value_of("FILE").unwrap();
    let mut f = File::open(&path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    let parser = Parser::new();
    let info = parser.parse_mp4(&mut buf);
    println!("{:#?}", info);
}

#[derive(Debug)]
struct Info {
    tracks: Vec<Track>,
}

#[derive(Debug)]
struct Track {
    id: u32,
    info: TrackInfo,
}

#[derive(Debug)]
enum TrackInfo {
    Audio(AudioTrack),
    Video(VideoTrack),
}

#[derive(Debug)]
struct AudioTrack {
    channel_count: u16,
    sample_rate: f32,
}

#[derive(Debug)]
struct VideoTrack {
    width: u16,
    height: u16,
}

struct Parser {
    tracks: Vec<Track>,
    current_track: Option<TrackBuilder>,
}

struct TrackBuilder {
    id: Option<u32>,
    info: Option<TrackInfo>,
}

impl Parser {
    fn new() -> Self {
        Self {
            tracks: vec![],
            current_track: None,
        }
    }

    fn parse_mp4(mut self, buf: &mut Vec<u8>) -> Info {
        let mut reader = Reader::new(buf);

        self.parse(&mut reader, buf.len() as u64);

        Info {
            tracks: self.tracks,
        }
    }

    fn parse(&mut self, reader: &mut Reader, end_offset: u64) {
        while reader.position() < end_offset {
            let box_start_offset = reader.position();
            let header = BoxHeader::parse(reader);

            if &header.box_type == "trak" {
                // We will build a Track from this box's children
                self.current_track = Some(TrackBuilder {
                    id: None,
                    info: None,
                });
            }

            let box_ = Mp4Box::parse_contents(reader, &header.box_type, header.inner_size);

            let box_ = match box_ {
                Some(b) => b,
                None => {
                    reader
                        .skip_bytes(header.inner_size as u32)
                        .unwrap_or_else(|e| panic!("Truncated '{}' box: {}", header.box_type, e));
                    continue;
                }
            };

            let box_end_offset = box_start_offset + header.box_size;
            match box_ {
                Mp4Box::Container(_) => {
                    self.parse(reader, box_end_offset);
                }
                Mp4Box::Tkhd(track_header_box) => {
                    self.current_track.as_mut().unwrap().id = Some(track_header_box.track_id);
                }
                Mp4Box::Stsd(sample_description_box) => {
                    for _ in 0..sample_description_box.entry_count {
                        let info = match sample_description_box.parse_entry(reader) {
                            SampleEntry::Mp4a(mp4a) => TrackInfo::Audio(AudioTrack {
                                channel_count: mp4a.channel_count,
                                sample_rate: mp4a.sample_rate,
                            }),
                            SampleEntry::Avc1(avc1) => TrackInfo::Video(VideoTrack {
                                width: avc1.width,
                                height: avc1.height,
                            }),
                        };
                        self.current_track.as_mut().unwrap().info = Some(info);
                    }
                }
                _ => {}
            }

            let remaining = (box_end_offset - reader.position()) as u32;
            if remaining > 0 {
                reader.skip_bytes(remaining).unwrap();
            }

            if &header.box_type == "trak" {
                let track_builder = self.current_track.take().unwrap();
                let id = track_builder.id.unwrap();
                let info = track_builder.info.unwrap();
                self.tracks.push(Track { id, info });
            }
        }
    }
}
