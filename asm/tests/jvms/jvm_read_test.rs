use java_asm::err::AsmResult;
use java_asm::jvms::element::ClassFile;
use java_asm::jvms::JvmsClassReader;
use java_asm::node::element::ClassNode;

#[test]
fn read_jvms_test() {
    println!("{:?}", read_jvms().unwrap());
}

#[test]
fn read_node() {
    let jvms = read_jvms().unwrap();
    let node = ClassNode::from_jvms(jvms);
    println!("{:?}", node);
}

fn read_jvms() -> AsmResult<ClassFile> {
    let bytes = include_bytes!("../res/bytecode/CompileTesting.class");
    JvmsClassReader::read_class_bytes(bytes)
}
