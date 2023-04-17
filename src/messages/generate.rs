use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct GenerateBody {
    pub msg_id: MsgId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenerateOkBody {
    pub in_reply_to: MsgId,
    pub id: String,
}
