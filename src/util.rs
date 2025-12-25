use crate::hash::*;

use std::fs;
use std::io;

use std::path::Path;

pub fn take_u16(input: &mut &[u8]) -> u16 {
    let (int_bytes, rest) = input.split_at(size_of::<u16>());
    *input = rest;
    u16::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn take_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

pub fn take_hash(input: &mut &[u8]) -> Hash {
    let (hashed_bytes, rest) = input.split_at(20);
    *input = rest;
    Hash(hashed_bytes.try_into().unwrap())
}

pub fn take_n_bytes(input: &mut &[u8], n: usize) -> Vec<u8> {
    let (bytes, rest) = input.split_at(n);
    *input = rest;
    bytes.to_vec()
}

pub fn write_to_file(path_str: String, content: Vec<u8>) -> Result<(), io::Error> {
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}
