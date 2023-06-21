use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use fly_dist_rs::{
    messages::{error::ErrorBody, Message, MsgId},
    node::Node,
};
use serde::{Deserialize, Serialize};

type Key = String;
type Val = usize;
type Offset = usize;

mod lin_kv {
    use crate::*;

    pub const ID: &str = "lin-kv";

    pub fn write<F>(node: &KafkaNode, key: Key, value: Val, on_reply: F)
    where
        F: 'static + Fn(&KafkaNode, Message<Body>) -> () + Send + Sync,
    {
        node.rpc_msg(&msg, on_reply);
    }

    pub fn read<F>(node: &KafkaNode, key: Key, on_reply: F)
    where
        F: 'static + Fn(&KafkaNode, Message<Body>) -> () + Send + Sync,
    {
        let msg = Message {
            src: node.node_id().to_string(),
            dest: ID.to_string(),
            body: Body::Read {
                msg_id: node.next_msg_id(),
                key,
            },
        };

        node.rpc_msg(&msg, on_reply);
    }

    pub fn cas<F>(node: &KafkaNode, key: Key, from: Val, to: Val, on_reply: F)
    where
        F: 'static + Fn(&KafkaNode, Message<Body>) -> () + Send + Sync,
    {
        let msg = Message {
            src: node.node_id().to_string(),
            dest: ID.to_string(),
            body: Body::Cas {
                msg_id: node.next_msg_id(),
                key,
                from,
                to,
            },
        };

        node.rpc_msg(&msg, on_reply);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Send {
        msg_id: MsgId,
        key: Key,
        msg: Val,
    },
    SendOk {
        in_reply_to: MsgId,
        offset: Offset,
    },
    Poll {
        msg_id: MsgId,
        offsets: HashMap<Key, Offset>,
    },
    PollOk {
        in_reply_to: MsgId,
        msgs: HashMap<Key, Vec<(Offset, Val)>>,
    },
    CommitOffsets {
        msg_id: MsgId,
        offsets: HashMap<Key, Offset>,
    },
    CommitOffsetsOk {
        in_reply_to: MsgId,
    },
    ListCommittedOffsets {
        msg_id: MsgId,
        keys: Vec<Key>,
    },
    ListCommittedOffsetsOk {
        in_reply_to: MsgId,
        offsets: HashMap<Key, Offset>,
    },

    Read {
        msg_id: MsgId,
        key: Key,
    },
    ReadOk {
        in_reply_to: MsgId,
        value: Val,
    },
    Write {
        msg_id: MsgId,
        key: Key,
        value: Val,
    },
    WriteOk {
        in_reply_to: MsgId,
    },
    Cas {
        msg_id: MsgId,
        key: Key,
        from: Val,
        to: Val,
    },
    CasOk {
        in_reply_to: MsgId,
    },

    Error(ErrorBody),
}

pub struct State
where
    Self: Send,
{
    logs_db: HashMap<Key, Vec<Val>>,
    committed_offsets: HashMap<Key, Offset>,
    latest_offsets: HashMap<Key, Offset>,
}

type KafkaNode = Node<Mutex<State>, Body>;

pub fn handle_send(node: &KafkaNode, msg: Message<Body>) -> () {
    let (msg_id, key, value, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Send { msg_id, key, msg },
        } => (msg_id, key, msg, src, dest),
        _ => unreachable!(),
    };

    {
        let mut state = node.state.as_ref().unwrap().lock().unwrap();
        let from = state
            .latest_offsets
            .entry(key.clone())
            .or_insert(0)
            .to_owned();
        let to = from.clone() + 1;

        let cas_key = format!("{}_next_offset", key.clone());

        lin_kv::cas(node, cas_key, from, to, move |node, msg| match msg.body {
            Body::CasOk { .. } => {
                let mut state = node.state.as_ref().unwrap().lock().unwrap();
                let logs = state
                    .logs_db
                    .entry(key.clone())
                    .or_insert_with(|| Vec::new());

                logs.push(value);

                let offset = logs.len() - 1;
                node.send_msg(&Message {
                    src: dest.clone(),
                    dest: src.clone(),
                    body: Body::SendOk {
                        in_reply_to: msg_id,
                        offset,
                    },
                })
            }
            Body::Error(ErrorBody {
                in_reply_to,
                code,
                text,
            }) => match code {
                20 => {
                    let msg = Message {
                        src: node.node_id().to_string(),
                        dest: ID.to_string(),
                        body: Body::Write {
                            msg_id: node.next_msg_id(),
                            key,
                            value,
                        },
                    };
                    node.send_msg(msg);
                }
            },
            _ => unreachable!(),
        });
    };
}

pub fn handle_poll(node: &KafkaNode, msg: Message<Body>) -> () {
    let (msg_id, offsets, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Poll { msg_id, offsets },
        } => (msg_id, offsets, src, dest),
        _ => unreachable!(),
    };

    let msgs = {
        let mut msgs = HashMap::new();
        let state = node.state.as_ref().unwrap().lock().unwrap();

        for (key, offset) in offsets {
            let Some(logs) = state.logs_db.get(&key) else {
                eprintln!("getting non existing key {}", key);
                continue;
            };

            msgs.insert(
                key,
                Vec::from_iter(
                    logs[offset..]
                        .iter()
                        .enumerate()
                        .map(|(i, v)| (i + offset, v.to_owned())),
                ),
            );
        }

        msgs
    };

    node.send_msg(&Message {
        src: dest,
        dest: src,
        body: Body::PollOk {
            in_reply_to: msg_id,
            msgs,
        },
    })
}

pub fn handle_commit_offsets(node: &KafkaNode, msg: Message<Body>) -> () {
    let (msg_id, offsets, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::CommitOffsets { msg_id, offsets },
        } => (msg_id, offsets, src, dest),
        _ => unreachable!(),
    };

    {
        let mut state = node.state.as_ref().unwrap().lock().unwrap();
        for (key, offset) in offsets {
            state.committed_offsets.insert(key, offset);
        }
    };

    node.send_msg(&Message {
        src: dest,
        dest: src,
        body: Body::CommitOffsetsOk {
            in_reply_to: msg_id,
        },
    })
}

pub fn handle_list_committed_offsets(node: &KafkaNode, msg: Message<Body>) -> () {
    let (msg_id, keys, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::ListCommittedOffsets { msg_id, keys },
        } => (msg_id, keys, src, dest),
        _ => unreachable!(),
    };

    let offsets = {
        let state = node.state.as_ref().unwrap().lock().unwrap();

        let mut offsets = HashMap::new();
        for key in keys {
            let Some(committed_offset) = state.committed_offsets.get(&key) else {
                continue;
            };

            offsets.insert(key.clone(), committed_offset.to_owned());
        }

        offsets
    };

    node.send_msg(&Message {
        src: dest,
        dest: src,
        body: Body::ListCommittedOffsetsOk {
            in_reply_to: msg_id,
            offsets,
        },
    })
}

pub fn handle_error(_node: &KafkaNode, msg: Message<Body>) -> () {
    let (code, in_reply_to, src, ..) = match msg {
        Message {
            src,
            dest,
            body: Body::Error(ErrorBody {
                code, in_reply_to, ..
            }),
        } => (code, in_reply_to, src, dest),
        _ => unreachable!(),
    };

    match code {
        _ => eprintln!(
            "Unhandle error >> src:{},in_reply_to:{},code:{}",
            src, in_reply_to, code
        ),
    }
}

#[tokio::main]
async fn main() {
    let state = State {
        logs_db: HashMap::new(),
        committed_offsets: HashMap::new(),
        latest_offsets: HashMap::new(),
    };
    let mut node = Node::new().with_state(Mutex::new(state));

    node.add_handler("send".to_string(), handle_send);
    node.add_handler("poll".to_string(), handle_poll);
    node.add_handler("commit_offsets".to_string(), handle_commit_offsets);
    node.add_handler(
        "list_committed_offsets".to_string(),
        handle_list_committed_offsets,
    );
    node.add_handler("error".to_string(), handle_error);

    node.try_init();

    let node = Arc::new(node);

    let main_node = node.clone();
    let main_jh = tokio::spawn(async move {
        loop {
            main_node.one_loop();
        }
    });

    let _ = tokio::join!(main_jh);
}
