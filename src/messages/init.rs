use crate::node::NodeId;
use serde::{Deserialize, Serialize};

use super::MsgId;

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
