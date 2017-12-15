use rustc_serialize::{json};
use sha2::{Sha256, Digest};
use time::{now_utc};


#[derive(RustcEncodable)]
pub struct Block {
    pub index : i64,
    pub timestamp : String,
    pub transactions : Vec<Transaction>,
    pub proof : i64,
    pub hash : Option<String>,
    pub previous_hash : Option<String>,
}

#[derive(RustcEncodable)]
pub struct Transaction {
    pub from : String,
    pub to : String,
    pub amount : f32
}

#[derive(RustcEncodable)]
pub struct Blockchain {
    pub chain : Vec<Block>
}

impl Block {
    fn new() -> Block {
        Block {
            index: 0,
            timestamp: String::new(),
            transactions: Vec::new(),
            proof: 0,
            previous_hash: None,
            hash: None
        }
    }

    fn genesis() -> Block {
        let mut b = Block::new();
        b.set_checksum();
        b
    }

    pub fn set_checksum(&mut self) {
        self.hash = None;
        let payload = json::encode(self).unwrap();

        let mut hasher = Sha256::default();
        hasher.input(payload.as_bytes());

        let hash = hasher.result().iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        self.hash = Some(hash);
    }
}

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            chain : vec!(Block::genesis())
        }
    }

    pub fn new_block(&mut self) -> &mut Block {
        let mut block = Block::new();
        {
            let last = self.chain.last().unwrap();
            block.index = last.index + 1;
            block.previous_hash = last.hash.clone();
            block.timestamp = now_utc().rfc3339().to_string();
        }

        self.chain.push(block);

        return self.chain.last_mut().unwrap();
    }
}
