use fly_dist_rs::{
    messages::{
        echo::{EchoBody, EchoOkBody},
        Message,
    },
    node::Node,
};

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

fn main() {
    let mut node = Node::new();

    node.add_handler("echo".to_string(), handle);

    node.handle_loop()
}
