extern crate iron;
extern crate router;
extern crate rustc_serialize;
extern crate sha2;
extern crate time;

use std::sync::{Mutex,Arc};
use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;
use std::io::Read;

mod blockchain;

fn main() {
    let mut router = Router::new();
    let owner = String::from("adam@szalkowski.de");
    let blockchain = Arc::new(Mutex::new(blockchain::Blockchain::new(owner)));

    {
        let blockchain_copy = blockchain.clone();
        router.get("/chain", move |_: &mut Request| {
            let b = blockchain_copy.lock().unwrap();
            let payload = json::encode(&b.chain).unwrap();
            Ok(Response::with((status::Ok, payload)))
        }, "chain");
    }

    {
        let blockchain_copy = blockchain.clone();
        router.get("/mine", move |_: &mut Request| {
            let mut b = blockchain_copy.lock().unwrap();
            let block = b.mine_block();
            let payload = json::encode(block).unwrap();
            Ok(Response::with((status::Ok, payload)))
        }, "mine");
    }

    {
        let blockchain_copy = blockchain.clone();
        router.get("/transactions/new", move |r: &mut Request| {
            let mut b = blockchain_copy.lock().unwrap();

            let mut payload = String::new();
            r.body.read_to_string(&mut payload).unwrap();

            let t : blockchain::Transaction = json::decode(&payload).unwrap();

            b.new_transaction(t);
            Ok(Response::with((status::Ok, "{}".to_string())))
        }, "transaction_new");
    }

    Iron::new(router).http("localhost:5000").unwrap();
    println!("On 5000");
}