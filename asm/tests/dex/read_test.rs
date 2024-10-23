use std::rc::Rc;
use java_asm::dex::{DexFile, DexFileAccessor};
use std::time::Instant;

#[test]
fn read_dex_test() {
    let dex_accessor = read_test_dex_file();
    let demo_class_offset = dex_accessor.file.class_defs[100].class_data_off;
    let resolve_start = Instant::now();
    let demo_class_data = dex_accessor.get_class_data(demo_class_offset).unwrap();
    println!("Class data resolved in {:?}", resolve_start.elapsed());
    
    let resolve_start = Instant::now();
    let demo_methods = demo_class_data.direct_methods.iter().map(|m| {
        let code_item = dex_accessor.get_code_item(m.code_off).unwrap();
        (Rc::clone(&m.name), code_item)
    }).collect::<Vec<_>>();
    println!("Methods instructions resolved in {:?}", resolve_start.elapsed());
    
    println!("{:#?}", demo_class_data);
    println!("{:#?}", demo_methods);
}

fn read_test_dex_file() -> DexFileAccessor {
    let start = Instant::now();
    let dex_file_bytes = include_bytes!("../res/dex/classes14.dex");
    let dex_file = DexFile::resolve_from_bytes(dex_file_bytes).unwrap();
    println!("Dex file resolved in {:?}", start.elapsed());
    DexFileAccessor {
        file: dex_file,
        bytes: dex_file_bytes,
    }
}
