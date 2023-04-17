use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EchoBody {
    pub msg_id: MsgId,
    pub echo: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EchoOkBody {
    pub msg_id: MsgId,
    pub in_reply_to: MsgId,
    pub echo: serde_json::Value,
}
