use crate::jvms::read_test::read_jvms;
use java_asm::node::element::ClassNode;
use std::time::Instant;

#[test]
fn read_node() {
    let jvms = read_jvms().unwrap();
    
    let start = Instant::now();
    let node = ClassNode::from_jvms(jvms);

    println!("node resolve cost: {:?}", start.elapsed());
    println!("{:#?}", node.unwrap());
}
