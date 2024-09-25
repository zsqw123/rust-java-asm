use java_asm::ClassNode;

use crate::jvms::read_test::read_jvms;

#[test]
fn read_node() {
    let jvms = read_jvms().unwrap();
    let node = ClassNode::from_jvms(jvms);
    println!("{:#?}", node.unwrap());
}
