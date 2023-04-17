use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastBody {
    Broadcast { msg_id: MsgId, message: i32 },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastOkBody {
    BroadcastOk { in_reply_to: MsgId, msg_id: MsgId },
}
