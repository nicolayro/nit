use crate::util::*;
use crate::hash::*;

use std::os::unix::fs::MetadataExt;
use std::fs;


#[derive(Debug)]
pub struct Index {
    pub header: IndexHeader,
    pub entries: Vec<IndexEntry>
}

#[derive(Debug)]
pub struct IndexHeader {
    /* 
     * 4-byte signature: 
     *  The signature is { 'D', 'I', 'R', 'C' } (stands for "dircache") 
     */
    pub signature: u32,
    /*
     * 4-byte version number:
     *  The current supported versions are 2, 3 and 4.
    */
    pub version: u32,
    /* 32-bit number of index entries */
    pub num_entries: u32
}

impl Index {
    pub fn read(filename: &str) -> Self {
        let contents = fs::read(filename).unwrap();
        let (hbytes, ebytes) = contents.split_at(12);

        let header = Self::read_header(hbytes);

        let entries = Self::read_entries(ebytes, header.num_entries as usize);
        Self { header, entries }
    }

    pub fn read_header(mut bytes: &[u8]) -> IndexHeader {
        let signature = take_u32(&mut bytes);
        let version = take_u32(&mut bytes);
        let num_entries = take_u32(&mut bytes);

        IndexHeader { signature, version, num_entries }
    }

    pub fn read_entries(mut bytes: &[u8], num_entries: usize) -> Vec<IndexEntry> {
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

    pub fn to_bytes(&self) -> Vec<u8> {
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


    pub fn to_tree_bytes(&self) -> Vec<u8> {
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

pub struct IndexEntry {
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
    pub key: Hash,
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
    pub fn create(key: Hash, filename: &str) -> Self{
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
        let name = filename;

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
            name: name.to_string()
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

    pub fn object_type(&self) -> u32 {
        // First 4 bits
        (self.mode >> 12) & 0x00F
    }

    pub fn permission(&self) -> u32 {
        // Final 9 bits
        self.mode & 0x1FF
    }
}

impl std::fmt::Display for IndexEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02o}{:04o} {} {}       {}", 
            self.object_type(),
            self.permission(),
            self.key,
            0,
            self.name)
    }
}

impl std::fmt::Debug for IndexEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

