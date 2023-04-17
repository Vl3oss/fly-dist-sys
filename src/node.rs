use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::messages::{self, CommonBody, Message};
use std::collections::HashMap;
use std::io::stdin;

pub type NodeId = String;

#[derive(Debug, PartialEq)]
enum NodeState {
    Uninitialized,
    Initialized,
}

pub struct Node<S = (), B = CommonBody> {
    pub node_id: Option<NodeId>,
    pub node_ids: Vec<NodeId>,
    handlers: HashMap<String, Box<dyn Fn(&Node<S, B>, Message<B>) -> Option<Message<B>>>>,
    node_state: NodeState,
    pub state: Option<S>,
}

impl<S, B> Node<S, B>
where
    B: Serialize + DeserializeOwned,
{
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

    fn try_handle_init(self: &mut Self, req_str: &String) -> Option<String> {
        let t = Message::extract_type_from_string(&req_str).unwrap();

        if t != "init" {
            panic!("Invalid initialized message type: {}", t)
        }

        messages::init::handle(self, &req_str)
    }

    pub fn add_handler<H>(self: &mut Self, t: String, handler: H) -> ()
    where
        H: Fn(&Self, Message<B>) -> Option<Message<B>> + 'static,
    {
        HashMap::insert(&mut self.handlers, t, Box::new(handler));
    }

    fn handle(self: &Self, req_str: &String) -> Option<String> {
        let t = Message::extract_type_from_string(&req_str).unwrap();
        let handler = HashMap::get(&self.handlers, &t);

        if handler.is_none() {
            eprintln!("Skip handling unknown message type: '{}'", t);
            return None;
        }
        let handler = handler.unwrap();

        let req_msg = serde_json::from_str(req_str).unwrap();
        let res_msg = (handler)(self, req_msg);

        res_msg.map(|m| serde_json::to_string(&m).unwrap())
    }

    fn send(self: &Self, msg: String) -> () {
        println!("{}", msg);
    }

    pub fn send_msg(self: &Self, msg: Message<B>) -> () {
        self.send(serde_json::to_string(&msg).unwrap());
    }

    pub fn with_state(mut self: Self, state: S) -> Self {
        self.state = Some(state);

        self
    }

    pub fn main_loop(self: &mut Self) -> ! {
        loop {
            let req_str = self.read();

            let res_msg = match self.node_state {
                NodeState::Uninitialized => self.try_handle_init(&req_str),
                NodeState::Initialized => self.handle(&req_str),
            };

            if let Some(msg) = res_msg {
                self.send(msg);
            }
        }
    }
}
