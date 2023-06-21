pub mod broadcast;
pub mod echo;
pub mod error;
pub mod generate;
pub mod init;
pub mod read;
pub mod topology;

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_json::Result;

pub type MsgId = u32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommonBody {
    #[serde(rename = "type")]
    pub t: String,
    pub msg_id: Option<MsgId>,
    pub in_reply_to: Option<MsgId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message<B = CommonBody>
where
    B: Clone + Debug,
{
    pub src: String,
    pub dest: String,
    pub body: B,
}

impl Message {
    pub fn to_common_message(msg: &String) -> Result<Message<CommonBody>> {
        serde_json::from_str::<Message<CommonBody>>(&msg)
    }
    pub fn extract_type_from_string(msg: &String) -> Result<String> {
        serde_json::from_str::<Message<CommonBody>>(&msg).map(|m| m.body.t)
    }
    pub fn extract_in_msg_id_from_string(msg: &String) -> Result<Option<MsgId>> {
        serde_json::from_str::<Message<CommonBody>>(&msg).map(|m| m.body.msg_id)
    }
    pub fn extract_in_reply_to_from_string(msg: &String) -> Result<Option<MsgId>> {
        serde_json::from_str::<Message<CommonBody>>(&msg).map(|m| m.body.in_reply_to)
    }
}
