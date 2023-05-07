use std::sync::{Arc, Mutex};

use fly_dist_rs::{
    messages::{error::ErrorBody, Message, MsgId},
    node::Node,
};
use serde::{Deserialize, Serialize};

const SEQ_KV: &str = "seq-kv";
const KEY: &str = "val";
type Val = i32;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Add {
        msg_id: MsgId,
        delta: Val,
    },
    AddOk {
        in_reply_to: MsgId,
    },
    Read {
        msg_id: MsgId,
    },
    ReadOk {
        value: Val,
        in_reply_to: MsgId,
    },

    #[serde(rename = "read")]
    ReadSeq {
        msg_id: MsgId,
        key: String,
    },
    Write {
        msg_id: MsgId,
        key: String,
        value: Val,
    },
    WriteOk {
        in_reply_to: MsgId,
    },
    Cas {
        msg_id: MsgId,
        key: String,
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
    value: Val,
    acc_delta: Val,
    syncing_delta: Val,
}

type GNode = Node<Mutex<State>, Body>;

pub fn handle_add(node: &GNode, msg: Message<Body>) -> () {
    let (msg_id, delta, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Add { delta, msg_id },
        } => (msg_id, delta, src, dest),
        _ => unreachable!(),
    };

    let mut state = node.state.as_ref().unwrap().lock().unwrap();
    state.acc_delta += delta;

    drop(state);

    node.send_msg(&Message {
        src: dest,
        dest: src,
        body: Body::AddOk {
            in_reply_to: msg_id,
        },
    })
}

pub fn handle_read(node: &GNode, msg: Message<Body>) -> () {
    let (msg_id, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Read { msg_id },
        } => (msg_id, src, dest),
        _ => unreachable!(),
    };

    let state = node.state.as_ref().unwrap().lock().unwrap();
    let value = state.value;
    drop(state);

    node.send_msg(&Message {
        src: dest,
        dest: src,
        body: Body::ReadOk {
            value,
            in_reply_to: msg_id,
        },
    })
}

pub fn handle_read_ok_seq(node: &GNode, msg: Message<Body>) -> () {
    let (value, src, ..) = match msg {
        Message {
            src,
            dest,
            body: Body::ReadOk { value, .. },
        } => (value, src, dest),
        _ => {
            eprintln!("unreachable {:?}", msg);
            unreachable!()
        }
    };
    if src != SEQ_KV {
        panic!("get read_ok from non seq-kv");
    }

    let mut state = node.state.as_ref().unwrap().lock().unwrap();
    state.value = value;
    eprintln!("set value from read seq >> {}", value);
    drop(state);
}

pub fn handle_cas_ok_seq(node: &GNode, msg: Message<Body>) -> () {
    let (src, ..) = match msg {
        Message {
            src,
            dest,
            body: Body::CasOk { .. },
        } => (src, dest),
        _ => unreachable!(),
    };
    if src != SEQ_KV {
        panic!("get read_ok from non seq-kv");
    }

    let mut state = node.state.as_ref().unwrap().lock().unwrap();
    state.value += state.acc_delta;
    state.acc_delta -= state.syncing_delta;
    state.syncing_delta = 0;
    eprintln!(
        "set value from cas seq >> value:{},acc_delta:{}",
        state.value, state.acc_delta
    );
    drop(state);
}

pub fn handle_error(node: &GNode, msg: Message<Body>) -> () {
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
    if src != SEQ_KV {
        panic!("get read_ok from non seq-kv");
    }

    match code {
        20 if node.node_id() == "n0" => node.send_msg(&Message {
            src: node.node_id().to_string(),
            dest: SEQ_KV.to_string(),
            body: Body::Write {
                msg_id: node.next_msg_id(),
                key: KEY.to_string(),
                value: 0,
            },
        }),
        _ => eprintln!(
            "Unhandle error >> src:{},in_reply_to:{},code:{}",
            src, in_reply_to, code
        ),
    }
}

fn sync_val(node: &GNode) -> () {
    let mut state = node.state.as_ref().unwrap().lock().unwrap();

    if state.acc_delta == 0 {
        return;
    }

    let delta = state.acc_delta;
    let value = state.value;
    state.syncing_delta = delta;
    drop(state);

    let cas_msg = Message {
        src: node.node_id().to_string(),
        dest: SEQ_KV.to_string(),
        body: Body::Cas {
            msg_id: node.next_msg_id(),
            key: KEY.to_string(),
            from: value,
            to: value + delta,
        },
    };

    node.send_msg(&cas_msg);
}

#[tokio::main]
async fn main() {
    let state = State {
        value: 0,
        acc_delta: 0,
        syncing_delta: 0,
    };
    let mut node = Node::new().with_state(Mutex::new(state));

    node.add_handler("add".to_string(), handle_add);
    node.add_handler("read".to_string(), handle_read);
    node.add_handler("read_ok".to_string(), handle_read_ok_seq);
    node.add_handler("cas_ok".to_string(), handle_cas_ok_seq);
    node.add_handler("error".to_string(), handle_error);

    node.try_init();

    let node = Arc::new(node);

    let main_node = node.clone();
    let main_jh = tokio::spawn(async move {
        loop {
            main_node.one_loop();
        }
    });

    let sync_node = node.clone();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
    let sync_jh = tokio::spawn(async move {
        interval.tick().await;
        loop {
            interval.tick().await;
            node.send_msg(&Message {
                src: node.node_id().to_string(),
                dest: SEQ_KV.to_string(),
                body: Body::ReadSeq {
                    msg_id: node.next_msg_id(),
                    key: KEY.to_string(),
                },
            });
            sync_val(&sync_node);
        }
    });

    let _ = tokio::join!(main_jh, sync_jh);
}
