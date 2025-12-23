use crate::hash::*;
use chrono::DateTime;

// TODO: Finn på bedre navn her
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

pub fn timestamp_to_date(seconds: u32, nanoseconds: u32) -> String {
    let seconds: i64 = i64::from(seconds);
    let dt = DateTime::from_timestamp(seconds, nanoseconds);
    match dt {
        Some(date) => format!("{}", date),
        None => String::from("")
    }
}
