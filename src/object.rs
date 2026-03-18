use crate::hash::*;
use crate::ROOT;
use crate::compress::*;

use std::fs;
use std::io;

use std::str::FromStr;
use std::path::Path;

#[derive(Debug, Copy, Clone)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
}

impl std::fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Blob   => write!(f, "blob"),
            ObjectKind::Tree   => write!(f, "tree"),
            ObjectKind::Commit => write!(f, "commit"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FileMode {
    Blob,
    BlobExe,
    BlobSym,
    Tree,
}

impl FileMode {
    pub fn value(&self) -> i32 {
        match self {
            FileMode::Blob    => 100644,
            FileMode::BlobExe => 100755,
            FileMode::BlobSym => 120000,
            FileMode::Tree    => 040000,
        }
    }

    pub fn object_kind(&self) -> ObjectKind {
        match self {
            FileMode::Tree => ObjectKind::Tree,
            _              => ObjectKind::Blob,
        }
    }
}

impl FromStr for FileMode {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "100644"           => Ok(FileMode::Blob),
            "100755"           => Ok(FileMode::BlobExe),
            "120000"           => Ok(FileMode::BlobSym),
            "40000" | "040000" => Ok(FileMode::Tree),
            _ => panic!("[ERROR]: Invalid file mode: {}", input)
        }
    }
}

pub fn hash_object(object_type: ObjectKind, content: Vec<u8>) -> Hash {
    let header = format!("{} {}\0", object_type, content.len());
    Hash::from_bytes(header, content)
}

pub fn write_object(object_type: ObjectKind, content: Vec<u8>) -> Result<Hash, io::Error> {
    let hash = hash_object(object_type, content.clone());

    let path_str = format!("{}/{}", ROOT, hash.to_object_path());
    let path = Path::new(&path_str);
    if path.exists() {
        return Ok(hash)
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let header = format!("{} {}\0", object_type, content.len());
    let compressed = compress_content(header, content)?;
    fs::write(path, compressed)?;

    println!("[INFO] {} {} created", object_type, hash);

    Ok(hash)
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::compress::compress_content;

    #[test]
    fn hash_blob_object() {
        let content = String::from("what is up, doc?").into_bytes();

        let hashed = hash_object(ObjectKind::Blob, content).to_string();

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
}
