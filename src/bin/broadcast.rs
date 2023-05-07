use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use tokio::time;

use fly_dist_rs::{
    messages::{
        broadcast::{BroadcastBody, BroadcastOkBody},
        read::{ReadBody, ReadOkBody},
        topology::{TopologyBody, TopologyOkBody},
        Message, MsgId,
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
        msg_id: MsgId,
        values: HashSet<Val>,
        known_nodes: HashSet<NodeId>,
    },
    PropagateOk {
        in_reply_to: MsgId,
    },
}

#[derive(Clone)]
pub struct State
where
    Self: Send,
{
    pub topology: HashMap<NodeId, Vec<NodeId>>,
    pub values: HashSet<Val>,
    pub unconfirmed_msgs: HashMap<MsgId, Message<Body>>,
    pub to_be_sent_vals: HashMap<NodeId, HashSet<Val>>,
}

type BroadcastNode = Node<Mutex<State>, Body>;

impl State {
    fn add_new_val(self: &mut Self, val: Val) -> () {
        self.values.insert(val);
    }
}

trait GetState<S>
where
    S: Send,
{
    fn get_state(self: &Self) -> &S;
}

impl GetState<Mutex<State>> for BroadcastNode {
    fn get_state(self: &Self) -> &Mutex<State> {
        &self.state.as_ref().unwrap()
    }
}

trait PropagateMsg {
    fn ready(self: &Self) -> bool;
    fn on_recv_val(self: &Self, vals: HashSet<Val>, known_nodes: &HashSet<String>) -> ();
    fn resend_un_resp_msgs(self: &Self) -> ();
    fn propagate_to_friends(self: &Self) -> ();
}

impl PropagateMsg for BroadcastNode {
    fn ready(self: &Self) -> bool {
        !self.get_state().lock().unwrap().topology.is_empty()
    }
    fn on_recv_val(&self, vals: HashSet<Val>, known_nodes: &HashSet<String>) -> () {
        let node_id = self.node_id();

        for val in vals {
            let mut state = self.get_state().lock().unwrap();
            if state.values.contains(&val) {
                continue;
            }
            state.add_new_val(val);

            let topology = &state.topology.to_owned();
            let friends: &Vec<_> = HashMap::get(topology, node_id).unwrap();
            let not_known_friends = friends.iter().filter(|&x| !known_nodes.contains(x));

            let to_be_sent_vals = &mut state.to_be_sent_vals;

            not_known_friends.for_each(|friend| {
                let current_to_be_sent_vals = match to_be_sent_vals.get_mut(friend) {
                    Some(x) => x,
                    None => {
                        to_be_sent_vals.insert(friend.to_string(), HashSet::new());
                        to_be_sent_vals.get_mut(friend).unwrap()
                    }
                };

                current_to_be_sent_vals.insert(val);
            });
        }
    }

    fn resend_un_resp_msgs(self: &Self) {
        let state = self.get_state().lock().unwrap();
        let unconfirmed_msgs = &state.unconfirmed_msgs;

        for msg in unconfirmed_msgs.values() {
            self.send_msg(&msg);
        }
    }

    fn propagate_to_friends(self: &Self) -> () {
        if !self.is_init() || !self.ready() {
            return;
        }

        let mut state = self.get_state().lock().unwrap();
        let node_id = self.node_id();

        let topology = &state.topology;
        let friends: &Vec<_> = HashMap::get(topology, node_id).unwrap();
        let to_be_sent_vals = state.to_be_sent_vals.to_owned();

        let known_nodes = {
            let mut known_nodes: HashSet<NodeId> = HashSet::from_iter(friends.to_owned());
            known_nodes.insert(node_id.clone());
            known_nodes
        };

        let mut sent_msgs = Vec::new();
        for (friend_id, vals) in to_be_sent_vals {
            let propagate_msg_id = self.next_msg_id();
            let propagate_body = Body::Propagate {
                msg_id: propagate_msg_id,
                values: vals,
                known_nodes: known_nodes.clone(),
            };

            let msg = Message {
                src: node_id.clone(),
                dest: friend_id.clone(),
                body: propagate_body.clone(),
            };
            self.send_msg(&msg);

            sent_msgs.push((propagate_msg_id, msg));
        }

        sent_msgs.into_iter().for_each(|(k, v)| {
            state.unconfirmed_msgs.insert(k, v);
        });
    }
}

pub fn handle_topology(node: &BroadcastNode, msg: Message<Body>) -> () {
    let (TopologyBody { msg_id, topology }, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Topology(body),
        } => (body, src, dest),
        _ => unreachable!(),
    };

    let mut state = node.state.as_ref().unwrap().lock().unwrap();
    state.topology = topology;

    let body = Body::TopologyOk(TopologyOkBody {
        in_reply_to: msg_id,
        msg_id: node.next_msg_id(),
    });

    node.send_msg(&Message {
        body,
        src: dest,
        dest: src,
    })
}

pub fn handle_broadcast(node: &BroadcastNode, msg: Message<Body>) -> () {
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

    node.on_recv_val(HashSet::from([value]), &HashSet::new());

    let body = Body::BroadcastOk(BroadcastOkBody {
        in_reply_to: msg_id,
    });

    node.send_msg(&Message {
        body,
        src: dest,
        dest: src,
    })
}

pub fn handle_propagate(node: &BroadcastNode, msg: Message<Body>) -> () {
    let (msg_id, values, known_nodes, src, dest) = match msg {
        Message {
            src,
            dest,
            body:
                Body::Propagate {
                    msg_id,
                    values,
                    known_nodes,
                },
        } => (msg_id, values, known_nodes, src, dest),
        _ => unreachable!(),
    };

    node.on_recv_val(values, &known_nodes);

    node.send_msg(&Message {
        src: dest,
        dest: src,
        body: Body::PropagateOk {
            in_reply_to: msg_id,
        },
    })
}

pub fn handle_propagate_ok(node: &BroadcastNode, msg: Message<Body>) -> () {
    let (in_reply_to, ..) = match msg {
        Message {
            src,
            dest,
            body: Body::PropagateOk { in_reply_to },
        } => (in_reply_to, src, dest),
        _ => unreachable!(),
    };

    let state = &mut node.get_state().lock().unwrap();
    let unconfirmed_msgs = &mut state.unconfirmed_msgs;

    unconfirmed_msgs.remove(&in_reply_to);
}

pub fn handle_read(node: &BroadcastNode, msg: Message<Body>) -> () {
    let (ReadBody { msg_id }, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Read(body),
        } => (body, src, dest),
        _ => unreachable!(),
    };

    let state = node.get_state().lock().unwrap();

    let body = Body::ReadOk(ReadOkBody {
        in_reply_to: msg_id,
        msg_id: node.next_msg_id(),
        messages: state.values.clone().into_iter().collect(),
    });

    node.send_msg(&Message {
        body,
        src: dest,
        dest: src,
    })
}

#[tokio::main]
async fn main() {
    let state = State {
        topology: HashMap::new(),
        values: HashSet::new(),
        unconfirmed_msgs: HashMap::new(),
        to_be_sent_vals: HashMap::new(),
    };
    let mut node = Node::new().with_state(Mutex::new(state));

    node.add_handler("topology".to_string(), handle_topology);
    node.add_handler("broadcast".to_string(), handle_broadcast);
    node.add_handler("read".to_string(), handle_read);
    node.add_handler("propagate".to_string(), handle_propagate);
    node.add_handler("propagate_ok".to_string(), handle_propagate_ok);

    node.try_init();
    let node = Arc::new(node);
    let mut interval = time::interval(time::Duration::from_millis(1000));
    let resend_node = Arc::clone(&node);
    let resend_task = tokio::spawn(async move {
        loop {
            interval.tick().await;
            resend_node.resend_un_resp_msgs();
        }
    });

    let main_node = Arc::clone(&node);
    let main_task = tokio::spawn(async move {
        loop {
            main_node.one_loop();
        }
    });

    let mut interval = time::interval(time::Duration::from_millis(200));
    let propagate_node = Arc::clone(&node);
    interval.tick().await;
    let propagate_task = tokio::spawn(async move {
        loop {
            interval.tick().await;
            propagate_node.propagate_to_friends();
        }
    });

    let _ = tokio::join!(main_task, resend_task, propagate_task);
}
