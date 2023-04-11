use crate::node::{Node, NodeId};
use serde::{Deserialize, Serialize};

use super::{Body, MsgId};

#[derive(Debug, Serialize, Deserialize)]
pub struct InitBody {
    pub msg_id: MsgId,
    pub node_id: NodeId,
    pub node_ids: Vec<NodeId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitOkBody {
    pub msg_id: MsgId,
    in_reply_to: MsgId,
}

pub fn handle(
    node: &mut Node,
    InitBody {
        node_id,
        node_ids,
        msg_id,
    }: InitBody,
) -> Option<Body> {
    node.node_id = Some(node_id);
    node.node_ids = node_ids;

    let body = InitOkBody {
        msg_id, // TODO: Have to increment this
        in_reply_to: msg_id,
    };
    Some(Body::InitOk(body))
}
