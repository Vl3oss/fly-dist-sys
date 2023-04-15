use crate::node::{Node, NodeId};
use serde::{Deserialize, Serialize};

use super::{Message, MsgId};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitBody {
    Init {
        msg_id: MsgId,
        node_id: NodeId,
        node_ids: Vec<NodeId>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitOkBody {
    InitOk { msg_id: MsgId, in_reply_to: MsgId },
}

pub fn handle<S>(node: &mut Node<S>, msg: &String) -> Option<String> {
    let Message {
        body:
            InitBody::Init {
                node_id,
                node_ids,
                msg_id,
            },
        src,
        dest,
    } = serde_json::from_str::<Message<InitBody>>(&msg).unwrap();

    node.initialize(node_id, node_ids);

    let body = InitOkBody::InitOk {
        msg_id, // TODO: Have to increment this
        in_reply_to: msg_id,
    };

    let resp_message = Message {
        src: dest,
        dest: src,
        body,
    };

    Some(serde_json::to_string(&resp_message).unwrap())
}
