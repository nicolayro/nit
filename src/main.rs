use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::fs::File;

use std::str::FromStr;
use std::os::unix::fs::MetadataExt;

use std::io::prelude::*;

use chrono::DateTime;
use sha1::{Sha1, Digest};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::write::DeflateEncoder;

fn main() -> Result<(), io::Error> {
    let root = ".git";

    // Git add
    //  hash-object
    let content = fs::read("main.c")?;
    let blob = hash_blob(content.clone());

    //  -w (store the object)
    let path_str = format!("{}/{}", root, blob.to_object_path());
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let header = format!("{} {}\0", ObjectKind::Blob, content.len());
    let compressed = compress_content(header, content)?;
    fs::write(path, compressed).unwrap();

    //  update-index
    let hash = blob;
    let filename = String::from("main.c");
    let entry = IndexEntry::create(hash, &filename);
    let mut entries: Vec<IndexEntry> = Vec::new();
    entries.push(entry);

    let index_header = IndexHeader {
        signature: u32::from_be_bytes([ b'D', b'I', b'R', b'C' ]),
        version: 2 as u32,
        num_entries: 1 as u32,
    };
    let index = Index {
        header: index_header,
        entries: entries
    };

    let index_bytes = index.to_bytes();

    let mut index = File::create(&String::from(".git/index"))
        .expect("ERROR: Unable to open index file");
    index.write_all(&index_bytes);
    

    //  write-tree
    let filename = String::from(".git/index");

    let index = Index::read(&filename);
    let tree = index.to_tree_bytes();

    let key = hash_tree(tree.clone());

    let path_str = format!("{}/{}", root, key.to_object_path());
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let header = format!("{} {}\0", ObjectKind::Tree, tree.len());
    let compressed = compress_content(header, tree)?;
    fs::write(path, compressed).unwrap();

    // Git commit
    let content = fs::read(&path).unwrap();
    let mut decoded = &decompress(content).unwrap()[..];

    let tree = Tree::read(&mut decoded.clone());
    for entry in tree.entries {
        println!("{}", entry);
    }

    let key = Hash::from_bytes(String::from(""), decoded.to_vec());
    let parent = Hash::from_hex("1305b699328eb20a3e0aed739c0ff05fffee698c");
    let author = Stamp {
        name: "Nicolay Roness".to_string(),
        email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
        timestamp: 1762103153 
    };

    let committer = Stamp {
        name: "Nicolay Roness".to_string(),
        email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
        timestamp: 1762103153 
    };
    let message = String::from("Make a commit");
    let commit = Commit::create(key, Some(parent), author, committer, message)
        .to_string();
    println!("{}", commit);

    // Store commit
    // Update refs


    Ok(())
}

#[derive(Debug)]
struct Hash([u8; 20]);

impl Hash {
    fn from_hex(hash: &str) -> Self {
        if hash.len() != 40 {
            panic!("ERROR: Hex encoded hash must be 40 characters, received '{}': {}",
                hash.len(),
                hash
            )
        }
        let bytes = match hex::decode(hash) {
            Ok(b) => b,
            Err(e) => panic!("ERROR: Invalid hash '{}'", hash)
        };

        Hash(bytes.try_into().unwrap())
    }

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

    fn to_object_path(&self) -> String {
        let (l1, l2) = self.0.split_at(1);
        format!("objects/{}/{}", hex::encode(l1), hex::encode(l2))
    }
}
    
#[derive(Debug, Copy, Clone)]
enum ObjectKind {
    Blob = 100644,
    Tree = 040000,
    Commit = 0
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
            ObjectKind::Commit => write!(f, "commit"),
        }
    }
}

fn hash_object(object_type: ObjectKind, content: Vec<u8>) -> Hash {
    match object_type {
        ObjectKind::Blob => hash_blob(content),
        ObjectKind::Tree => hash_tree(content),
        ObjectKind::Commit => hash_commit(content)
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

fn hash_commit(content: Vec<u8>) -> Hash {
    let header = format!("{} {}\0", ObjectKind::Commit, content.len());
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

fn compress_content(header: String, data: Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&header.into_bytes())?;
    encoder.write_all(&data)?;
    encoder.finish()
}

fn compress(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&bytes)?;
    encoder.finish()
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
    fn read(filename: &str) -> Self {
        let contents = fs::read(filename).unwrap();
        let (hbytes, ebytes) = contents.split_at(12);

        let header = Self::read_header(hbytes);

        let entries = Self::read_entries(ebytes, header.num_entries as usize);
        Self { header, entries }
    }

    fn read_header(mut bytes: &[u8]) -> IndexHeader {
        let signature = take_u32(&mut bytes);
        let version = take_u32(&mut bytes);
        let num_entries = take_u32(&mut bytes);

        IndexHeader { signature, version, num_entries }
    }

    fn read_entries(mut bytes: &[u8], num_entries: usize) -> Vec<IndexEntry> {
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

    fn to_bytes(&self) -> Vec<u8> {
        let mut index_bytes: Vec<u8> = Vec::new();

        index_bytes.extend_from_slice(&self.header.signature.to_be_bytes());
        index_bytes.extend_from_slice(&self.header.version.to_be_bytes());
        index_bytes.extend_from_slice(&self.header.num_entries.to_be_bytes());

        for entry in &self.entries {
            index_bytes.extend_from_slice(&entry.ctime_sec.to_be_bytes());
            index_bytes.extend_from_slice(&entry.ctime_nano.to_be_bytes());
            index_bytes.extend_from_slice(&entry.mtime_sec.to_be_bytes());
            index_bytes.extend_from_slice(&entry.mtime_nano.to_be_bytes());
            index_bytes.extend_from_slice(&entry.dev.to_be_bytes());
            index_bytes.extend_from_slice(&entry.ino.to_be_bytes());
            index_bytes.extend_from_slice(&entry.mode.to_be_bytes());
            index_bytes.extend_from_slice(&entry.uid.to_be_bytes());
            index_bytes.extend_from_slice(&entry.gid.to_be_bytes());
            index_bytes.extend_from_slice(&entry.size.to_be_bytes());
            index_bytes.extend_from_slice(&entry.key.0);
            index_bytes.extend_from_slice(&entry.flags.to_be_bytes());
            index_bytes.extend_from_slice(&entry.name.as_bytes());
            let padding_len = 8 - ((6 + entry.name_len()) % 8);
            for _ in 0..padding_len {
                index_bytes.push(0);
            }

        }

        index_bytes
    }


    fn to_tree_bytes(&self) -> Vec<u8> {
        let mut tree: Vec<u8> = Vec::new();

        for entry in &self.entries {
            let mut bytes = format!(
                "{:06} {}\0", 
                100644,
                entry.name,
            ).into_bytes();
            bytes.extend_from_slice(&entry.key.0);
            tree.append(&mut bytes);
        }
        tree
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
        let mode       = (1000 & 0x00F) << 12 | 0o0644 & 0x1FF;
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

    fn object_type(&self) -> u32 {
        // First 4 bits
        (self.mode >> 12) & 0x00F
    }

    fn permission(&self) -> u32 {
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
            println!("{} |", data);
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
        println!("reading entries");
        if let Some(pos) = bytes.iter().position(|&x| x == 0) {
            let (content, rest) = bytes.split_at(pos);
            let data: Vec<&str> = str::from_utf8(content).ok()?
                .split(" ")
                .collect();
            for (i, s) in data.clone().into_iter().enumerate() {
                println!("{} {}", i, s);
            }

            if data.len() != 2 {
                println!("finished reading entries");
                return None;
            }

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

struct Stamp {
    name: String,
    email: String,
    timestamp: u32,
}

impl fmt::Display for Stamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}> {} +0100", self.name, self.email, self.timestamp)
    }
}

struct Commit {
    tree: Hash,
    parent: Option<Hash>,
    author: Stamp,
    committer: Stamp,
    message: String,
}

impl Commit {
    fn create(
        tree: Hash, 
        parent: Option<Hash>, 
        author: Stamp, 
        committer: Stamp, 
        message: String
    ) -> Self {
        Commit {
            tree,
            parent,
            author,
            committer,
            message
        }

    }

    fn read(bytes: &mut &[u8]) -> Option<Commit> {
        Commit::read_header(bytes);
        let commit = str::from_utf8(bytes).ok()?;
        let mut lines = commit.lines().peekable();

        let tree = Commit::read_tree(lines.next()?);
        let parent = if lines.peek()?.starts_with("parent ") {
            Some(Hash::from_hex(&lines.next()?[7..47]))
        } else { None } ;
        let author = Commit::read_author(lines.next()?);
        let committer = Commit::read_committer(lines.next()?);

        lines.next()?; // Empty line
        let message = lines.collect::<Vec<&str>>().join("\n");

        Some(
            Commit {
                tree,
                parent,
                author,
                committer,
                message
            }
        )
    }

    fn read_header(bytes: &mut &[u8]) {
        if let Some(pos) = bytes.iter().position(|&x| x == 0) {
            take_n_bytes(bytes, pos + 1);
        }
    }

    fn read_tree(tree: &str) -> Hash{
        let hash = &tree["tree ".len()..];
        Hash::from_hex(&hash[..40])
    }

    fn read_parent(parent: &str) -> Hash{
        let hash = &parent["parent ".len()..];
        Hash::from_hex(&hash[..40])
    }

    fn read_author(author: &str) -> Stamp {
        Commit::take_stamp(&author["author ".len()..])
    }

    fn read_committer(committer: &str) -> Stamp {
        Commit::take_stamp(&committer["committer ".len()..])
    }

    fn take_stamp(stamp: &str) -> Stamp {
        let (name, stamp) = stamp.split_once(" <").unwrap();
        let (email, stamp) = stamp.split_once("> ").unwrap();
        let (timestamp, _timezone) = stamp.split_once(" ").unwrap();
        Stamp {
            name: name.to_string(),
            email: email.to_string(),
            timestamp: timestamp.parse().unwrap(),

        }
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "tree {}", self.tree)?;
        if let Some(parent) = &self.parent  {
            writeln!(f, "parent {}", parent)?;
        }
        writeln!(f, "author {}", self.author)?;
        writeln!(f, "committer {}", self.committer)?;
        write!(f, "\n{}", self.message)
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
    fn hash_from_hex() {
        let input = "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12";

        let hash = Hash::from_hex(input).to_string();

        let expected = String::from("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
        assert_eq!(hash, expected);
    }

    #[test]
    fn hash_blob_object() {
        let content = String::from("what is up, doc?").into_bytes();

        let hashed = hash_blob(content).to_string();

        let expected = String::from("bd9dbf5aae1a3862dd1526723246b20206e5fc37");
        assert_eq!(hashed, expected);
    }

    #[test]
    fn compress_blob_object() {
        let content = fs::read("playground/main.c").unwrap();
        let header = format!("{} {}\0", ObjectKind::Blob, content.len());
        let compressed = compress_content(header, content).unwrap();

        let expected = fs::read("playground/.git/objects/d2/676eb8d33f7a3c4d3b133f0dad9040b81c5082").unwrap();
        assert_eq!(compressed, expected);
    }

    #[test]
    fn read_header_from_index() {
        let filename = String::from("examples/index");

        let index = Index::read(&filename);
        let bytes: [u8; 4] = index.header.signature.to_be_bytes();
        let actual = str::from_utf8(&bytes).unwrap();

        let expected = "DIRC";
        assert_eq!(actual, expected);
    }

    #[test]
    fn read_entry_hash_from_index() {
        let filename = String::from("examples/index");

        let index = Index::read(&filename);
        let key = index.entries[0].key.to_string();

        let expected = String::from("ea8c4bf7f35f6f77f75d92ad8ce8349f6e81ddba");
        assert_eq!(key, expected);
    }

    #[test]
    fn parse_mode_from_index() {
        let filename = String::from("examples/index");

        let index = Index::read(&filename);
        let object_type = index.entries[0].object_type();
        let permission = index.entries[0].permission();

        let expected_type = 0o10;
        let expected_permission = 0o0644;

        assert_eq!(object_type, expected_type);
        assert_eq!(permission, expected_permission);
    }

    #[test]
    fn list_entry_from_index() {
        let filename = String::from("examples/index");

        let index = Index::read(&filename);
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
        let filename = String::from("examples/tree");
        let content = fs::read(&filename).unwrap();
        let mut decoded = &decompress(content).unwrap()[..];

        let index = Tree::read(&mut decoded);
        let index_entry = index.entries[4].to_string();

        let expected = "040000 tree f37ef49b903a6db9fa814b04f8226569f6d0f592    examples";
        assert_eq!(index_entry, expected);
    }

    #[test]
    fn create_tree_hash_from_index() {
        let filename = String::from("examples/index_with_tree");
        let content = fs::read(&filename).unwrap();
        let mut decoded = &decompress(content).unwrap()[..];

        let tree = Tree::read(&mut decoded).write_tree();
        let key = hash_tree(tree).to_string();

        let expected = String::from("f37ef49b903a6db9fa814b04f8226569f6d0f592");
        assert_eq!(key, expected);
    }

    #[test]
    fn read_commit_object() {
        let filename = String::from("examples/commit");
        let content = fs::read(&filename).unwrap();
        let mut commit_file = &decompress(content).unwrap()[..];
        
        let commit = Commit::read(&mut commit_file).unwrap();
        assert_eq!(commit.author.name, "Nicolay Roness");
        assert_eq!(commit.parent.unwrap().to_string(), "c631313b6cc3a747eac28cdb26802678a96b870b");
        assert_eq!(commit.message, "create blob from file");
    }

    #[test]
    fn create_commit_from_tree() {
        let filename = String::from("examples/tree");
        let content = fs::read(&filename).unwrap();
        let mut decoded = &decompress(content).unwrap()[..];

        let key = Hash::from_bytes(String::from(""), decoded.to_vec());
        let parent = Some(Hash::from_hex("c631313b6cc3a747eac28cdb26802678a96b870b"));
        let author = Stamp {
            name: "Nicolay Roness".to_string(),
            email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
            timestamp: 1762103153 
        };

        let committer = Stamp {
            name: "Nicolay Roness".to_string(),
            email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
            timestamp: 1762103153 
        };
        let message = String::from("create blob from file");
        let commit = Commit::create(key, parent, author, committer, message)
            .to_string();

        let expected =
            "tree fa1c738cb61be8fe31fa4427b7bb7c5b12fe4151
parent c631313b6cc3a747eac28cdb26802678a96b870b
author Nicolay Roness <nicolay.caspersen.roness@sparebank1.no> 1762103153 +0100
committer Nicolay Roness <nicolay.caspersen.roness@sparebank1.no> 1762103153 +0100

create blob from file";

        assert_eq!(commit, expected);
    }
}
