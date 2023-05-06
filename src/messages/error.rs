use serde::{Deserialize, Serialize};

use super::MsgId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorBody {
    pub in_reply_to: MsgId,
    pub code: u32,
    pub text: String,
}
