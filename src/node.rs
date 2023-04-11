use crate::messages::echo;
use crate::messages::init;
use crate::messages::{Body, Message};
use std::io::stdin;

pub type NodeId = String;

#[derive(Debug)]
pub struct Node {
    pub node_id: Option<NodeId>,
    pub node_ids: Vec<NodeId>,
}

impl Node {
    pub fn new() -> Self {
        Node {
            node_id: None,
            node_ids: vec![],
        }
    }

    fn read(self: &Self) -> Message {
        let mut msg_raw = String::new();
        let _ = stdin().read_line(&mut msg_raw);

        serde_json::from_str(&msg_raw).unwrap()
    }

    fn handle(self: &mut Self, msg: Message) -> Option<Message> {
        let body = match msg.body {
            Body::Init(body) => init::handle(self, body),
            Body::Echo(body) => echo::handle(self, body),
            _ => {
                eprintln!("Unhandle message {:?}", msg);
                None
            }
        };

        if let Some(b) = body {
            Some(Message::new(self.node_id.clone().unwrap(), msg.src, b))
        } else {
            None
        }
    }

    fn send(self: &Self, msg: &Message) -> () {
        println!("{}", serde_json::to_string(&msg).unwrap());
    }

    pub fn handle_loop(self: &mut Self) -> ! {
        loop {
            let req_msg = self.read();
            let res_msg = self.handle(req_msg);

            if let Some(msg) = res_msg {
                self.send(&msg);
            }
        }
    }
}
