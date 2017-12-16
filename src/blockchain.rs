use rustc_serialize::json;
use sha2::{Sha256, Digest};
use time;
use std::mem;


#[derive(RustcEncodable)]
pub struct Block {
    pub data: BlockData,
    pub hash: String,
}

#[derive(RustcEncodable)]
pub struct BlockData {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub proof: u64,
    pub previous_hash: Option<String>,
}

#[derive(RustcEncodable,RustcDecodable)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f32,
}

#[derive(RustcEncodable)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    transactions: Vec<Transaction>,
    owner: String,
}

impl Block {
    fn new() -> Block {
        Block {
            data: BlockData {
                index: 0,
                timestamp: 0,
                transactions: Vec::new(),
                proof: 0,
                previous_hash: None,
            },
            hash: String::new(),
        }
    }

    fn genesis() -> Block {
        let mut b = Block::new();
        b.data.proof = 100;
        b
    }

    pub fn set_hash(&mut self) {
        self.hash = self.data.hash();
    }

    pub fn hash_valid(&self) -> bool {
        self.hash == self.data.hash()
    }
}

impl BlockData {
    fn hash(&self) -> String {
        let payload = json::encode(self).unwrap();

        let mut hasher = Sha256::default();
        hasher.input(payload.as_bytes());

        hasher.result().iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

impl Blockchain {
    pub fn new(owner: String) -> Blockchain {
        Blockchain {
            chain: vec!(Block::genesis()),
            transactions: vec!(),
            owner: owner,
        }
    }

    pub fn new_block(&mut self, proof: u64) -> &Block {
        let mut block = Block::new();
        {
            let last = self.chain.last().unwrap();
            block.data.index = last.data.index + 1;
            block.data.previous_hash = Some(last.hash.clone());
        }

        block.data.timestamp = time::get_time().sec;
        block.data.transactions = mem::replace(&mut self.transactions, vec!());
        block.data.proof = proof;
        block.set_hash();

        self.chain.push(block);

        self.chain.last().unwrap()
    }

    fn last_proof(&self) -> u64 {
        let ref block = self.chain.last().unwrap();
        return block.data.proof;
    }

    fn proof_of_work(&self) -> u64 {
        let last_proof = self.last_proof();
        let mut proof = 0u64;

        while !Blockchain::valid_proof(last_proof, proof) {
            proof += 1;
        }

        proof

    }

    fn valid_proof(last_proof: u64, proof: u64) -> bool {
        let mut hasher = Sha256::default();
        hasher.input(last_proof.to_string().as_bytes());
        hasher.input(proof.to_string().as_bytes());

        hasher.result().iter()
            .take(2)
            .all(|x| *x == 0u8)
    }

    pub fn mine_block(&mut self) -> &Block {
        let proof = self.proof_of_work();

        let t = Transaction {
            from: String::new(),
            to: self.owner.clone(),
            amount: 1f32,
        };

        self.new_transaction(t);
        self.new_block(proof)
    }

    pub fn new_transaction(&mut self, t: Transaction) -> &Transaction {
        self.transactions.push(t);
        self.transactions.last().unwrap()
    }
}
