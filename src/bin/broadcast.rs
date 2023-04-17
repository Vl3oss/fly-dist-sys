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

pub struct State {
    topology: HashMap<NodeId, Vec<NodeId>>,
    values: Vec<i32>,
}

pub fn handle_topology(node: &Node<RefCell<State>>, msg: &String) -> Option<String> {
    let Message {
        body: TopologyBody::Topology { msg_id, topology },
        src,
        dest,
    } = serde_json::from_str::<Message<TopologyBody>>(&msg).unwrap();

    let mut state = node.state.as_ref().unwrap().borrow_mut();
    state.topology = topology;

    let body = TopologyOkBody::TopologyOk {
        in_reply_to: msg_id,
        msg_id: msg_id + 1,
    };

    let resp_message = Message {
        body,
        src: dest,
        dest: src,
    };

    Some(serde_json::to_string(&resp_message).unwrap())
}

pub fn handle_broadcast(node: &Node<RefCell<State>>, msg: &String) -> Option<String> {
    let Message {
        body: BroadcastBody::Broadcast { msg_id, message },
        src,
        dest,
    } = serde_json::from_str::<Message<BroadcastBody>>(&msg).unwrap();

    let mut state = node.state.as_ref().unwrap().borrow_mut();
    state.values.push(message);

    let body = BroadcastOkBody::BroadcastOk {
        in_reply_to: msg_id,
        msg_id: msg_id + 1,
    };

    let resp_message = Message {
        body,
        src: dest,
        dest: src,
    };

    Some(serde_json::to_string(&resp_message).unwrap())
}

pub fn handle_read(node: &Node<RefCell<State>>, msg: &String) -> Option<String> {
    let Message {
        body: ReadBody::Read { msg_id },
        src,
        dest,
    } = serde_json::from_str::<Message<ReadBody>>(&msg).unwrap();

    let state = node.state.as_ref().unwrap().borrow();

    let body = ReadOkBody::ReadOk {
        in_reply_to: msg_id,
        msg_id: msg_id + 1,
        messages: state.values.clone(),
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
        topology: HashMap::new(),
        values: vec![],
    };
    let mut node = Node::new().with_state(RefCell::new(state));

    node.add_handler("topology".to_string(), handle_topology);
    node.add_handler("broadcast".to_string(), handle_broadcast);
    node.add_handler("read".to_string(), handle_read);

    node.main_loop()
}
