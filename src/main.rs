use node::Node;

mod messages;
mod node;

fn main() {
    let mut node = Node::new();

    node.handle_loop()
}
