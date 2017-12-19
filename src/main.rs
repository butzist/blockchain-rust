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

use std::sync::{Mutex,Arc};
use logger::Logger;
use iron::prelude::*;
use iron::middleware::*;
use iron::headers::*;
use iron::mime::*;
use iron::status;
use router::Router;

mod blockchain;
mod nodes;

quick_error! {
    #[derive(Debug)]
    pub enum APIError {
        EmptyBody {
            description("request has empty body")
        }
    }
}

struct RestAPI;

impl BeforeMiddleware for RestAPI {
    fn before(&self, _: &mut Request) -> IronResult<()> {
        Ok(())
    }
}

impl AfterMiddleware for RestAPI {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        if res.status.unwrap() == status::Ok {
            res.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, Vec::new())));
        }

        Ok(res)
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
            let payload = itry!(serde_json::to_string(&b.chain()));

            Ok(Response::with((status::Ok, payload)))
        }, "chain");
    }

    {
        let blockchain_copy = blockchain.clone();
        router.get("/mine", move |_: &mut Request| {
            let mut b = blockchain_copy.lock().unwrap();
            let block = b.mine_block();

            let payload = itry!(serde_json::to_string(block));
            Ok(Response::with((status::Ok, payload)))
        }, "mine");
    }

    {
        let blockchain_copy = blockchain.clone();
        router.post("/transactions/new", move |r: &mut Request| {
            let mut b = blockchain_copy.lock().unwrap();

            let body = itry!(r.get::<bodyparser::Struct<blockchain::Transaction>>());
            let t = itry!(body.ok_or(APIError::EmptyBody));

            b.new_transaction(t);

            Ok(Response::with((status::Ok, "{}")))
        }, "transaction_new");
    }

    {
        let nodes_copy = nodes.clone();
        router.get("/nodes", move |_: &mut Request| {
            let n = nodes_copy.lock().unwrap();

            let payload = itry!(serde_json::to_string(&n.nodes()));
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

            Ok(Response::with((status::Ok, "{}")))
        }, "nodes_resolve");
    }

    {
        let nodes_copy = nodes.clone();
        router.post("/nodes/add", move |r: &mut Request| {
            let mut n = nodes_copy.lock().unwrap();

            let body = itry!(r.get::<bodyparser::Struct<String>>());
            let uri = itry!(body.ok_or(APIError::EmptyBody));

            itry!(n.add_node(uri));
            Ok(Response::with((status::Ok, "{}")))
        }, "nodes_add");
    }

    let mut chain = Chain::new(router);
    chain.link_before(RestAPI);
    chain.link_after(RestAPI);

    let (logger_before, logger_after) = Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    Iron::new(chain).http("localhost:5000").unwrap();
    println!("On 5000");
}