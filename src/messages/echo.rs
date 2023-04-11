use crate::node::Node;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Body, MsgId};

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoBody {
    pub msg_id: MsgId,
    pub echo: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoOkBody {
    pub msg_id: MsgId,
    in_reply_to: MsgId,
    echo: Value,
}

pub fn handle(_node: &Node, EchoBody { echo, msg_id }: EchoBody) -> Option<Body> {
    let body = EchoOkBody {
        msg_id, // TODO: Have to increment this
        in_reply_to: msg_id,
        echo,
    };

    Some(Body::EchoOk(body))
}
