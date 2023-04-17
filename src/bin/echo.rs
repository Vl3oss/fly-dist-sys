use fly_dist_rs::{
    messages::{
        echo::{EchoBody, EchoOkBody},
        Message,
    },
    node::Node,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Echo(EchoBody),
    EchoOk(EchoOkBody),
}

pub fn handle(_node: &Node<(), Body>, msg: Message<Body>) -> Option<Message<Body>> {
    let (EchoBody { echo, msg_id }, src, dest) = match msg {
        Message {
            src,
            dest,
            body: Body::Echo(body),
        } => (body, src, dest),
        _ => unreachable!(),
    };

    let body = Body::EchoOk(EchoOkBody {
        msg_id: msg_id + 1,
        in_reply_to: msg_id,
        echo,
    });

    Some(Message {
        body,
        src: dest,
        dest: src,
    })
}

fn main() {
    let mut node = Node::new();

    node.add_handler("echo".to_string(), handle);

    node.main_loop()
}
