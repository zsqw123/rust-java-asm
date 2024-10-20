use java_asm::dex::{DexFile, DexFileAccessor};

#[test]
fn read_dex_test() {
    let dex_accessor = read_test_dex_file();
    let demo_class_offset = dex_accessor.file.class_defs[11].class_data_off;
    let demo_class_data = dex_accessor.get_class_data(demo_class_offset).unwrap();
    demo_class_data;
}

fn read_test_dex_file() -> DexFileAccessor<'static> {
    let dex_file_bytes = include_bytes!("../res/dex/classes14.dex");
    let dex_file = DexFile::resolve_from_bytes(dex_file_bytes).unwrap();
    DexFileAccessor {
        file: dex_file,
        bytes: dex_file_bytes,
    }
}
