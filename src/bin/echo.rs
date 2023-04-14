use fly_dist_rs::{messages, node::Node};

fn main() {
    let mut node = Node::new();

    node.add_handler("echo".to_string(), messages::echo::handle);

    node.handle_loop()
}
