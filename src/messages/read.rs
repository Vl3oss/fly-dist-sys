use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReadBody {
    Read { msg_id: MsgId },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReadOkBody {
    ReadOk {
        in_reply_to: MsgId,
        msg_id: MsgId,
        messages: Vec<i32>,
    },
}
