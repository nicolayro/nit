use std::{fs, io};
use std::fs::File;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use crate::commit::{Commit, Stamp};
use crate::hash::Hash;
use crate::{BRANCH, IGNORE, INDEX_FILE, ROOT};
use crate::index::{Index, IndexEntry};
use crate::object::{write_object, ObjectKind};
use crate::tree::TreeCache;
use crate::util::write_to_file;

pub fn collect_files(path: PathBuf) -> Vec<PathBuf> {
    if path.is_dir() {
        paths_in_dir(path).into_iter().flat_map(collect_files).collect()
    } else {
        vec![path]
    }
}

fn paths_in_dir(path: PathBuf) -> Vec<PathBuf> {
    fs::read_dir(path)
        .expect("Unable to read directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| !IGNORE.iter().any(|i| p.ends_with(i)))
        .collect()
}

pub fn write_objects(files: Vec<PathBuf>) -> Vec<(PathBuf, Hash)> {
    files.into_iter().filter_map(|path| {
        match write_blob(&path) {
            Ok(hash) => Some((path, hash)),
            Err(err) => {
                println!("[ERROR]: Unable to write blob {:?}: {}", path, err);
                None
            }
        }
    }).collect()
}

fn write_blob(file: &PathBuf) -> Result<Hash, io::Error> {
    let content = fs::read(file)?;
    write_object(ObjectKind::Blob, content)
}

pub fn remove_leading_dot_slash(path: PathBuf) -> PathBuf {
    let components: Vec<_> = path.components().collect();

    if let Some(Component::CurDir) = components.first() {
        components.iter().skip(1).collect()
    } else {
        path.to_path_buf()
    }
}

fn blob_to_index_entry((path, hash): (PathBuf, Hash)) -> IndexEntry {
    let path = remove_leading_dot_slash(path);
    IndexEntry::create(hash, &path.to_string_lossy())
}

pub fn update_index(blobs: Vec<(PathBuf, Hash)>) {
    let new_entries = blobs
        .into_iter()
        .map(blob_to_index_entry)
        .collect();

    let index = Index::read(INDEX_FILE).extend(new_entries);
    write_index(index);
}

fn write_index(index: Index) {
    let index_bytes = index.to_bytes();
    let mut index = File::create(String::from(INDEX_FILE)).unwrap();
    index.write_all(&index_bytes).unwrap()
}

pub fn write_cache(cache: TreeCache) -> Result<Hash, io::Error> {
    cache.write()
}

pub fn commit_tree(key: Hash, message: String) -> Result<Hash, io::Error> {
    // create commit
    let parent = get_parent();
    let author = get_author();
    let committer = get_author();
    let commit = Commit::create(key, parent, author, committer, message);

    // write commit
    let commit_content = commit.to_string().into_bytes();
    write_object(ObjectKind::Commit, commit_content)
}

fn get_author() -> Stamp {
    Stamp {
        name: "Nicolay Roness".to_string(),
        email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32,
    }
}

pub fn get_parent() -> Option<Hash> {
    let path = format!("{}/refs/heads/{}", ROOT, BRANCH);
    if Path::new(&path).exists() {
        let parent_hex = fs::read_to_string(path).unwrap();
        Some(Hash::from_hex(&parent_hex[..40]))
    } else {
        println!("[INFO] refs do not exists");
        None
    }
}


pub fn update_refs(commit: Hash) -> Result<(), io::Error> {
    let path = format!("{}/refs/heads/{}", ROOT, BRANCH);
    let content = format!("{}", commit).into_bytes();
    write_to_file(path, content)
}
