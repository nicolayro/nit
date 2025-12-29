use std::fs;
use std::io;

use std::io::{Write};
use std::path::{Path, PathBuf, Component};
use std::fs::File;
use std::time::SystemTime;
use std::collections::HashMap;


mod compress;
mod commit;
mod directory;
mod object;
mod hash;
mod index;
mod tree;
mod util;

use compress::*;
use commit::*;
use directory::*;
use hash::*;
use tree::*;
use index::*;
use util::*;
use object::*;

const ROOT: &str   = ".git";
const BRANCH: &str = "refs";

fn remove_leading_dot_slash(path: PathBuf) -> PathBuf {
    let components: Vec<_> = path.components().collect();

    if let Some(Component::CurDir) = components.first() {
        components.iter().skip(1).collect()
    } else {
        path.to_path_buf()
    }
}


fn add(path: PathBuf) -> Vec<IndexEntry> {
    let dir = std::fs::read_dir(path).expect("Unable to read directory");
    let mut entries = Vec::new();
    for path in dir {
        let path = path.unwrap().path();
        if IGNORE.iter().any(|i| path.ends_with(i)) {
            continue
        }

        if path.is_dir() {
            let sub_directory = add(path);
            entries.extend(sub_directory);
        } else {
            let hash = add_file(&path);
            match hash {
                Ok(hash) => {
                    let path  = remove_leading_dot_slash(path);
                    let filename = path.to_str().unwrap();
                    let entry = IndexEntry::create(hash, filename);
                    entries.push(entry);
                },
                Err(err) => println!("Error adding {:?}: {}", path, err)
            }
        }
    }
    entries
}

fn add_file(file: &PathBuf) -> Result<Hash, io::Error> {
    //  hash-object
    let content = fs::read(file)?;
    let blob = hash_blob(content.clone());

    //  -w (store the object)
    let path_str = format!("{}/{}", ROOT, blob.to_object_path());
    let path = Path::new(&path_str);

    if path.exists() {
        println!("{:?} already exists, skipping write", path);
        return Ok(blob)
    }

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

fn write_blob(entry: TreeEntry) {
    let mut bytes = format!(
        "{:06} {}\0", 
        ObjectKind::Blob,
        entry.name.to_string_lossy(),
    ).into_bytes();
    bytes.extend(&entry.key.0)
}  

fn create_tree(entries: Vec<IndexEntry>) -> Result<Hash, io::Error> {
    let mut cache = Cache {
        blobs: Vec::new(),
        trees: HashMap::new()
    };

    // src/main.rs
    // src/mod.rs
    // cargo.toml

    for entry in entries {
        let path = PathBuf::from(&entry.name);
        let components: Vec<Component> = path.components().collect();
        if components.len() > 1 {
            let (base, rest) = components.split_first().expect("ERROR: Split first should always work in len > 1");
            let base: PathBuf = base.into();
            let rest: PathBuf = rest.iter().collect();

            let entry = TreeEntry::new(entry.key, ObjectKind::Blob, rest);
            let mut sub_cache = cache.trees.entry(base).or_insert(Cache::new());
            sub_cache.update(entry); 
        } else {
            let blob = TreeEntry::new(entry.key, ObjectKind::Blob, entry.name.into());
            cache.blobs.push(blob);
        }
    }

    dbg!(cache);
    todo!()
}

fn write_tree(tree: Vec<u8>) -> Result<Hash, io::Error> {
    // let tree = index.to_tree_bytes();

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

    // Create objects and write index
    let entries = add(PathBuf::from("."));

    // Write index
    write_index(entries).unwrap();
        

    let index_file = format!("{}/index", ROOT);
    let index = Index::read(&index_file);

    //  write-tree
    let tree = create_tree(index.entries).unwrap();

    // Git commit
    let tree = read_tree(tree).unwrap();
    let commit = commit(tree).unwrap();
    println!("{}", commit);

    // Store commit
    let commit_hash = store_commit(commit).unwrap();

    // Update refs
    update_refs(commit_hash).unwrap();

}
