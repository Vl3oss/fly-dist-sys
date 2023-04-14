pub mod echo;
pub mod error;
pub mod init;

use serde::{Deserialize, Serialize};
use serde_json::Result;

type MsgId = u32;

#[derive(Debug, Serialize, Deserialize)]
pub struct CommonBody {
    #[serde(rename = "type")]
    pub t: String,
    msg_id: Option<MsgId>,
    in_reply_to: Option<MsgId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message<B = CommonBody> {
    pub src: String,
    pub dest: String,
    pub body: B,
}

impl Message {
    pub fn extract_type_from_string(msg: &String) -> Result<String> {
        serde_json::from_str::<Message<CommonBody>>(&msg).map(|m| m.body.t)
    }
}
