use java_asm::jvms::read::jvms_reader::JvmsClassReader;

#[test]
fn read_jvms() {
    let bytes = include_bytes!("../res/bytecode/CompileTesting.class");
    JvmsClassReader::read_class_bytes(bytes).unwrap();
}
