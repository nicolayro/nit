use crate::hash::*;
use crate::object::*;
use crate::take_hash;
use std::fmt;

use std::str::FromStr;

pub struct Tree {
    pub entries: Vec<TreeEntry>
}

pub struct TreeEntry {
    pub key: Hash,
    pub mode: ObjectKind,
    pub name: String,
}

impl Tree {
    pub fn read(bytes: &mut &[u8]) -> Self{
        Tree::read_header(bytes);

        let mut entries = Vec::new();
        while let Some(entry) = TreeEntry::read(bytes) {
            entries.push(entry);
        };

        Tree { entries }
    }

    pub fn read_header(bytes: &mut &[u8]) {
        if let Some(pos) = bytes.iter().position(|&x| x == 0) {
            let (content, rest) = bytes.split_at(pos);
            let data: &str = str::from_utf8(content).unwrap();
            *bytes = &rest[1..];
            println!("{} |", data);
        }
    }

    pub fn write_tree(&self) -> Vec<u8> {
        self.entries
            .iter()
            .map(|entry| entry.as_bytes())
            .flatten()
            .collect()
    }
}

impl TreeEntry {
    pub fn read(bytes: &mut &[u8]) -> Option<Self> {
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

    pub fn as_bytes(&self) -> Vec<u8> {
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
    use std::fs;
    use crate::compress::*;

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
}
