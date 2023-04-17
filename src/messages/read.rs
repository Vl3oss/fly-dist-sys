use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ReadBody {
    pub msg_id: MsgId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReadOkBody<Val = i32> {
    pub in_reply_to: MsgId,
    pub msg_id: MsgId,
    pub messages: Vec<Val>,
}
