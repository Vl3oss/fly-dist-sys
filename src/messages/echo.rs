use crate::node::Node;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Message, MsgId};

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

pub fn handle(_node: &Node, msg: &String) -> Option<String> {
    let Message {
        body: EchoBody::Echo { echo, msg_id },
        src,
        dest,
    } = serde_json::from_str::<Message<EchoBody>>(&msg).unwrap();

    let body = EchoOkBody::EchoOk {
        msg_id, // TODO: Have to increment this
        in_reply_to: msg_id,
        echo,
    };

    let resp_message = Message {
        body,
        src: dest,
        dest: src,
    };

    Some(serde_json::to_string(&resp_message).unwrap())
}
