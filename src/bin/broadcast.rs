use std::{cell::RefCell, collections::HashMap};

use fly_dist_rs::{
    messages::{
        broadcast::{BroadcastBody, BroadcastOkBody},
        read::{ReadBody, ReadOkBody},
        topology::{TopologyBody, TopologyOkBody},
        Message,
    },
    node::{Node, NodeId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Broadcast(BroadcastBody),
    BroadcastOk(BroadcastOkBody),
    Read(ReadBody),
    ReadOk(ReadOkBody),
    Topology(TopologyBody),
    TopologyOk(TopologyOkBody),
}

pub struct State {
    topology: HashMap<NodeId, Vec<NodeId>>,
    values: Vec<i32>,
}

pub fn handle_topology(
    node: &Node<RefCell<State>, Body>,
    msg: Message<Body>,
) -> Option<Message<Body>> {
    let (TopologyBody { msg_id, topology }, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Topology(body),
        } => (body, src, dest),
        _ => unreachable!(),
    };

    let mut state = node.state.as_ref().unwrap().borrow_mut();
    state.topology = topology;

    let body = Body::TopologyOk(TopologyOkBody {
        in_reply_to: msg_id,
        msg_id: msg_id + 1,
    });

    Some(Message {
        body,
        src: dest,
        dest: src,
    })
}

pub fn handle_broadcast(
    node: &Node<RefCell<State>, Body>,
    msg: Message<Body>,
) -> Option<Message<Body>> {
    let (BroadcastBody { msg_id, message }, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Broadcast(body),
        } => (body, src, dest),
        _ => unreachable!(),
    };

    let mut state = node.state.as_ref().unwrap().borrow_mut();
    state.values.push(message);

    let body = Body::BroadcastOk(BroadcastOkBody {
        in_reply_to: msg_id,
        msg_id: msg_id + 1,
    });

    Some(Message {
        body,
        src: dest,
        dest: src,
    })
}

pub fn handle_read(node: &Node<RefCell<State>, Body>, msg: Message<Body>) -> Option<Message<Body>> {
    let (ReadBody { msg_id }, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Read(body),
        } => (body, src, dest),
        _ => unreachable!(),
    };

    let state = node.state.as_ref().unwrap().borrow();

    let body = Body::ReadOk(ReadOkBody {
        in_reply_to: msg_id,
        msg_id: msg_id + 1,
        messages: state.values.clone(),
    });

    Some(Message {
        body,
        src: dest,
        dest: src,
    })
}

fn main() {
    let state = State {
        topology: HashMap::new(),
        values: vec![],
    };
    let mut node = Node::new().with_state(RefCell::new(state));

    node.add_handler("topology".to_string(), handle_topology);
    node.add_handler("broadcast".to_string(), handle_broadcast);
    node.add_handler("read".to_string(), handle_read);

    node.main_loop()
}
