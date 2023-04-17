use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::node::NodeId;

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TopologyBody {
    Topology {
        msg_id: MsgId,
        topology: HashMap<NodeId, Vec<NodeId>>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TopologyOkBody {
    TopologyOk { in_reply_to: MsgId, msg_id: MsgId },
}
