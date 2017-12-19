use serde_json;
use sha2::{Sha256, Digest};
use time;
use std::mem;


#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub struct Block {
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub proof: u64,
    pub previous_hash: Option<String>,
}

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq)]
pub struct Transaction {
    pub from: Option<String>,
    pub to: String,
    pub amount: f32,
}

#[derive(Debug)]
pub struct Blockchain {
    chain: Vec<Block>,
    transactions: Vec<Transaction>,
    owner: String,
}

impl Block {
    fn new() -> Block {
        Block {
            timestamp: 0,
            transactions: Vec::new(),
            proof: 0,
            previous_hash: None,
        }
    }

    fn genesis() -> Block {
        let mut b = Block::new();
        b.proof = 100;
        b
    }

    fn hash(&self) -> String {
        let payload = serde_json::to_string(self).unwrap();

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

    pub fn chain(&self) -> &Vec<Block> {
        &self.chain
    }

    pub fn new_block(&mut self, proof: u64) -> &Block {
        let mut block = Block::new();
        {
            let last = self.chain.last().unwrap();
            block.previous_hash = Some(last.hash());
        }

        block.timestamp = time::get_time().sec;
        block.transactions = mem::replace(&mut self.transactions, vec!());
        block.proof = proof;

        self.chain.push(block);

        self.chain.last().unwrap()
    }

    fn last_proof(&self) -> u64 {
        let block = self.chain.last().unwrap();
        return block.proof;
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
            from: None,
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

    fn valid_chain(chain: &Vec<Block>) -> bool {
        if chain.first().unwrap() != &Block::genesis() {
            return false;
        }

        chain.iter().zip(chain.iter().skip(1))
            .all(|(previous_block, block)| {
                block.previous_hash == Some(previous_block.hash())
                    && Blockchain::valid_proof(previous_block.proof, block.proof)
        })
    }

    pub fn try_update(&mut self, chain: Vec<Block>) -> Result<&Vec<Block>, ()> {
        if Blockchain::valid_chain(&chain) && chain.len() > self.chain.len() {
            self.chain = chain;
            Ok(&self.chain)
        } else {
            Err(())
        }
    }
}
