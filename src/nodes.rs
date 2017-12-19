use futures::{Future, Stream};
use hyper::{error,Client, Uri};
use tokio_core::reactor::Core;
use std::error::Error;
use rayon::prelude::*;
use blockchain;
use std::collections::HashSet;
use serde_json;


#[derive(Debug)]
pub struct Nodes {
    nodes: HashSet<Uri>,
}

impl Nodes {
    pub fn new() -> Nodes {
        Nodes {
            nodes: HashSet::new(),
        }
    }

    pub fn nodes(&self) -> Vec<String> {
        self.nodes.iter().map(|x| x.to_string()).collect()
    }

    pub fn add_node(&mut self, uri: String) -> Result<(), error::UriError> {
        let parsed = uri.parse()?;
        self.nodes.insert(parsed);

        Ok(())
    }

    fn query_node(uri: &Uri) -> Result<Vec<blockchain::Block>, Box<Error>> {
        let mut core = Core::new()?;
        let client = Client::new(&core.handle());

        let request = client.get(uri.clone());
        let result = core.run(request)?;

        let body = result.body().concat2().wait()?;
        let decoded = String::from_utf8(body.to_vec())?;
        let chain : Vec<blockchain::Block> = serde_json::from_str(decoded.as_str())?;

        Ok(chain)
    }

    pub fn resolve<F>(&self, f: F)
        where F: Fn(Vec<blockchain::Block>) + Sync + Send
    {
        self.nodes.par_iter().for_each(
            |u| {
                let chain = Nodes::query_node(u).unwrap_or(Vec::new());
                f(chain);
            }
        )
    }
}