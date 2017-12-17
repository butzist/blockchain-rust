extern crate futures;
extern crate hyper;
extern crate iron;
extern crate sha2;
extern crate rayon;
extern crate router;
extern crate rustc_serialize;
extern crate time;
extern crate tokio_core;

use std::sync::{Mutex,Arc};
use iron::prelude::*;
use iron::status;
use router::Router;
use rustc_serialize::json;
use std::io::Read;

mod blockchain;
mod nodes;


macro_rules! try_iron {
    ($x:expr) => {
        match $x {
            Ok(v) => v,
            Err(e) => {
                println!("{}", e);
                let err = IronError::new(e, (status::InternalServerError, "Internal server error".to_string()));
                return Err(err);
            }
        }
    }
}

fn main() {
    let mut router = Router::new();
    let owner = String::from("adam@szalkowski.de");
    let blockchain = Arc::new(Mutex::new(blockchain::Blockchain::new(owner)));
    let nodes = Arc::new(Mutex::new(nodes::Nodes::new()));

    {
        let blockchain_copy = blockchain.clone();
        router.get("/chain", move |_: &mut Request| {
            let b = blockchain_copy.lock().unwrap();
            let payload = try_iron!(json::encode(&b.chain()));

            Ok(Response::with((status::Ok, payload)))
        }, "chain");
    }

    {
        let blockchain_copy = blockchain.clone();
        router.get("/mine", move |_: &mut Request| {
            let mut b = blockchain_copy.lock().unwrap();
            let block = b.mine_block();

            let payload = try_iron!(json::encode(block));
            Ok(Response::with((status::Ok, payload)))
        }, "mine");
    }

    {
        let blockchain_copy = blockchain.clone();
        router.get("/transactions/new", move |r: &mut Request| {
            let mut b = blockchain_copy.lock().unwrap();

            let mut payload = String::new();
            try_iron!(r.body.read_to_string(&mut payload));
            let t : blockchain::Transaction = try_iron!(json::decode(&payload));

            b.new_transaction(t);
            Ok(Response::with((status::Ok, "{}".to_string())))
        }, "transaction_new");
    }

    {
        let nodes_copy = nodes.clone();
        router.get("/nodes", move |_: &mut Request| {
            let n = nodes_copy.lock().unwrap();

            let payload = try_iron!(json::encode(&n.nodes()));
            Ok(Response::with((status::Ok, payload)))
        }, "nodes");
    }

    {
        let blockchain_copy = blockchain.clone();
        let nodes_copy = nodes.clone();
        router.get("/nodes/resolve", move |_: &mut Request| {
            let n = nodes_copy.lock().unwrap();

            n.resolve(|chain| {
                let mut b = blockchain_copy.lock().unwrap();
                b.try_update(chain).unwrap_or(&Vec::new());
            });

            let payload = "{}";
            Ok(Response::with((status::Ok, payload)))
        }, "nodes_resolve");
    }


    {
        let nodes_copy = nodes.clone();
        router.get("/nodes/add", move |r: &mut Request| {
            let mut n = nodes_copy.lock().unwrap();

            let mut payload = String::new();
            try_iron!(r.body.read_to_string(&mut payload));
            let uri : String = try_iron!(json::decode(&payload));

            try_iron!(n.add_node(uri));
            Ok(Response::with((status::Ok, "{}".to_string())))

        }, "nodes_add");
    }

    Iron::new(router).http("localhost:5000").unwrap();
    println!("On 5000");
}