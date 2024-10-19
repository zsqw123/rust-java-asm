use java_asm::dex::DexFile;

#[test]
fn read_dex_test() {
    let _dex_file = read_test_dex_file();
    _dex_file;
}

fn read_test_dex_file() -> DexFile {
    let dex_file_bytes = include_bytes!("../res/dex/classes14.dex");
    DexFile::resolve_from_bytes(dex_file_bytes).unwrap()
}
