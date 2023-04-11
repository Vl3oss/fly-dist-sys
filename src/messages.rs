pub mod echo;
pub mod init;

use serde::{Deserialize, Serialize};

use self::{
    echo::{EchoBody, EchoOkBody},
    init::{InitBody, InitOkBody},
};

type MsgId = u32;

// struct CommonBody {
//     t: String, // type
//     msg_id: Option<MsgId>,
//     in_reply_to: Option<MsgId>,
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub msg_id: MsgId,
    in_reply_to: MsgId,
    code: u32,
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Init(InitBody),
    InitOk(InitOkBody),
    Echo(EchoBody),
    EchoOk(EchoOkBody),
    Error(ErrorBody),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

impl Message {
    pub fn new(src: String, dest: String, body: Body) -> Self {
        Message { src, dest, body }
    }
}
