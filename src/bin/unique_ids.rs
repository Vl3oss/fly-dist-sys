use std::cell::RefCell;

use fly_dist_rs::{
    messages::{
        generate::{GenerateBody, GenerateOkBody},
        Message,
    },
    node::Node,
};
use serde::{Deserialize, Serialize};

pub struct State {
    count: RefCell<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Generate(GenerateBody),
    GenerateOk(GenerateOkBody),
}

pub fn handle(node: &Node<State, Body>, msg: Message<Body>) -> Option<Message<Body>> {
    let (GenerateBody { msg_id }, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Generate(body),
        } => (body, src, dest),
        _ => unreachable!(),
    };

    let mut count = node.state.as_ref().unwrap().count.borrow_mut();
    let id = node.node_id.clone().unwrap() + &count.clone().to_string();
    *count += 1;

    let body = Body::GenerateOk(GenerateOkBody {
        in_reply_to: msg_id,
        id,
    });

    Some(Message {
        body,
        src: dest,
        dest: src,
    })
}

fn main() {
    let state = State {
        count: RefCell::new(0),
    };
    let mut node = Node::new().with_state(state);

    node.add_handler("generate".to_string(), handle);

    node.main_loop()
}
