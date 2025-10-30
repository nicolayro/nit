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
    fn from_string(input: String) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(input);
        Hash(hasher.finalize().into())
    }

    fn from_blob(content: &String) -> Self {
        let blob = format!("{} {}\0{}", Object::Blob, content.len(), content);
        Self::from_string(blob)
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

fn take_u16(input: &mut &[u8]) -> u16 {
    let (int_bytes, rest) = input.split_at(size_of::<u16>());
    *input = rest;
    u16::from_be_bytes(int_bytes.try_into().unwrap())
}

fn take_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

fn take_hash(input: &mut &[u8]) -> Hash {
    let (hashed_bytes, rest) = input.split_at(20);
    *input = rest;
    Hash(hashed_bytes.try_into().unwrap())
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
    ctime_sec: u32,
    ctime_nano: u32,
    /*
     * The last time a file's data changed. 
     *  32-bit ctime seconds and 32-bit ctime nanosecond fractions 
     */
    mtime_sec: u32,
    mtime_nano: u32,
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

impl std::fmt::Display for IndexEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "  ctime {}", 
            timestamp_to_date(self.ctime_sec, self.ctime_nano))?;
        writeln!(f, "  mtime {}", 
            timestamp_to_date(self.mtime_sec, self.mtime_nano))?;
        writeln!(f, "  dev   {}", self.dev)?;
        writeln!(f, "  ino   {}", self.ino)?;
        writeln!(f, "  mode  {}", self.mode)?;
        writeln!(f, "  uid   {}", self.uid)?;
        writeln!(f, "  gid   {}", self.gid)?;
        writeln!(f, "  size  {}", self.size)?;
        writeln!(f, "  key   {}", self.key)?;
        writeln!(f, "  flags {}", self.flags)?;
        writeln!(f, "}}")
    }
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
        let signature = take_u32(&mut bytes);
        let version = take_u32(&mut bytes);
        let num_entries = take_u32(&mut bytes);

        IndexHeader { signature, version, num_entries }
    }

    fn parse_entries(mut bytes: &[u8]) -> Vec<IndexEntry> {
        let mut entries = Vec::new();

        let ctime_sec  = take_u32(&mut bytes);
        let ctime_nano = take_u32(&mut bytes);
        let mtime_sec  = take_u32(&mut bytes);
        let mtime_nano = take_u32(&mut bytes);
        let dev        = take_u32(&mut bytes);
        let ino        = take_u32(&mut bytes);
        let mode       = take_u32(&mut bytes);
        let uid        = take_u32(&mut bytes);
        let gid        = take_u32(&mut bytes);
        let size       = take_u32(&mut bytes);

        let key = take_hash(&mut bytes);
        let flags = take_u16(&mut bytes);

        entries.push(IndexEntry {
            ctime_sec,
            ctime_nano,
            mtime_sec,
            mtime_nano,
            dev,
            ino,
            mode,
            uid,
            gid,
            size,
            key,
            flags
        });

        entries
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_hash() {
        let input = String::from("The quick brown fox jumps over the lazy dog");

        let hashed = Hash::from_string(input);

        let expected = String::from("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
        assert_eq!(hashed.to_string(), expected);
    }

    #[test]
    fn hash_object_blob() {
        let content = String::from("what is up, doc?");

        let hashed = Hash::from_blob(&content);

        let expected = String::from("bd9dbf5aae1a3862dd1526723246b20206e5fc37");
        assert_eq!(hashed.to_string(), expected);
    }


    #[test]
    fn parse_index_header() {
        let filename = "example_index";

        let index = Index::from_blob(filename);
        let bytes: [u8; 4] = index.header.signature.to_be_bytes();
        let actual = str::from_utf8(&bytes).unwrap();

        let expected = "DIRC";
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_index_entry_hash() {
        let filename = "example_index";

        let index = Index::from_blob(filename);
        let key = index.entries[0].key.to_string();

        let expected = String::from("ea8c4bf7f35f6f77f75d92ad8ce8349f6e81ddba");
        assert_eq!(key, expected);
    }
}

