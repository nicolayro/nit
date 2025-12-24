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
