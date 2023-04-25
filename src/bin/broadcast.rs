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
        value: Val,
        known_nodes: HashSet<NodeId>,
    },
    PropagateOk {
        in_reply_to: MsgId,
    },
}

pub struct State
where
    Self: Send,
{
    topology: HashMap<NodeId, Vec<NodeId>>,
    values: HashSet<Val>,
    unconfirmed_msgs: HashMap<MsgId, Message<Body>>,
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
    fn on_recv_val(self: &Self, val: Val, known_nodes: &HashSet<String>) -> ();
    fn resend_un_resp_msgs(self: &Self) -> ();
}

impl PropagateMsg for BroadcastNode {
    fn on_recv_val(&self, val: Val, known_nodes: &HashSet<String>) -> () {
        let mut state = self.get_state().lock().unwrap();
        if state.values.contains(&val) {
            return;
        }
        state.add_new_val(val);

        let node_id = self.node_id();

        let topology = &state.topology;
        let friends: &Vec<_> = HashMap::get(topology, node_id).unwrap();
        let not_known_friends: Vec<String> = friends
            .iter()
            .filter(|&x| !known_nodes.contains(x))
            .map(|f| f.clone())
            .collect();

        let mut new_known_nodes: HashSet<String> = known_nodes.clone();
        new_known_nodes.insert(node_id.to_string());

        friends.iter().for_each(|n| {
            new_known_nodes.insert(n.to_string());
        });

        let unconfirmed_msgs = &mut state.unconfirmed_msgs;

        for dest in not_known_friends {
            let propagate_msg_id = self.next_msg_id();
            let propagate_body = Body::Propagate {
                msg_id: propagate_msg_id,
                value: val,
                known_nodes: new_known_nodes.clone(),
            };

            let msg = Message {
                src: node_id.clone(),
                dest: dest.clone(),
                body: propagate_body.clone(),
            };
            self.send_msg(&msg);

            unconfirmed_msgs.insert(propagate_msg_id, msg);
        }
    }

    fn resend_un_resp_msgs(self: &Self) {
        let state = self.get_state().lock().unwrap();
        let unconfirmed_msgs = &state.unconfirmed_msgs;

        for msg in unconfirmed_msgs.values() {
            self.send_msg(&msg);
        }
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

    let mut state = node.state.as_ref().unwrap().lock().unwrap();
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

    node.on_recv_val(value, &HashSet::new());

    let body = Body::BroadcastOk(BroadcastOkBody {
        in_reply_to: msg_id,
    });

    Some(Message {
        body,
        src: dest,
        dest: src,
    })
}

pub fn handle_propagate(node: &BroadcastNode, msg: Message<Body>) -> Option<Message<Body>> {
    let (msg_id, value, known_nodes, src, dest) = match msg {
        Message {
            src,
            dest,
            body:
                Body::Propagate {
                    msg_id,
                    value,
                    known_nodes,
                },
        } => (msg_id, value, known_nodes, src, dest),
        _ => unreachable!(),
    };

    node.on_recv_val(value, &known_nodes);

    Some(Message {
        src: dest,
        dest: src,
        body: Body::PropagateOk {
            in_reply_to: msg_id,
        },
    })
}

pub fn handle_propagate_ok(node: &BroadcastNode, msg: Message<Body>) -> Option<Message<Body>> {
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

    let state = node.get_state().lock().unwrap();

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

#[tokio::main]
async fn main() {
    let state = State {
        topology: HashMap::new(),
        values: HashSet::new(),
        unconfirmed_msgs: HashMap::new(),
    };
    let mut node = Node::new().with_state(Mutex::new(state));

    node.add_handler("topology".to_string(), handle_topology);
    node.add_handler("broadcast".to_string(), handle_broadcast);
    node.add_handler("read".to_string(), handle_read);
    node.add_handler("propagate".to_string(), handle_propagate);
    node.add_handler("propagate_ok".to_string(), handle_propagate_ok);

    node.try_init();
    let node = Arc::new(node);

    let mut interval = time::interval(time::Duration::from_millis(100));
    let resend_node = Arc::clone(&node);
    let resend_task = tokio::spawn(async move {
        loop {
            interval.tick().await;
            resend_node.resend_un_resp_msgs();
            eprintln!(
                "unconfirmed_msgs={:?}",
                &resend_node
                    .get_state()
                    .lock()
                    .unwrap()
                    .unconfirmed_msgs
                    .keys()
            );
        }
    });

    let main_node = Arc::clone(&node);
    let main_task = tokio::spawn(async move {
        loop {
            main_node.one_loop();
        }
    });

    let _ = tokio::join!(main_task, resend_task);
}
