use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::messages::init::{InitBody, InitOkBody};
use crate::messages::{CommonBody, Message, MsgId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::stdin;

pub type NodeId = String;

#[derive(Debug, PartialEq)]
struct NodeConfig {
    node_id: NodeId,
    node_ids: Vec<NodeId>,
    internal_msg_id: RefCell<MsgId>,
}

#[derive(Debug, PartialEq)]
enum NodeState {
    Uninitialized,
    Initialized(NodeConfig),
}

pub struct Node<S = (), B = CommonBody> {
    handlers: HashMap<String, Box<dyn Fn(&Node<S, B>, Message<B>) -> Option<Message<B>>>>,
    node_state: NodeState,
    pub state: Option<S>,
    on_end_loop: Box<dyn Fn(&Self) -> ()>,
}

impl<S, B> Node<S, B>
where
    B: Serialize + DeserializeOwned,
{
    pub fn new() -> Self {
        Node {
            handlers: HashMap::new(),
            node_state: NodeState::Uninitialized,
            state: None,
            on_end_loop: Box::new(|_| ()),
        }
    }

    fn read(self: &Self) -> String {
        let mut msg_raw = String::new();
        let _ = stdin().read_line(&mut msg_raw);

        msg_raw
    }

    pub fn read_loop<H>(self: &Self, on_msg: H) -> !
    where
        H: Fn(&Self, String) -> (),
    {
        loop {
            let msg_str = self.read();
            on_msg(self, msg_str);
        }
    }

    pub fn initialize(self: &mut Self, node_id: NodeId, node_ids: Vec<NodeId>) -> () {
        self.node_state = NodeState::Initialized(NodeConfig {
            node_id,
            node_ids,
            internal_msg_id: RefCell::new(0),
        })
    }

    pub fn node_id(self: &Self) -> &NodeId {
        let node_id = match &self.node_state {
            NodeState::Uninitialized => panic!("Node is uninitialized"),
            NodeState::Initialized(config) => &config.node_id,
        };

        node_id
    }

    pub fn node_ids(self: &Self) -> &Vec<NodeId> {
        let node_ids = match &self.node_state {
            NodeState::Uninitialized => panic!("Node is uninitialized"),
            NodeState::Initialized(config) => &config.node_ids,
        };

        node_ids
    }

    pub fn next_msg_id(self: &Self) -> MsgId {
        let mut msg_id = match &self.node_state {
            NodeState::Uninitialized => panic!("Node is uninitialized"),
            NodeState::Initialized(config) => config.internal_msg_id.borrow_mut(),
        };

        *msg_id += 1;
        *msg_id
    }

    fn try_init(self: &mut Self) -> () {
        let req_str = self.read();
        let t = Message::extract_type_from_string(&req_str).unwrap();

        if t != "init" {
            panic!("Invalid initialized message type: {}", t)
        }

        let Message {
            body:
                InitBody::Init {
                    node_id,
                    node_ids,
                    msg_id,
                },
            src,
            dest,
        } = serde_json::from_str::<Message<InitBody>>(&req_str).unwrap();

        self.initialize(node_id, node_ids);

        let body = InitOkBody::InitOk {
            msg_id: msg_id + 1,
            in_reply_to: msg_id,
        };

        let resp_message = Message {
            src: dest,
            dest: src,
            body,
        };

        self.send(serde_json::to_string(&resp_message).unwrap());
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
        eprintln!("node {} sends {}", self.node_id(), msg);

        println!("{}", msg);
    }

    pub fn send_msg(self: &Self, msg: &Message<B>) -> () {
        self.send(serde_json::to_string(msg).unwrap());
    }

    pub fn with_state(mut self: Self, state: S) -> Self {
        self.state = Some(state);

        self
    }

    pub fn on_end_loop<F>(mut self: Self, f: F) -> Self
    where
        F: Fn(&Self) -> () + 'static,
    {
        self.on_end_loop = Box::new(f);

        self
    }

    fn one_loop(self: &Self) -> () {
        let req_str = self.read();
        let res_msg = self.handle(&req_str);

        if let Some(msg) = res_msg {
            self.send(msg)
        }
    }

    pub fn main_loop(self: &mut Self) -> ! {
        self.try_init();
        loop {
            self.one_loop();

            (self.on_end_loop)(self);
        }
    }
}
