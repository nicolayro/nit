use crate::hash::*;

use std::str::FromStr;

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
    match object_type {
        ObjectKind::Blob => hash_blob(content),
        ObjectKind::Tree => hash_tree(content),
        ObjectKind::Commit => hash_commit(content)
    }
}

pub fn hash_blob(content: Vec<u8>) -> Hash {
    let header = format!("{} {}\0", ObjectKind::Blob, content.len());
    Hash::from_bytes(header, content)
}

pub fn hash_tree(content: Vec<u8>) -> Hash {
    let header = format!("{} {}\0", ObjectKind::Tree, content.len());
    Hash::from_bytes(header, content)
}

pub fn hash_commit(content: Vec<u8>) -> Hash {
    let header = format!("{} {}\0", ObjectKind::Commit, content.len());
    Hash::from_bytes(header, content)
}
