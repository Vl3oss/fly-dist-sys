use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateBody {
    pub msg_id: MsgId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateOkBody {
    pub in_reply_to: MsgId,
    pub id: String,
}
