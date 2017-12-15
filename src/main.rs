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

mod blockchain;

fn main() {
    let mut router = Router::new();
    let blockchain = Arc::new(Mutex::new(blockchain::Blockchain::new()));

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
        router.get("/add_block", move |_: &mut Request| {
            let mut b = blockchain_copy.lock().unwrap();
            let block = b.new_block();
            let payload = json::encode(block).unwrap();
            Ok(Response::with((status::Ok, payload)))
        }, "block");
    }

    Iron::new(router).http("localhost:5000").unwrap();
    println!("On 5000");
}