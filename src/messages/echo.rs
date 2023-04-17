use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoBody {
    pub msg_id: MsgId,
    pub echo: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoOkBody {
    pub msg_id: MsgId,
    pub in_reply_to: MsgId,
    pub echo: Value,
}
