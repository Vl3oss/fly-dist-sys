use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadBody {
    pub msg_id: MsgId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadOkBody {
    pub in_reply_to: MsgId,
    pub msg_id: MsgId,
    pub messages: Vec<i32>,
}
