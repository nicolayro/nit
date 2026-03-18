use crate::hash::*;
use crate::object::*;
use crate::index::*;
use crate::take_hash;

use std::fmt;
use std::io;
use std::str::FromStr;
use std::path::{PathBuf, Component};
use std::collections::HashMap;


#[derive(Debug)]
pub struct TreeCache {
    pub blobs: Vec<TreeEntry>,
    pub trees: HashMap<PathBuf, TreeCache>
}

#[derive(Debug)]
pub struct TreeEntry {
    pub key: Hash,
    pub mode: FileMode,
    pub name: PathBuf,
}
impl TreeEntry {
    pub fn new(key: Hash, mode: FileMode, name: PathBuf) -> Self {
        TreeEntry { key, mode, name }
    }

    pub fn _read(bytes: &mut &[u8]) -> Option<Self> {
        if let Some(pos) = bytes.iter().position(|&x| x == 0) {
            let (content, rest) = bytes.split_at(pos);
            let data: Vec<&str> = str::from_utf8(content).ok()?
                .split(" ")
                .collect();
            for (i, s) in data.clone().into_iter().enumerate() {
                println!("{} {}", i, s);
            }

            let mode: FileMode = FileMode::from_str(data[0]).unwrap();
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
            self.mode.value(),
            self.name.to_string_lossy(),
        ).into_bytes();
        bytes.extend_from_slice(&self.key.0);
        bytes
    }
}

impl fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:06} {} {}    {}",
            self.mode.value(),
            self.mode.object_kind(),
            self.key,
            self.name.to_string_lossy())
    }
}

impl TreeCache {
    pub fn new() -> Self {
        Self {
            blobs: Vec::new(),
            trees: HashMap::new()
        }
    }

    pub fn from_index(index: Index) -> Self {
        let mut cache = TreeCache::new();

        for entry in index.entries {
            let path = PathBuf::from(&entry.name);
            let components: Vec<Component> = path.components().collect();
            if components.len() > 1 {
                let (base, rest) = components.split_first().expect("ERROR: Split first should always work in len > 1");
                let base: PathBuf = base.into();
                let rest: PathBuf = rest.iter().collect();

                let sub_cache = cache.get_or_create_tree_mut(base);

                let mode = entry.file_mode();
                let entry = TreeEntry::new(entry.key, mode, rest);

                sub_cache.add_tree(entry);
            } else {
                let mode = entry.file_mode();
                let blob = TreeEntry::new(entry.key, mode, entry.name.into());
                cache.add_blob(blob);
            }
        }

        cache
    }

    pub fn write(self) -> Result<Hash, io::Error> {
        let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

        for blob in self.blobs {
            let name = blob.name.to_string_lossy().into_owned();
            entries.push((name, blob.as_bytes()));
        }

        for (dir, subtree) in self.trees {
            let hash = subtree.write().unwrap_or_else(|err|
                panic!("[ERROR]: Unable to create tree for '{}': {}", dir.to_string_lossy(), err)
            );
            let entry = TreeEntry::new(hash, FileMode::Tree, dir.clone());
            let sort_key = format!("{}/", dir.to_string_lossy());
            entries.push((sort_key, entry.as_bytes()));
        }

        entries.sort_by_key(|(n, _)| n.clone());
        let tree: Vec<u8> = entries.into_iter().flat_map(|(_, b)| b).collect();
        write_object(ObjectKind::Tree, tree)
    }

    pub fn get_or_create_tree_mut(&mut self, tree_name: PathBuf) -> &mut TreeCache {
        self.trees.entry(tree_name).or_insert(TreeCache::new())
    }

    pub fn add_blob(&mut self, entry: TreeEntry) {
        self.blobs.push(entry);
    }

    pub fn add_tree(&mut self, entry: TreeEntry) {
        let path = PathBuf::from(&entry.name);
        let components: Vec<Component> = path.components().collect();
        if components.len() > 1 {
            let (base, rest) = components.split_first().expect("ERROR: Split first should always work in len > 1");
            let base: PathBuf = base.into();
            let rest: PathBuf = rest.iter().collect();

            let entry = TreeEntry::new(entry.key, entry.mode, rest);
            let sub_cache = self.trees.entry(base).or_insert(TreeCache::new());
            sub_cache.add_tree(entry);
        } else {
            let blob = TreeEntry::new(entry.key, entry.mode, entry.name); // !!!!!!!!!!!!
            self.add_blob(blob);
        }
    }
}

#[cfg(test)]
mod tests {
    struct Tree {
        pub entries: Vec<TreeEntry>
    }

    impl Tree {
        pub fn _new() -> Self {
            let entries = Vec::new();
            Self { entries }
        }

        pub fn _read(bytes: &mut &[u8]) -> Self{
            Tree::_read_header(bytes);

            let mut entries = Vec::new();
            while let Some(entry) = TreeEntry::_read(bytes) {
                entries.push(entry);
            };

            Tree { entries }
        }

        pub fn _read_header(bytes: &mut &[u8]) {
            if let Some(pos) = bytes.iter().position(|&x| x == 0) {
                let (content, rest) = bytes.split_at(pos);
                let data: &str = str::from_utf8(content).unwrap();
                *bytes = &rest[1..];
                println!("{} |", data);
            }
        }

        pub fn _to_bytes(&self) -> Vec<u8> {
            self.entries
                .iter()
                .map(|entry| entry.as_bytes())
                .flatten()
                .collect()
        }
    }

    use super::*;
    use std::fs;
    use crate::compress::*;

    #[test]
    fn parse_entry_hash_from_staging_area() {
        let filename = String::from("examples/tree");
        let content = fs::read(&filename).unwrap();
        let mut decoded = &_decompress(content).unwrap()[..];

        let index = Tree::_read(&mut decoded);
        let index_entry = index.entries[4].to_string();

        let expected = "040000 tree f37ef49b903a6db9fa814b04f8226569f6d0f592    examples";
        assert_eq!(index_entry, expected);
    }

    #[test]
    fn create_tree_hash_from_index() {
        let filename = String::from("examples/index_with_tree");
        let content = fs::read(&filename).unwrap();
        let mut decoded = &_decompress(content).unwrap()[..];

        let tree = Tree::_read(&mut decoded)._to_bytes();
        let key = hash_object(ObjectKind::Tree, tree).to_string();

        let expected = String::from("f37ef49b903a6db9fa814b04f8226569f6d0f592");
        assert_eq!(key, expected);
    }
}
