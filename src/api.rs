use std::sync::Mutex;
use blockchain;
use bodyparser;
use nodes;
use iron::status;
use iron::headers::*;
use iron::middleware::*;
use iron::mime::*;
use iron::prelude::*;
use serde_json;

quick_error! {
        #[derive(Debug)]
        pub enum APIError {
            EmptyBody {
                description("request has empty body")
            }
        }
    }

pub struct BlockchainAPI {
    blockchain: Mutex<blockchain::Blockchain>,
    nodes: Mutex<nodes::Nodes>,
}

impl BeforeMiddleware for BlockchainAPI {
    fn before(&self, _: &mut Request) -> IronResult<()> {
        Ok(())
    }
}

impl AfterMiddleware for BlockchainAPI {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        if res.status.unwrap() == status::Ok {
            res.headers.set(ContentType(Mime(TopLevel::Application, SubLevel::Json, Vec::new())));
        }

        Ok(res)
    }
}

impl BlockchainAPI {
    pub fn new(blockchain: blockchain::Blockchain, nodes: nodes::Nodes) -> BlockchainAPI {
        BlockchainAPI {
            blockchain: Mutex::new(blockchain),
            nodes: Mutex::new(nodes),
        }
    }

    pub fn handle_chain(&self, _: &mut Request) -> IronResult<Response> {
        let b = self.blockchain.lock().unwrap();
        let payload = itry!(serde_json::to_string(&b.chain()));

        Ok(Response::with((status::Ok, payload)))
    }

    pub fn handle_mine(&self, _: &mut Request) -> IronResult<Response> {
        let mut b = self.blockchain.lock().unwrap();
        let block = b.mine_block();

        let payload = itry!(serde_json::to_string(block));
        Ok(Response::with((status::Ok, payload)))
    }

    pub fn handle_transactions_new(&self, r: &mut Request) -> IronResult<Response> {
        let mut b = self.blockchain.lock().unwrap();

        let body = itry!(r.get::<bodyparser::Struct<blockchain::Transaction>>());
        let t = itry!(body.ok_or(APIError::EmptyBody));

        b.new_transaction(t);

        Ok(Response::with((status::Ok, "{}")))
    }

    pub fn handle_nodes(&self, _: &mut Request) -> IronResult<Response> {
        let n = self.nodes.lock().unwrap();

        let payload = itry!(serde_json::to_string(&n.nodes()));
        Ok(Response::with((status::Ok, payload)))
    }

    pub fn handle_nodes_resolve(&self, _: &mut Request) -> IronResult<Response> {
        let n = self.nodes.lock().unwrap();

        n.resolve(|chain| {
            let mut b = self.blockchain.lock().unwrap();
            b.try_update(chain).unwrap_or(&Vec::new());
        });

        Ok(Response::with((status::Ok, "{}")))
    }

    pub fn handle_nodes_add(&self, r: &mut Request) -> IronResult<Response> {
        let mut n = self.nodes.lock().unwrap();

        let body = itry!(r.get::<bodyparser::Struct<String>>());
        let uri = itry!(body.ok_or(APIError::EmptyBody));

        itry!(n.add_node(uri));
        Ok(Response::with((status::Ok, "{}")))
    }
}