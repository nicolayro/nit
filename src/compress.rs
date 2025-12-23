use std::io;
use std::io::prelude::*;

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

pub fn compress_content(header: String, data: Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&header.into_bytes())?;
    encoder.write_all(&data)?;
    encoder.finish()
}

pub fn compress(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&bytes)?;
    encoder.finish()
}

pub fn decompress(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut out = Vec::new();
    z.read_to_end(&mut out)?;
    Ok(out)
}

