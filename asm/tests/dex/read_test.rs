use java_asm::dex::{DexFile, DexFileAccessor};
use java_asm::smali::{Dex2Smali, SmaliNode};
use std::rc::Rc;
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

    let resolve_start = Instant::now();
    let instructions = demo_methods.iter().map(|(method_name, code_item)| {
        code_item.as_ref().map(|code_item| {
            let container_smali = code_item.insn_container.to_smali(&dex_accessor);
            let prefix = format!("method {} {}", method_name, container_smali.prefix);
            SmaliNode::new_with_children_and_postfix(
                prefix, container_smali.children, container_smali.postfix.unwrap(),
            ).render(0)
        })
    }).filter_map(|x| x).collect::<Vec<_>>();
    println!("Instructions smali generated in {:?}", resolve_start.elapsed());
    
    println!("{:#?}", demo_class_data);
    println!("{:#?}", demo_methods);
    println!("instructions:\n{}", instructions.join("\n"));
}

fn read_test_dex_file() -> DexFileAccessor {
    let start = Instant::now();
    let dex_file_bytes = include_bytes!("../res/dex/classes14.dex");
    let dex_file = DexFile::resolve_from_bytes(dex_file_bytes).unwrap();
    println!("Dex file resolved in {:?}", start.elapsed());
    DexFileAccessor::new(dex_file, dex_file_bytes.to_vec())
}
