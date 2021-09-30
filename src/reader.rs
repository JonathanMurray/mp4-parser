use std::convert::TryInto;
use std::io::{Cursor, Read, Seek, SeekFrom};

pub struct Reader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(buf),
        }
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn read_u8(&mut self) -> u8 {
        let mut buf = [0; 1];
        self.cursor.read_exact(&mut buf).unwrap();
        u8::from_be_bytes((&buf[..]).try_into().unwrap())
    }

    pub fn read_u16(&mut self) -> u16 {
        let mut buf = [0; 2];
        self.cursor.read_exact(&mut buf).unwrap();
        u16::from_be_bytes((&buf[..]).try_into().unwrap())
    }

    pub fn read_i16(&mut self) -> i16 {
        let mut buf = [0; 2];
        self.cursor.read_exact(&mut buf).unwrap();
        i16::from_be_bytes((&buf[..]).try_into().unwrap())
    }

    pub fn read_u32(&mut self) -> u32 {
        let mut buf = [0; 4];
        self.cursor.read_exact(&mut buf).unwrap();
        u32::from_be_bytes((&buf[..]).try_into().unwrap())
    }

    pub fn read_i32(&mut self) -> i32 {
        let mut buf = [0; 4];
        self.cursor.read_exact(&mut buf).unwrap();
        i32::from_be_bytes((&buf[..]).try_into().unwrap())
    }

    pub fn read_u64(&mut self) -> u64 {
        let mut buf = [0; 8];
        self.cursor.read_exact(&mut buf).unwrap();
        u64::from_be_bytes((&buf[..]).try_into().unwrap())
    }

    pub fn read_fixed_point_16_16(&mut self) -> f32 {
        let mut buf = [0; 4];
        self.cursor.read_exact(&mut buf).unwrap();
        let n = u32::from_be_bytes((&buf[..]).try_into().unwrap());
        n as f32 / 2_u32.pow(16) as f32
    }

    pub fn read_fixed_point_8_8(&mut self) -> f32 {
        let mut buf = [0; 2];
        self.cursor.read_exact(&mut buf).unwrap();
        let n = u16::from_be_bytes((&buf[..]).try_into().unwrap());
        n as f32 / 2_u32.pow(8) as f32
    }

    pub fn read_string(&mut self, len: usize) -> String {
        let mut buf = Vec::new();
        buf.resize(len, 0);
        self.cursor.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    pub fn read_string_inexact(&mut self, max_len: usize) -> String {
        let mut buf = Vec::new();
        buf.resize(max_len, 0);
        let _n_read = self.cursor.read(&mut buf).unwrap();
        String::from_utf8_lossy(&buf).to_string()
    }

    pub fn read_bytes(&mut self, n_bytes: usize) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.resize(n_bytes, 0);
        self.cursor.read_exact(&mut buf).unwrap();
        buf
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) {
        self.cursor.read_exact(buf).unwrap();
    }

    pub fn skip_bytes(&mut self, n_bytes: u32) -> Result<(), String> {
        let pos = self.cursor.position();
        let target = pos + n_bytes as u64;
        let file_len = self.cursor.get_ref().len() as u64;
        if target > file_len {
            let err = format!(
                "Seeking {} from {} would land on {}, but the file is only {} bytes long",
                n_bytes, pos, target, file_len
            );
            return Err(err);
        }
        self.cursor.seek(SeekFrom::Current(n_bytes as i64)).unwrap();
        Ok(())
    }
}
