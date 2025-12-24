use sha1::{Sha1, Digest};

#[derive(Debug)]
pub struct Hash(pub [u8; 20]);

impl Hash {
    pub fn from_hex(hash: &str) -> Self {
        if hash.len() != 40 {
            panic!("ERROR: Hex encoded hash must be 40 characters, received '{}': {}",
                hash.len(),
                hash
            )
        }
        let bytes = match hex::decode(hash) {
            Ok(b) => b,
            Err(err) => panic!("ERROR: Invalid hash '{}': {}", hash, err)
        };

        Hash(bytes.try_into().unwrap())
    }

    pub fn from_bytes(header: String, content: Vec<u8>) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(header);
        hasher.update(content);
        Hash(hasher.finalize().into())
    }

    pub fn to_object_path(&self) -> String {
        let (l1, l2) = self.0.split_at(1);
        format!("objects/{}/{}", hex::encode(l1), hex::encode(l2))
    }
}
    

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sha1_hash() {
        let input = String::from("The quick brown fox jumps over the lazy dog");

        let hashed = Hash::from_bytes(String::from(""), input.into_bytes()).to_string();

        let expected = String::from("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
        assert_eq!(hashed, expected);
    }

    #[test]
    fn hash_from_hex() {
        let input = "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12";

        let hash = Hash::from_hex(input).to_string();

        let expected = String::from("2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
        assert_eq!(hash, expected);
    }
}
