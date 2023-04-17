use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastBody {
    pub msg_id: MsgId,
    pub message: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BroadcastOkBody {
    pub in_reply_to: MsgId,
    pub msg_id: MsgId,
}
