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
            Err(e) => panic!("ERROR: Invalid hash '{}'", hash)
        };

        Hash(bytes.try_into().unwrap())
    }

    pub fn from_string(input: String) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(input);
        Hash(hasher.finalize().into())
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
