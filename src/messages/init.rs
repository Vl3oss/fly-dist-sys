use crate::node::{Node, NodeId};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{Message, MsgId};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitBody {
    Init {
        msg_id: MsgId,
        node_id: NodeId,
        node_ids: Vec<NodeId>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitOkBody {
    InitOk { msg_id: MsgId, in_reply_to: MsgId },
}

pub fn handle<S, B>(node: &mut Node<S, B>, req_str: &String) -> Option<String>
where
    B: Serialize + DeserializeOwned,
{
    let Message {
        body:
            InitBody::Init {
                node_id,
                node_ids,
                msg_id,
            },
        src,
        dest,
    } = serde_json::from_str::<Message<InitBody>>(&req_str).unwrap();

    node.initialize(node_id, node_ids);

    let body = InitOkBody::InitOk {
        msg_id: msg_id + 1,
        in_reply_to: msg_id,
    };

    let resp_message = Message {
        src: dest,
        dest: src,
        body,
    };

    Some(serde_json::to_string(&resp_message).unwrap())
}
