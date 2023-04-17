use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

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

type Val = i32;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Broadcast(BroadcastBody<Val>),
    BroadcastOk(BroadcastOkBody),
    Read(ReadBody),
    ReadOk(ReadOkBody<Val>),
    Topology(TopologyBody),
    TopologyOk(TopologyOkBody),
    Propagate {
        value: Val,
        known_nodes: HashSet<NodeId>,
    },
}

pub struct State {
    topology: HashMap<NodeId, Vec<NodeId>>,
    values: HashSet<Val>,
}

type BroadcastNode = Node<RefCell<State>, Body>;

impl State {
    fn add_new_val(self: &mut Self, val: Val) -> () {
        self.values.insert(val);
    }
}

fn on_recv_val(node: &BroadcastNode, val: Val, known_nodes: HashSet<String>) -> () {
    let mut state = node.state.as_ref().unwrap().borrow_mut();
    state.add_new_val(val);

    let topology = &state.topology;
    let friends: &Vec<_> = HashMap::get(topology, node.node_id.as_ref().unwrap()).unwrap();

    let not_known_friends = friends.iter().filter(|&x| !known_nodes.contains(x));

    let mut new_known_nodes: HashSet<String> = known_nodes.clone();
    friends.iter().for_each(|n| {
        new_known_nodes.insert(n.to_string());
    });
    new_known_nodes.insert(node.node_id.clone().unwrap());

    let res_body = Body::Propagate {
        value: val,
        known_nodes: new_known_nodes,
    };

    for dest in not_known_friends {
        node.send_msg(Message {
            src: node.node_id.clone().unwrap(),
            dest: dest.clone(),
            body: res_body.clone(),
        })
    }
}

pub fn handle_topology(node: &BroadcastNode, msg: Message<Body>) -> Option<Message<Body>> {
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

pub fn handle_broadcast(node: &BroadcastNode, msg: Message<Body>) -> Option<Message<Body>> {
    let (
        BroadcastBody {
            msg_id,
            message: value,
        },
        src,
        dest,
    ) = match msg {
        Message {
            src,
            dest,
            body: Body::Broadcast(body),
        } => (body, src, dest),
        _ => unreachable!(),
    };

    on_recv_val(node, value, HashSet::new());

    let body = Body::BroadcastOk(BroadcastOkBody {
        in_reply_to: msg_id,
        msg_id: msg_id + 2,
    });

    Some(Message {
        body,
        src: dest,
        dest: src,
    })
}

pub fn handle_propagate(node: &BroadcastNode, msg: Message<Body>) -> Option<Message<Body>> {
    let (value, known_nodes, ..) = match msg {
        Message {
            src,
            dest,
            body: Body::Propagate { value, known_nodes },
        } => (value, known_nodes, src, dest),
        _ => unreachable!(),
    };

    on_recv_val(node, value, known_nodes);

    // let body = Body::BroadcastOk(BroadcastOkBody {
    //     in_reply_to: msg_id,
    //     msg_id: msg_id + 2,
    // });

    // Some(Message {
    //     body,
    //     src: dest,
    //     dest: src,
    // })
    None
}

pub fn handle_read(node: &BroadcastNode, msg: Message<Body>) -> Option<Message<Body>> {
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
        messages: state.values.clone().into_iter().collect(),
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
        values: HashSet::new(),
    };
    let mut node = Node::new().with_state(RefCell::new(state));

    node.add_handler("topology".to_string(), handle_topology);
    node.add_handler("broadcast".to_string(), handle_broadcast);
    node.add_handler("read".to_string(), handle_read);
    node.add_handler("propagate".to_string(), handle_propagate);

    node.main_loop()
}
