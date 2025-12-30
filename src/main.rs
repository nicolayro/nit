use std::env;
use std::fs;
use std::io;

use std::io::{Write};
use std::fs::File;
use std::path::{PathBuf, Component};
use std::process::exit;
use std::time::SystemTime;

mod compress;
mod commit;
mod command;
mod object;
mod hash;
mod index;
mod tree;
mod util;

use commit::*;
use command::*;
use hash::*;
use tree::*;
use index::*;
use util::*;
use object::*;

const ROOT: &str   = ".git";
const INDEX_FILE: &str = ".git/index";
const BRANCH: &str = "command";
const IGNORE: [&str; 3] = [".git", "playground", "target"];

fn get_author() -> Stamp {
    Stamp {
        name: "Nicolay Roness".to_string(),
        email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32 - 5 * 86400
    }
}

fn get_parent() -> Hash {
    let path = format!("{}/refs/heads/{}", ROOT, BRANCH);
    let parent_hex = fs::read_to_string(path)
        .expect("ERROR: Unable to read parent hex. Tip: Are you on the correct branch?");
    Hash::from_hex(&parent_hex[..40])
}

fn remove_leading_dot_slash(path: PathBuf) -> PathBuf {
    let components: Vec<_> = path.components().collect();

    if let Some(Component::CurDir) = components.first() {
        components.iter().skip(1).collect()
    } else {
        path.to_path_buf()
    }
}

fn add(path: PathBuf) -> Vec<IndexEntry> {
    let mut entries = Vec::new();
    if path.is_dir() {
        let dir = std::fs::read_dir(path).expect("Unable to read directory");
        for path in dir {
            let path = path.unwrap().path();
            if IGNORE.iter().any(|i| path.ends_with(i)) {
                println!("[INFO] ignoring {}", path.to_string_lossy());
                continue
            }

            let sub_directory = add(path);
            entries.extend(sub_directory);
        }
    } else {
        let hash = write_blob(&path);
        match hash {
            Ok(hash) => {
                let path  = remove_leading_dot_slash(path);
                let filename = path.to_string_lossy();
                let entry = IndexEntry::create(hash, &filename);
                entries.push(entry);
            },
            Err(err) => println!("[ERROR]: Unable to write blob {:?}: {}", path, err)
        }
    }
    entries
}

fn write_blob(file: &PathBuf) -> Result<Hash, io::Error> {
    let content = fs::read(file)?;
    write_object(ObjectKind::Blob, content)
}

fn write_tree(tree: Vec<u8>) -> Result<Hash, io::Error> {
    write_object(ObjectKind::Tree, tree)
}

fn write_commit(commit: Commit) -> Result<Hash, io::Error> {
    let commit_content = format!("{}", commit).into_bytes();
    write_object(ObjectKind::Commit, commit_content)
}

fn write_index(index: Index) -> Result<(), io::Error> {
    let index_bytes = index.to_bytes();
    let mut index = File::create(String::from(INDEX_FILE))?;
    index.write_all(&index_bytes)
}

fn write_cache(cache: TreeCache) -> Result<Hash, io::Error> {
    let mut trees_as_bytes: Vec<(String, Vec<u8>)> = Vec::new();

    for blob in cache.blobs {
        let name = format!("{}", blob.name.to_string_lossy());
        trees_as_bytes.push((name, blob.as_bytes()));
    }

    for (dir, cache) in cache.trees {
        let hash = write_cache(cache).unwrap_or_else(|err|
            panic!("[ERROR]: Unable to create tree for '{}': {}", dir.to_string_lossy(), err)
        );

        let mut bytes = format!(
            "{:06} {}\0",
            ObjectKind::Tree as i32,
            dir.to_string_lossy(),
        ).into_bytes();
        bytes.extend_from_slice(&hash.0);

        let name = format!("{}/", dir.to_string_lossy());
        trees_as_bytes.push((name, bytes));
    }

    trees_as_bytes.sort_by_key(|(n,_)| n.clone());

    let tree: Vec<u8> = trees_as_bytes
        .into_iter()
        .flat_map(|(_, t)| t)
        .collect();

    write_tree(tree)
}

fn commit(key: Hash, message: String) -> Result<Hash, io::Error> {
    // create commit
    let parent = get_parent();
    let author = get_author();
    let committer = get_author();
    let commit = Commit::create(key, Some(parent), author, committer, message);

    // write commit
    write_commit(commit)
}

fn update_refs(commit: Hash) -> Result<(), io::Error> {
    let path = format!("{}/refs/heads/{}", ROOT, BRANCH);
    let content = format!("{}", commit).into_bytes();
    write_to_file(path, content)
}

fn usage() {
    println!("USAGE: nit <command> <args>");
    println!("command:");
    println!("   add     <file|dir>");
    println!("   commit  <message>");
}

fn main() {

    let args: Vec<String> = env::args().collect();
    let command = match Command::parse(args) {
        Ok(command) => {
            println!("[INFO]: Executing command: '{:?}'", command);
            command
        },
        Err(err) => {
            eprintln!("ERROR: {}", err);
            usage();
            exit(1);
        }
    };

    match command {
        Command::Add(path_buf) => {
            /* == Git add == */
            // 1. create objects
            let new_entries = add(path_buf);
            let index = Index::read(INDEX_FILE);
            let updated_index = index.extend(new_entries);

            // 2. write to index
            write_index(updated_index).unwrap();
        },
        Command::Commit(message) => {
            /* == Git commit == */
            // 0. read staging area (index)
            let index = Index::read(INDEX_FILE);

            // 1. write-tree
            let cache = TreeCache::from_index(index);
            let tree_hash = write_cache(cache).unwrap();

            // 2. write to commit
            let commit_hash = commit(tree_hash, message).unwrap();

            // 3. update refs
            update_refs(commit_hash).unwrap();
        }
    };
}

