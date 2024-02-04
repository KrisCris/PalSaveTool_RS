use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
};

/**
 * if zlib_compress_times == '2'
 *      decompressed_len = 2nd decompressed size, i.e. uncompressed size
 *      compressed_len = 1st decompressed size
 * zlib_compress_times = '1' or '2'.
 * magic_bytes == 'PlZ'
 */
pub struct PalSave {
    decompressed_len: u32,
    compressed_len: u32,
    magic_bytes: [u8; 3],
    zlib_compress_times: char,
    body: Vec<u8>,
}

impl PalSave {
    const MAGIC_BYTES: &[u8] = b"PlZ";

    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = fs::read(path)?;
        Self::from_bytes(&data)
    }

    fn from_bytes(data: &[u8]) -> io::Result<Self> {
        if data.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Data too short",
            ));
        }

        let decompressed_len = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let compressed_len = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let magic_bytes = data[8..11].try_into().unwrap();
        let zlib_compress_times = data[11] as char;
        let body = data[12..].to_vec();

        if magic_bytes != PalSave::MAGIC_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid magic bytes `{:?}`, not a compressed Palworld save.",
                    magic_bytes
                ),
            ));
        }

        match zlib_compress_times {
            '1' => {
                if data.len() - 12 != compressed_len as usize {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unmatched file length.",
                    ));
                }
            }
            '2' => {}
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsupported zlib_compress_times `'{zlib_compress_times}'`."),
                ))
            }
        }

        Ok(Self {
            decompressed_len,
            compressed_len,
            magic_bytes,
            zlib_compress_times,
            body,
        })
    }

    pub fn get_decompressed_body(&self) -> io::Result<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(&self.body[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        if self.zlib_compress_times == '2' {
            if decompressed_data.len() != self.compressed_len as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unmatched file length.",
                ));
            }

            let mut decoder = ZlibDecoder::new(&decompressed_data[..]);
            let mut decompressed_data_2 = Vec::new();
            decoder.read_to_end(&mut decompressed_data_2)?;

            decompressed_data = decompressed_data_2;
        }

        if decompressed_data.len() != self.decompressed_len as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unmatched file length.",
            ));
        }

        Ok(decompressed_data)
    }

    pub fn update(&mut self, modified_body: &Vec<u8>) -> io::Result<()> {
        let decompressed_len = modified_body.len() as u32;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&modified_body)?;
        let mut compressed_data = encoder.finish()?;

        let compressed_len = compressed_data.len() as u32;

        if self.zlib_compress_times == '2' {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&compressed_data)?;
            compressed_data = encoder.finish()?;
        }

        self.decompressed_len = decompressed_len;
        self.compressed_len = compressed_len;
        self.body = compressed_data;

        Ok(())
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;

        file.write_all(&self.decompressed_len.to_le_bytes())?;
        file.write_all(&self.compressed_len.to_le_bytes())?;

        file.write_all(&self.magic_bytes)?;

        file.write_all(&[self.zlib_compress_times as u8])?;
        file.write_all(&self.body)?;

        Ok(())
    }

    pub fn from_decompressed_file<P: AsRef<Path>>(path: P, zlib_compress_times: char) -> io::Result<Self> {
        let data = fs::read(path)?;
        let mut palsav_data = PalSave {
            decompressed_len: data.len() as u32,
            compressed_len: 0,
            magic_bytes: Self::MAGIC_BYTES.try_into().unwrap(),
            zlib_compress_times: zlib_compress_times,
            body: Vec::new(),
        };
        palsav_data.update(&data)?;

        Ok(palsav_data)
    }
}

#[cfg(test)]
mod test {
    use crate::PalSave;
    use std::io;

    #[test]
    fn test_file_save() -> io::Result<()> {
        let mut level_sav = PalSave::from_file("~/Downloads/Level.sav")?;

        let save_data = level_sav.get_decompressed_body()?;
        level_sav.update(&save_data)?;

        level_sav.to_file("~/Downloads/Level.new.sav")?;

        let level_sav_new = PalSave::from_file("~/Downloads/Level.new.sav")?;

        assert_eq!(save_data, level_sav_new.get_decompressed_body().unwrap());
        Ok(())
    }
}
