use {
    crate::response::Response,
    std::{
        collections::HashMap,
        io::{prelude::*, BufReader, Result},
        net::TcpStream,
    },
};

pub type HandlerFn = fn(Response);

#[derive(PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    // POST,
    //PUT,
    //DELETE,
}

struct Node {
    nodes: Vec<Node>,
    key: String,
    handler: Option<HandlerFn>,
}

impl Node {
    pub fn new(key: &str) -> Self {
        Self {
            nodes: Vec::new(),
            key: String::from(key),
            handler: None,
        }
    }

    pub fn insert(&mut self, path: &str, f: HandlerFn) {
        match path.split_once('/') {
            Some((root, "")) => {
                self.key = String::from(root);
                self.handler = Some(f);
            }

            Some(("", path)) => self.insert(path, f),

            Some((root, path)) => {
                let node = self.nodes.iter_mut().find(|m| root == &m.key);
                match node {
                    Some(n) => n.insert(path, f),
                    None => {
                        let mut node = Node::new(root);
                        node.insert(path, f);
                        self.nodes.push(node);
                    }
                }
            }

            None => {
                let mut node = Node::new(path);
                node.handler = Some(f);
                self.nodes.push(node);
            }
        }
    }

    pub fn get(&self, path: &str) -> Option<&HandlerFn> {
        match path.split_once('/') {
            Some((root, "")) => {
                if root == &self.key {
                    self.handler.as_ref()
                } else {
                    None
                }
            }

            Some(("", path)) => self.get(path),

            Some((root, path)) => {
                let node = self.nodes.iter().find(|m| root == &m.key);
                if let Some(node) = node {
                    node.get(path)
                } else {
                    None
                }
            }

            None => {
                let node = self.nodes.iter().find(|m| path == &m.key);
                if let Some(node) = node {
                    node.handler.as_ref()
                } else {
                    None
                }
            }
        }
    }
}

pub struct Router {
    routes: HashMap<Method, Node>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn insert(&mut self, method: Method, path: &str, handler: HandlerFn) {
        let node = self.routes.entry(method).or_insert(Node::new("/"));
        node.insert(path, handler);
    }

    pub fn route_client(&self, client: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(&client);
        let buf = reader.fill_buf()?;

        // read a single line (if one exists)
        let mut line = String::new();
        let mut line_reader = BufReader::new(buf);
        let len = line_reader.read_line(&mut line)?;

        // consume bytes read from original reader
        reader.consume(len);
        if len == 0 {
            return self.bad_request(client);
        }

        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() < 2 {
            self.bad_request(client)
        } else {
            match (parts[0], parts[1]) {
                ("GET", path) => self.handle(Method::GET, path, client),
                _ => self.bad_request(client),
            }
        }
    }

    pub fn handle(&self, method: Method, resource: &str, client: TcpStream) -> Result<()> {
        let res = Response::new(client);
        if let Some(node) = self.routes.get(&method) {
            if let Some(handler) = node.get(resource) {
                return Ok(handler(res));
            }
        }

        // default not found
        res.sendfile(404, "static/404.html")
    }

    pub fn bad_request(&self, client: TcpStream) -> Result<()> {
        let res = Response::new(client);
        res.sendfile(404, "static/404.html")
    }
}
