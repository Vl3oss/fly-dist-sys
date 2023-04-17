use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::node::NodeId;

use super::MsgId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopologyBody {
    pub msg_id: MsgId,
    pub topology: HashMap<NodeId, Vec<NodeId>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct TopologyOkBody {
    pub in_reply_to: MsgId,
    pub msg_id: MsgId,
}
