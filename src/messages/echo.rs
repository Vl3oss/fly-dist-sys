use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoBody {
    Echo { msg_id: MsgId, echo: Value },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EchoOkBody {
    EchoOk {
        msg_id: MsgId,
        in_reply_to: MsgId,
        echo: Value,
    },
}
