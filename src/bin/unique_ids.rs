use std::cell::RefCell;

use fly_dist_rs::{
    messages::{
        generate::{GenerateBody, GenerateOkBody},
        Message,
    },
    node::Node,
};

pub struct State {
    count: RefCell<u32>,
}

pub fn handle(node: &Node<State>, msg: &String) -> Option<String> {
    let Message {
        body: GenerateBody::Generate { msg_id },
        src,
        dest,
    } = serde_json::from_str::<Message<GenerateBody>>(&msg).unwrap();

    let mut count = node.state.as_ref().unwrap().count.borrow_mut();
    let id = node.node_id.clone().unwrap() + &count.clone().to_string();
    *count += 1;

    let body = GenerateOkBody::GenerateOk {
        in_reply_to: msg_id,
        id,
    };

    let resp_message = Message {
        body,
        src: dest,
        dest: src,
    };

    Some(serde_json::to_string(&resp_message).unwrap())
}

fn main() {
    let state = State {
        count: RefCell::new(0),
    };
    let mut node = Node::new().with_state(state);

    node.add_handler("generate".to_string(), handle);

    node.main_loop()
}
