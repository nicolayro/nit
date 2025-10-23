use sha1::{Sha1, Digest};

fn main() {
    let _content = "test content";
    let _filename = "filename.txt";
}

fn hash(input: String) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input);
    let hashed = hasher.finalize();

    hex::encode(hashed)
}


fn hash_blob(content: String) -> String {
    let blob = format!("blob {}\0{}", content.len(), content);
    hash(blob)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_hash() {
        let hashed = hash(String::from("The quick brown fox jumps over the lazy dog"));
        let expected = String::from("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
        assert_eq!(hashed, expected);
    }

    #[test]
    fn hash_object_blob() {
        let content = String::from("what is up, doc?");
        let hashed = hash_blob(content);
        let expected = String::from("bd9dbf5aae1a3862dd1526723246b20206e5fc37");

        assert_eq!(hashed, expected);
    }


    #[test]
    fn hash_object_blob() {
        let content = String::from("what is up, doc?");
        let hashed = hash_blob(content);
        let expected = String::from("bd9dbf5aae1a3862dd1526723246b20206e5fc37");

        assert_eq!(hashed, expected);
    }
}
