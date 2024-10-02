use java_asm::AsmResult;
use java_asm::jvms::element::ClassFile;
use java_asm::jvms::JvmsClassReader;

#[test]
fn read_jvms_test() {
    println!("{:?}", read_jvms().unwrap());
}

pub(crate) fn read_jvms() -> AsmResult<ClassFile> {
    let bytes = include_bytes!("../res/bytecode/CompileTesting.class");
    JvmsClassReader::read_class_bytes(bytes)
}
