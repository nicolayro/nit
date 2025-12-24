use std::fs;
use std::io;

use std::io::{Write};
use std::path::Path;
use std::fs::File;
use std::time::SystemTime;

mod compress;
mod commit;
mod object;
mod hash;
mod index;
mod tree;
mod util;

use compress::*;
use commit::*;
use hash::*;
// use tree::*;
use index::*;
use util::*;
use object::*;

const ROOT: &str   = ".git";
const BRANCH: &str = "refs";

fn add(file: &str) -> Result<Hash, io::Error> {
    //  hash-object
    let content = fs::read(file)?;
    let blob = hash_blob(content.clone());

    //  -w (store the object)
    let path_str = format!("{}/{}", ROOT, blob.to_object_path());
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let header = format!("{} {}\0", ObjectKind::Blob, content.len());
    let compressed = compress_content(header, content)?;
    fs::write(path, compressed).unwrap();

    Ok(blob)
}

fn write_index(entries: Vec<IndexEntry>) -> Result<(), io::Error> {
    let index_header = IndexHeader {
        signature: u32::from_be_bytes([ b'D', b'I', b'R', b'C' ]),
        version: 2 as u32,
        num_entries: entries.len() as u32,
    };

    let index = Index {
        header: index_header,
        entries: entries
    };

    let index_bytes = index.to_bytes();
    let index_path = format!("{}/index", ROOT);
    let mut index = File::create(&String::from(index_path))?;
    index.write_all(&index_bytes)?;
    Ok(())
}

fn write_tree() -> Result<Hash, io::Error> {
    let filename = format!("{}/index", ROOT);

    let index = Index::read(&filename);
    let tree = index.to_tree_bytes();

    let key = hash_tree(tree.clone());

    let path_str = format!("{}/{}", ROOT, key.to_object_path());
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let header = format!("{} {}\0", ObjectKind::Tree, tree.len());
    let compressed = compress_content(header, tree)?;
    fs::write(path, compressed)?;

    Ok(key)
}

fn commit(tree: Vec<u8>) -> Result<Commit, io::Error> {
    let key = Hash::from_bytes(String::from(""), tree);
    let path = format!("{}/refs/heads/{}", ROOT, BRANCH);
    let parent_hex = fs::read_to_string(path)?;
    let parent = Hash::from_hex(&parent_hex[..40]);
    let author = Stamp {
        name: "Nicolay Roness".to_string(),
        email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32 - 5 * 86400
    };

    let committer = Stamp {
        name: "Nicolay Roness".to_string(),
        email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
        timestamp: 1762103153 
    };
    let message = String::from("Make a commit");
    let commit = Commit::create(key, Some(parent), author, committer, message);

    Ok(commit)
}

fn read_tree(tree_hash: Hash) -> Result<Vec<u8>, io::Error> {
    let path_str = format!("{}/{}", ROOT, tree_hash.to_object_path());
    let path = Path::new(&path_str);
    let content = fs::read(&path).unwrap();
    let decoded = &decompress(content).unwrap()[..];

    Ok(decoded.to_vec())
}

fn store_commit(commit: Commit) -> Result<Hash, io::Error> {
    let commit_content = format!("{}", commit).into_bytes();
    let hash = hash_commit(commit_content.clone());
    let path_str = format!("{}/{}", ROOT, hash.to_object_path());
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let header = format!("{} {}\0", ObjectKind::Commit, commit_content.len());
    let compressed = compress_content(header, commit_content)?;
    println!("Writing commit to {}", hash.to_object_path());
    fs::write(path, compressed).unwrap();

    Ok(hash)
}

fn update_refs(commit: Hash) -> Result<(), io::Error> {
    let path_str = format!("{}/refs/heads/{}", ROOT, BRANCH);
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}", commit))?;
    Ok(())
}

fn main() {
    let dir = ["main.c", "main.h"];

    // Create objects and write index
    let mut entries: Vec<IndexEntry> = Vec::new();
    for file in dir {
        let hash = add(file);
        match hash {
            Ok(hash) => {
                let entry = IndexEntry::create(hash, file);
                entries.push(entry);
            },
            Err(err) => println!("Error adding {}: {}", file, err)
        }
    }
    write_index(entries).unwrap();

    //  write-tree
    let tree_hash = write_tree().unwrap();

    // Git commit
    let tree = read_tree(tree_hash).unwrap();
    let commit = commit(tree).unwrap();
    println!("{}", commit);

    // Store commit
    let commit_hash = store_commit(commit).unwrap();

    // Update refs
    update_refs(commit_hash).unwrap();
}
