extern crate futures;
extern crate hyper;
#[macro_use] extern crate iron;
#[macro_use] extern crate quick_error;
extern crate bodyparser;
extern crate logger;
extern crate sha2;
extern crate rayon;
extern crate router;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate time;
extern crate tokio_core;

use std::sync::Arc;
use logger::Logger;
use iron::prelude::*;
use router::Router;

mod blockchain;
mod nodes;
mod api;


fn main() {
    let mut router = Router::new();
    let owner = String::from("adam@szalkowski.de");
    let blockchain = blockchain::Blockchain::new(owner);
    let nodes = nodes::Nodes::new();

    let api = Arc::new(api::BlockchainAPI::new(blockchain, nodes));

    {
        let api_copy = api.clone();
        router.get("/chain", move |r: &mut Request| api_copy.handle_chain(r), "chain");
    }

    {
        let api_copy = api.clone();
        router.get("/mine", move |r: &mut Request| api_copy.handle_mine(r), "mine");
    }

    {
        let api_copy = api.clone();
        router.post("/transactions/new", move |r: &mut Request| api_copy.handle_transactions_new(r), "transactions_new");
    }

    {
        let api_copy = api.clone();
        router.get("/nodes", move |r: &mut Request| api_copy.handle_nodes(r), "nodes");
    }

    {
        let api_copy = api.clone();
        router.get("/nodes/resolve", move |r: &mut Request| api_copy.handle_nodes_resolve(r), "nodes_resolve");
    }

    {
        let api_copy = api.clone();
        router.get("/nodes/add", move |r: &mut Request| api_copy.handle_nodes_add(r), "nodes_add");
    }

    let mut chain = Chain::new(router);
    chain.link_before(api.clone());
    chain.link_after(api.clone());

    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    Iron::new(chain).http("localhost:5000").unwrap();
    println!("On 5000");
}