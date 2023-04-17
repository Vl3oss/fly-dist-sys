use crate::messages::{self, Message};
use std::collections::HashMap;
use std::io::stdin;
use std::ops::Deref;

pub type NodeId = String;

#[derive(Debug, PartialEq)]
enum NodeState {
    Uninitialized,
    Initialized,
}

pub struct Node<S = ()> {
    pub node_id: Option<NodeId>,
    pub node_ids: Vec<NodeId>,
    handlers: HashMap<String, Box<dyn Fn(&Node<S>, &String) -> Option<String>>>,
    node_state: NodeState,
    pub state: Option<S>,
}

impl<S> Node<S> {
    pub fn new() -> Self {
        Node {
            node_id: None,
            node_ids: vec![],
            handlers: HashMap::new(),
            node_state: NodeState::Uninitialized,
            state: None,
        }
    }

    fn read(self: &Self) -> String {
        let mut msg_raw = String::new();
        let _ = stdin().read_line(&mut msg_raw);

        msg_raw
    }

    pub fn initialize(self: &mut Self, node_id: NodeId, node_ids: Vec<NodeId>) -> () {
        self.node_id = Some(node_id);
        self.node_ids = node_ids;
        self.node_state = NodeState::Initialized;
    }

    fn try_handle_init(self: &mut Self, msg: &String) -> Option<String> {
        let t = Message::extract_type_from_string(&msg).unwrap();

        if t != "init" {
            panic!("Invalid initialized message type: {}", t)
        }

        messages::init::handle(self, &msg)
    }

    pub fn add_handler<H>(self: &mut Self, t: String, handler: H) -> ()
    where
        H: Fn(&Self, &String) -> Option<String> + 'static,
    {
        HashMap::insert(&mut self.handlers, t, Box::new(handler));
    }

    fn handle(self: &Self, msg: &String) -> Option<String> {
        let t = Message::extract_type_from_string(&msg).unwrap();

        let handler = HashMap::get(&self.handlers, &t).unwrap().deref();

        handler(self, &msg)
    }

    fn send(self: &Self, msg: String) -> () {
        println!("{}", msg);
    }

    pub fn with_state(mut self: Self, state: S) -> Self {
        self.state = Some(state);

        self
    }

    pub fn main_loop(self: &mut Self) -> ! {
        loop {
            let req_msg = self.read();

            let res_msg = match self.node_state {
                NodeState::Uninitialized => self.try_handle_init(&req_msg),
                NodeState::Initialized => self.handle(&req_msg),
            };

            if let Some(msg) = res_msg {
                self.send(msg);
            }
        }
    }
}
