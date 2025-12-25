use crate::hash::*;
use crate::ROOT;
use crate::compress::*;
use crate::util::*;

use std::fs;
use std::io;

use std::str::FromStr;
use std::path::Path;

#[derive(Debug, Copy, Clone)]
pub enum ObjectKind {
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

pub fn hash_object(object_type: ObjectKind, content: Vec<u8>) -> Hash {
    let header = format!("{} {}\0", object_type, content.len());
    Hash::from_bytes(header, content)
}

pub fn hash_blob(content: Vec<u8>) -> Hash {
    hash_object(ObjectKind::Blob, content)
}

pub fn hash_tree(content: Vec<u8>) -> Hash {
    hash_object(ObjectKind::Tree, content)
}

pub fn hash_commit(content: Vec<u8>) -> Hash {
    hash_object(ObjectKind::Commit, content)
}

pub fn write_object(object_type: ObjectKind, content: Vec<u8>) -> Result<Hash, io::Error> {
    let hash = hash_object(object_type, content.clone());

    let path_str = format!("{}/{}", ROOT, hash.to_object_path());
    let path = Path::new(&path_str);
    if path.exists() {
        println!("[INFO] {} {} already exists", object_type, hash);
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

        let hashed = hash_blob(content).to_string();

        let expected = String::from("bd9dbf5aae1a3862dd1526723246b20206e5fc37");
        assert_eq!(hashed, expected);
    }


    #[test]
    fn compress_blob_object() {
        let content = std::fs::read("playground/main.c").unwrap();
        let header = format!("{} {}\0", ObjectKind::Blob, content.len());
        let compressed = compress_content(header, content).unwrap();

        let expected = std::fs::read("playground/.git/objects/d2/676eb8d33f7a3c4d3b133f0dad9040b81c5082").unwrap();
        assert_eq!(compressed, expected);
    }
}
