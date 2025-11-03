use std::{fmt, fs};
use std::os::unix::fs::MetadataExt;
use std::str::FromStr;

use sha1::{Sha1, Digest};
use chrono::DateTime;

use std::io;
use std::io::prelude::*;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::read::ZlibEncoder;

fn main() { 
    let filename =
        String::from("examples/example_tree");

    let content = fs::read(filename).unwrap();
    let mut decoded = &decompress(content).unwrap()[..];
    
    let tree = Tree::read(&mut decoded);

    for entry in tree.entries {
        println!("{}", entry);
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

    fn from_bytes(header: String, content: Vec<u8>) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(header);
        hasher.update(content);
        Hash(hasher.finalize().into())
    }
}

#[derive(Debug, Copy, Clone)]
enum ObjectKind {
    Blob = 100644,
    Tree = 040000
}

impl FromStr for ObjectKind {

    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "100644" => Ok(ObjectKind::Blob),
            "40000"  => Ok(ObjectKind::Tree),
            _ => panic!("ERROR: Invalid object mode: {}", input)
        }
    }

}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Blob => write!(f, "blob"),
            ObjectKind::Tree => write!(f, "tree"),
        }
    }
}

fn hash_object(object_type: ObjectKind, content: Vec<u8>) -> Hash {
    match object_type {
        ObjectKind::Blob => hash_blob(content),
        ObjectKind::Tree => hash_tree(content)
    }

}

fn hash_blob(content: Vec<u8>) -> Hash {
    let header = format!("{} {}\0", ObjectKind::Blob, content.len());
    Hash::from_bytes(header, content)
}

fn hash_tree(content: Vec<u8>) -> Hash {
    let header = format!("{} {}\0", ObjectKind::Tree, content.len());
    Hash::from_bytes(header, content)
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
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

fn take_n_bytes(input: &mut &[u8], n: usize) -> Vec<u8> {
    let (bytes, rest) = input.split_at(n);
    *input = rest;
    bytes.to_vec()
}

fn compress(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut z = ZlibEncoder::new(&bytes[..], Compression::fast());
    let mut buffer = Vec::new();
    z.read_to_end(&mut buffer)?;
    Ok(buffer)
}

fn decompress(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut z = ZlibDecoder::new(&bytes[..]);
    let mut out = Vec::new();
    z.read_to_end(&mut out)?;
    Ok(out)
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

impl Index {
    fn from_staged(filename: &str) -> Self {
        let contents = fs::read(filename).unwrap();
        let (hbytes, ebytes) = contents.split_at(12);

        let header = Self::parse_header(hbytes);

        let entries = Self::parse_entries(ebytes, header.num_entries as usize);
        Self { header, entries }
    }

    fn parse_header(mut bytes: &[u8]) -> IndexHeader {
        let signature = take_u32(&mut bytes);
        let version = take_u32(&mut bytes);
        let num_entries = take_u32(&mut bytes);

        IndexHeader { signature, version, num_entries }
    }

    fn parse_entries(mut bytes: &[u8], num_entries: usize) -> Vec<IndexEntry> {
        let mut entries = Vec::with_capacity(num_entries);

        for _ in 0..num_entries {
            let entry = IndexEntry::read(&mut bytes);

            // Pad 1-8 nul bytes as necessary to pad the entry 
            // to a multiple of eight bytes 
            let padding_len = 8 - ((6 + entry.name_len()) % 8);
            take_n_bytes(&mut bytes, padding_len);

            entries.push(entry);
        }

        entries
    }
}

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

    /* Variable length name entry (relative unix path)*/
    name: String,
}

impl IndexEntry {
    fn create(key: Hash, filename: &String) -> Self{
        let stat = fs::metadata(filename).unwrap();

        let ctime_sec  = stat.ctime() as u32;
        let ctime_nano = stat.ctime_nsec() as u32;
        let mtime_sec  = stat.mtime() as u32;
        let mtime_nano = stat.mtime_nsec() as u32;
        let dev        = stat.dev() as u32;
        let ino        = stat.ino() as u32;
        let mode       = (1000 & 0xFFF) << 12 | 0o0644 & 0xFFF;
        let uid        = stat.uid() as u32;
        let gid        = stat.gid() as u32;
        let size       = stat.len() as u32;
        let flags      = filename.len() as u16;
        let name = filename.clone();

        IndexEntry {
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
            flags,
            name
        }
    }

    fn read(bytes: &mut &[u8]) -> Self {
        let ctime_sec  = take_u32(bytes);
        let ctime_nano = take_u32(bytes);
        let mtime_sec  = take_u32(bytes);
        let mtime_nano = take_u32(bytes);
        let dev        = take_u32(bytes);
        let ino        = take_u32(bytes);
        let mode       = take_u32(bytes);
        let uid        = take_u32(bytes);
        let gid        = take_u32(bytes);
        let size       = take_u32(bytes);
        let key        = take_hash(bytes);
        let flags      = take_u16(bytes);
        let name_len = Self::name_len_from_flags(flags);
        let name_bytes = take_n_bytes(bytes, name_len);
        let name = String::from_utf8(name_bytes)
            .expect("ERROR: Unable to read file name");

        IndexEntry {
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
            flags,
            name
        }
    }

    fn name_len(&self) -> usize {
        Self::name_len_from_flags(self.flags)
    }

    fn name_len_from_flags(flags: u16) -> usize {
        (flags & 0x0FFF).into()
    }

    fn object_type(self: &Self) -> u32 {
        // First 4 bits
        (self.mode >> 12) & 0x00F
    }

    fn permission(self: &Self) -> u32 {
        // Final 9 bits
        self.mode & 0x1FF
    }
}

impl fmt::Display for IndexEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02o}{:04o} {} {}       {}", 
            self.object_type(),
            self.permission(),
            self.key,
            0,
            self.name)
    }
}

impl fmt::Debug for IndexEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "  ctime {}:{}", self.ctime_sec, self.ctime_nano)?;
        writeln!(f, "  mtime {}:{}", self.mtime_sec, self.mtime_nano)?;
        writeln!(f, "  dev   {}", self.dev)?;
        writeln!(f, "  ino   {}", self.ino)?;
        writeln!(f, "  mode  {}", self.mode)?;
        writeln!(f, "  uid   {}", self.uid)?;
        writeln!(f, "  gid   {}", self.gid)?;
        writeln!(f, "  size  {}", self.size)?;
        writeln!(f, "  key   {}", self.key)?;
        writeln!(f, "  flags {}", self.flags)?;
        writeln!(f, "  name  {}", self.name)?;
        writeln!(f, "}}")
    }
}


struct Tree {
    entries: Vec<TreeEntry>
}

struct TreeEntry {
    key: Hash,
    mode: ObjectKind,
    name: String,
}

impl Tree {
    fn read(bytes: &mut &[u8]) -> Self{
        Tree::read_header(bytes);

        let mut entries = Vec::new();
        while let Some(entry) = TreeEntry::read(bytes) {
            entries.push(entry);
        };

        Tree { entries }
    }

    fn read_header(bytes: &mut &[u8]) {
        if let Some(pos) = bytes.iter().position(|&x| x == 0) {
            let (content, rest) = bytes.split_at(pos);
            let data: &str = str::from_utf8(content).unwrap();
            *bytes = &rest[1..];
        }
    }

    fn write_tree(&self) -> Vec<u8> {
        self.entries
            .iter()
            .map(|entry| entry.as_bytes())
            .flatten()
            .collect()
    }
}

impl TreeEntry {
    fn read(bytes: &mut &[u8]) -> Option<Self> {
         if let Some(pos) = bytes.iter().position(|&x| x == 0) {
             let (content, rest) = bytes.split_at(pos);
             let data: Vec<&str> = str::from_utf8(content).unwrap()
                 .split(" ")
                 .collect();

             let mode: ObjectKind = ObjectKind::from_str(data[0]).unwrap();
             let name = data[1].parse().unwrap();

             *bytes = &rest[1..];

             let key = take_hash(bytes);

             Some(
                 TreeEntry {
                     mode,
                     name,
                     key
                 }
             )
         } else {
             None
         }
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = format!(
            "{:06} {}\0", 
            self.mode as i32,
            self.name,
        ).into_bytes();
        bytes.extend_from_slice(&self.key.0);
        bytes
    }
}

impl fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:06} {} {}    {}", 
            self.mode as i32,
            self.mode,
            self.key,
            self.name)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_hash() {
        let input = String::from("The quick brown fox jumps over the lazy dog");

        let hashed = Hash::from_string(input).to_string();

        let expected = String::from("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
        assert_eq!(hashed, expected);
    }

    #[test]
    fn hash_blob_object() {
        let content = String::from("what is up, doc?").into_bytes();

        let hashed = hash_blob(content).to_string();

        let expected = String::from("bd9dbf5aae1a3862dd1526723246b20206e5fc37");
        assert_eq!(hashed, expected);
    }

    #[test]
    fn parse_header_from_index() {
        let filename = String::from("examples/example_index");

        let index = Index::from_staged(&filename);
        let bytes: [u8; 4] = index.header.signature.to_be_bytes();
        let actual = str::from_utf8(&bytes).unwrap();

        let expected = "DIRC";
        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_entry_hash_from_index() {
        let filename = String::from("examples/example_index");

        let index = Index::from_staged(&filename);
        let key = index.entries[0].key.to_string();

        let expected = String::from("ea8c4bf7f35f6f77f75d92ad8ce8349f6e81ddba");
        assert_eq!(key, expected);
    }

    #[test]
    fn parse_mode_from_index() {
        let filename = String::from("examples/example_index");

        let index = Index::from_staged(&filename);
        let object_type = index.entries[0].object_type();
        let permission = index.entries[0].permission();

        let expected_type = 0o10;
        let expected_permission = 0o0644;

        assert_eq!(object_type, expected_type);
        assert_eq!(permission, expected_permission);
    }

    #[test]
    fn list_entry_from_index() {
        let filename = String::from("examples/example_index");

        let index = Index::from_staged(&filename);
        let output = index.entries[5].to_string();

        let expected = 
              "100644 d9fa2b8cd651190f6ff5932113491d0a2995b116 0       examples/blob.c";
        assert_eq!(output, expected);
    }

    #[test]
    fn create_blob_entry_from_file() {
        let filename = String::from("examples/blob.c");
        let contents = fs::read(&filename).unwrap();

        let key = hash_blob(contents);

        let index_entry = IndexEntry::create(key, &filename).to_string();

        let expected = 
            "100644 d9fa2b8cd651190f6ff5932113491d0a2995b116 0       examples/blob.c";

        assert_eq!(index_entry, expected);
    }

    #[test]
    fn parse_entry_hash_from_staging_area() {
        let filename = String::from("examples/example_tree");
        let content = fs::read(&filename).unwrap();
        let mut decoded = &decompress(content).unwrap()[..];

        let index = Tree::read(&mut decoded);
        let index_entry = index.entries[2].to_string();

        let expected = "100644 blob 4b0f39b054979bb74888c6d5bea8fbb03c0ea5de    example_index";
        assert_eq!(index_entry, expected);
    }

    #[test]
    fn create_tree_hash_from_index() {
        let filename = String::from("examples/example_tree");
        let content = fs::read(&filename).unwrap();
        let mut decoded = &decompress(content).unwrap()[..];

        let tree = Tree::read(&mut decoded).write_tree();
        let key = hash_tree(tree).to_string();
        
        let expected = String::from("f37ef49b903a6db9fa814b04f8226569f6d0f592");
        assert_eq!(key, expected);
    }
}
