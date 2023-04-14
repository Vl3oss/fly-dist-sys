use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub msg_id: MsgId,
    in_reply_to: MsgId,
    code: u32,
    text: String,
}
