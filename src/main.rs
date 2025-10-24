use sha1::{Sha1, Digest};
use std::fmt;

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

/*
== Index entry

  Index entries are sorted in ascending order on the name field,
  interpreted as a string of unsigned bytes (i.e. memcmp() order, no
  localization, no special casing of directory separator '/'). Entries
  with the same name are sorted by their stage field.

  32-bit ctime seconds, the last time a file's metadata changed
    this is stat(2) data

  32-bit ctime nanosecond fractions
    this is stat(2) data

  32-bit mtime seconds, the last time a file's data changed
    this is stat(2) data

  32-bit mtime nanosecond fractions
    this is stat(2) data

  32-bit dev
    this is stat(2) data

  32-bit ino
    this is stat(2) data

  32-bit mode, split into (high to low bits)

    4-bit object type
      valid values in binary are 1000 (regular file), 1010 (symbolic link)
      and 1110 (gitlink)

    3-bit unused

    9-bit unix permission. Only 0755 and 0644 are valid for regular files.
    Symbolic links and gitlinks have value 0 in this field.

  32-bit uid
    this is stat(2) data

  32-bit gid
    this is stat(2) data

  32-bit file size
    This is the on-disk size from stat(2), truncated to 32-bit.

  160-bit SHA-1 for the represented object

  A 16-bit 'flags' field split into (high to low bits)

    1-bit assume-valid flag

    1-bit extended flag (must be zero in version 2)

    2-bit stage (during merge)

    12-bit name length if the length is less than 0xFFF; otherwise 0xFFF
    is stored in this field.
*/

struct IndexEntry {
    ctime: u64,
    mtime: u64,
    dev: u32,
    ino: u32,
    mode: u32,
    uid: u32,
    gid: u32,
    size: u32,
    key: Hash,
    flags: u16,
}


impl IndexEntry {
    fn from_blob(key: String, filename: String, content: String) -> Self {
        todo!("Not implemented yet")
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_hash() {
        let hashed = Hash::from(String::from("The quick brown fox jumps over the lazy dog"));
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

}

