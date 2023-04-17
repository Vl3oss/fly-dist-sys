use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BroadcastBody<Val = i32> {
    pub msg_id: MsgId,
    pub message: Val,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BroadcastOkBody {
    pub in_reply_to: MsgId,
    pub msg_id: MsgId,
}
