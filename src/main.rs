use std::env;

use std::path::PathBuf;
use std::process::exit;

mod command;
mod commit;
mod compress;
mod hash;
mod index;
mod object;
mod plumbing;
mod tree;
mod util;

use command::*;
use index::*;
use plumbing::*;
use tree::*;
use util::*;

const ROOT: &str = ".git";
const INDEX_FILE: &str = ".git/index";
const BRANCH: &str = "main";
const IGNORE: [&str; 3] = [".git", "playground", "target"];





fn add(path: PathBuf) {
    // nit add <path>
    //            ^
    let files = collect_files(path);

    // git hash-object -w
    let objects = write_objects(files);

    // git update-index
    update_index(objects);
}





fn commit(message: String) {
    // 0. read staging area (index)
    let index = Index::read(INDEX_FILE);

    // 1. write-tree
    let cache = TreeCache::from_index(index);
    let tree_hash = write_cache(cache).unwrap();

    // 2. write-commit
    let commit_hash = commit_tree(tree_hash, message).unwrap();

    // 3. update refs
    update_refs(commit_hash).unwrap();
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
        }
        Err(err) => {
            eprintln!("ERROR: {}", err);
            usage();
            exit(1);
        }
    };

    match command {
        Command::Add(path) => add(path),
        Command::Commit(message) => commit(message),
    };
}



