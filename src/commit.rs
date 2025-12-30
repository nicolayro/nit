use crate::hash::*;

use std::fmt;

pub struct Stamp {
    pub name: String,
    pub email: String,
    pub timestamp: u32,
}

impl fmt::Display for Stamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}> {} +0100", self.name, self.email, self.timestamp)
    }
}

pub struct Commit {
    tree: Hash,
    parent: Option<Hash>,
    author: Stamp,
    committer: Stamp,
    message: String,
}

impl Commit {
    pub fn create(
        tree: Hash, 
        parent: Option<Hash>, 
        author: Stamp, 
        committer: Stamp, 
        message: String
    ) -> Self {
        Self {
            tree,
            parent,
            author,
            committer,
            message
        }
    }
}

#[cfg(test)]
impl Commit {
    pub fn read(bytes: &mut &[u8]) -> Option<Commit> {
        Commit::read_header(bytes);
        let commit = str::from_utf8(bytes).ok()?;
        let mut lines = commit.lines().peekable();

        let tree = Commit::read_tree(lines.next()?);
        let parent = if lines.peek()?.starts_with("parent ") {
            Some(Hash::from_hex(&lines.next()?[7..47]))
        } else { None } ;
        let author = Commit::read_author(lines.next()?);
        let committer = Commit::read_committer(lines.next()?);

        lines.next()?; // Empty line
        let mut message = lines.collect::<Vec<&str>>().join("\n");
        message.push_str("\n");

        Some(
            Commit {
                tree,
                parent,
                author,
                committer,
                message
            }
        )
    }

    fn read_header(bytes: &mut &[u8]) {
        if let Some(pos) = bytes.iter().position(|&x| x == 0) {
            crate::take_n_bytes(bytes, pos + 1);
        }
    }

    fn read_tree(tree: &str) -> Hash{
        let hash = &tree["tree ".len()..];
        Hash::from_hex(&hash[..40])
    }

    fn read_parent(parent: &str) -> Hash{
        let hash = &parent["parent ".len()..];
        Hash::from_hex(&hash[..40])
    }

    fn read_author(author: &str) -> Stamp {
        Commit::take_stamp(&author["author ".len()..])
    }

    fn read_committer(committer: &str) -> Stamp {
        Commit::take_stamp(&committer["committer ".len()..])
    }

    fn take_stamp(stamp: &str) -> Stamp {
        let (name, stamp) = stamp.split_once(" <").unwrap();
        let (email, stamp) = stamp.split_once("> ").unwrap();
        let (timestamp, _timezone) = stamp.split_once(" ").unwrap();
        Stamp {
            name: name.to_string(),
            email: email.to_string(),
            timestamp: timestamp.parse().unwrap(),

        }
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "tree {}", self.tree)?;
        if let Some(parent) = &self.parent  {
            writeln!(f, "parent {}", parent)?;
        }
        writeln!(f, "author {}", self.author)?;
        writeln!(f, "committer {}", self.committer)?;
        writeln!(f, "\n{}", self.message)
    }
}

#[cfg(test)]
use chrono::DateTime;

#[cfg(test)]
pub fn timestamp_to_date(seconds: u32, nanoseconds: u32) -> String {
    let seconds: i64 = i64::from(seconds);
    let dt = DateTime::from_timestamp(seconds, nanoseconds);
    match dt {
        Some(date) => format!("{}", date),
        None => String::from("")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use crate::compress::*;

    #[test]
    fn read_commit_object() {
        let filename = String::from("examples/commit");
        let content = fs::read(&filename).unwrap();
        let mut commit_file = &_decompress(content).unwrap()[..];

        let commit = Commit::read(&mut commit_file).unwrap();
        assert_eq!(commit.author.name, "Nicolay Roness");
        assert_eq!(commit.parent.unwrap().to_string(), "c631313b6cc3a747eac28cdb26802678a96b870b");
        assert_eq!(commit.message, "create blob from file\n");
    }

    #[test]
    fn create_commit_from_tree() {
        let filename = String::from("examples/commit_tree");
        let content = fs::read(&filename).unwrap();
        let decoded = &_decompress(content).unwrap()[..];

        let key = Hash::from_bytes(String::from(""), decoded.to_vec());
        let parent = Some(Hash::from_hex("f60b322c7351b08514fceed6f69102138ab420e7"));
        let author = Stamp {
            name: "Nicolay Roness".to_string(),
            email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
            timestamp: 1764365370 
        };

        let committer = Stamp {
            name: "Nicolay Roness".to_string(),
            email: "nicolay.caspersen.roness@sparebank1.no".to_string(),
            timestamp: 1764365370 
        };
        let message = String::from("det virker!");
        let commit = Commit::create(key, parent, author, committer, message)
            .to_string();

        let expected =
            "tree b03318345a1f9d098d0bfa44d6111818ab701fbe
parent f60b322c7351b08514fceed6f69102138ab420e7
author Nicolay Roness <nicolay.caspersen.roness@sparebank1.no> 1764365370 +0100
committer Nicolay Roness <nicolay.caspersen.roness@sparebank1.no> 1764365370 +0100

det virker!\n";
        assert_eq!(commit, expected);
    }
}

