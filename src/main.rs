use std::fs;

use sha1::{Sha1, Digest};
use chrono::DateTime;

fn main() { }

enum Object {
    Blob,
    Tree
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Blob => write!(f, "blob"),
            Object::Tree => write!(f, "tree"),
        }
    }
}

#[derive(Debug)]
struct Hash([u8; 20]);

impl Hash {
    fn from(input: String) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(input);
        Hash(hasher.finalize().into())
    }


    fn blob(content: &String) -> Self {
        let blob = format!("{} {}\0{}", Object::Blob, content.len(), content);
        Self::from(blob)
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

enum Mode {
    File = 100644,
    Tree = 040000
}

fn read_be_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

fn timestamp_to_date(seconds: u32, nanoseconds: u32) -> String {
    let seconds: i64 = i64::from(seconds);
    let dt = DateTime::from_timestamp(seconds, nanoseconds);
    match dt {
        Some(date) => format!("{}", date),
        None => String::from("")
    }
}


#[derive(Debug)]
struct Index {
    header: IndexHeader,
    entries: Vec<IndexEntry>
}

#[derive(Debug)]
struct IndexHeader {
    /* 
     * 4-byte signature: 
     *  The signature is { 'D', 'I', 'R', 'C' } (stands for "dircache") 
     */
    signature: u32,
    /*
     * 4-byte version number:
     *  The current supported versions are 2, 3 and 4.
    */
    version: u32,
    /* 32-bit number of index entries */
    num_entries: u32
}

#[derive(Debug)]
struct IndexEntry {
    /*
     * The last time a file's metadata changed. 
     * 32-bit ctime seconds and 32-bit ctime nanosecond fractions 
     */
    ctime: u64,
    /*
     * The last time a file's data changed. 
     *  32-bit ctime seconds and 32-bit ctime nanosecond fractions 
     */
    mtime: u64,
    /* stat(2) data */
    dev: u32,
    /* stat(2) data */
    ino: u32,
    /*
     * Mode:
     *  4-bit object type. Valid values in binary are 
     *    1000 (regular file), 1010 (symbolic link) and 1110 (gitlink)
     *  3-bit unused
     *  9-bit unix permission. 
     *   Only 0755 and 0644 are valid for regular files.
     *   Symbolic links and gitlinks have value 0 in this field
     */
    mode: u32,
    /* stat(2) data */
    uid: u32,
    /* stat(2) data */
    gid: u32,
    /* on-disk file size from stat(2) */
    size: u32,
    /* object name (SHA-1 hash) */
    key: Hash,
    /*
     * A 16-bit 'flags' field split into (high to low bits)
     *   1-bit assume-valid flag
     *   1-bit extended flag (must be zero in version 2)
     *   2-bit stage (during merge)
     *   12-bit name length if the length is less than 0xFFF; otherwise 0xFFF
     *   is stored in this field.
    */
    flags: u16,
}


impl Index {
    fn from_blob(filename: &str) -> Self {
        let contents = fs::read(filename).unwrap();
        let (hbytes, ebytes) = contents.split_at(12);

        let header = Self::parse_header(hbytes);
        let entries = Self::parse_entries(ebytes);
        Self { header, entries }
    }

    fn parse_header(mut bytes: &[u8]) -> IndexHeader {
        let signature = read_be_u32(&mut bytes);
        let version = read_be_u32(&mut bytes);
        let num_entries = read_be_u32(&mut bytes);

        IndexHeader { signature, version, num_entries }
    }

    fn parse_entries(mut bytes: &[u8]) -> Vec<IndexEntry> {
        for i in 0..10 {
            let ctime_seconds = read_be_u32(&mut bytes);
            let ctime_nanoseconds = read_be_u32(&mut bytes);
            println!(
                "{:2}{:16}{:16} {:?}", 
                i, 
                ctime_seconds, 
                ctime_nanoseconds, 
                timestamp_to_date(ctime_seconds, ctime_nanoseconds)
            );
        }

        Vec::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_hash() {
        let input = String::from("The quick brown fox jumps over the lazy dog");

        let hashed = Hash::from(input);

        let expected = String::from("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
        assert_eq!(hashed.to_string(), expected);
    }

    #[test]
    fn hash_object_blob() {
        let content = String::from("what is up, doc?");

        let hashed = Hash::blob(&content);

        let expected = String::from("bd9dbf5aae1a3862dd1526723246b20206e5fc37");
        assert_eq!(hashed.to_string(), expected);
    }


    #[test]
    fn index_entry_from_blob() {
        let filename = ".git/index";

        let data = Index::from_blob(filename);
        let bytes: [u8; 4] = data.header.signature.to_be_bytes();
        let actual = str::from_utf8(&bytes).unwrap();

        let expected = "DIRC";
        assert_eq!(actual, expected);
    }
}

