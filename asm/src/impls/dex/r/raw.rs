use crate::{mutf8_to_string, AsmResult};
use crate::dex::{CodeItem, DSleb128, DUInt, DULeb128, DexFile, EncodedCatchHandler, Header, InsnContainer, StringData};
use crate::err::AsmResultOkExt;
use crate::impls::jvms::r::{ReadContext, ReadFrom};

impl ReadFrom for CodeItem {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let registers_size = context.read()?;
        let ins_size = context.read()?;
        let outs_size = context.read()?;
        let tries_size = context.read()?;
        let debug_info_off = context.read()?;
        let insn_container = context.read()?;
        let handlers;
        let tries;
        if tries_size > 0 {
            // padding to makes `tries` is 4-byte aligned
            context.align(4);
            handlers = Default::default();
            tries = context.read_vec(tries_size)?;
        } else {
            handlers = context.read()?;
            tries = Vec::new();
        }
        CodeItem {
            registers_size, ins_size, outs_size, tries_size,
            debug_info_off, insn_container, tries, handlers,
        }.ok()
    }
}

impl ReadFrom for InsnContainer {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let insns_size: DUInt = context.read()?;
        let mut insns = Vec::new();
        let len_of_insns = (insns_size * 2) as usize;
        let mut cur = 0usize;
        while cur < len_of_insns {
            let start = context.index;
            let insn = context.read()?;
            insns.push(insn);
            let end = context.index;
            cur += end - start;
        }
        Ok(InsnContainer { insns_size, insns })
    }
}

impl ReadFrom for EncodedCatchHandler {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let size: DSleb128 = context.read()?;
        let size_value = size.value();
        let handler_size = size_value.abs() as usize;
        let handlers = context.read_vec(handler_size)?;
        let catch_all_addr = if size_value < 0 {
            Some(context.read()?)
        } else {
            None
        };
        Ok(EncodedCatchHandler { size, handlers, catch_all_addr })
    }
}

impl ReadFrom for DexFile {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let header: Header = context.read()?;
        let string_ids = context.read_vec(header.string_ids_size)?;
        let type_ids = context.read_vec(header.type_ids_size)?;
        let proto_ids = context.read_vec(header.proto_ids_size)?;
        let field_ids = context.read_vec(header.field_ids_size)?;
        let method_ids = context.read_vec(header.method_ids_size)?;
        let class_defs = context.read_vec(header.class_defs_size)?;
        DexFile {
            header,
            string_ids, type_ids, proto_ids, field_ids, method_ids,
            class_defs,
        }.ok()
    }
}

impl ReadFrom for StringData {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let utf16_size: DULeb128 = context.read()?;
        let mut vec = Vec::new();
        loop {
            let current: u8 = context.read()?;
            if current == 0 { break; }
            vec.push(current);
        }
        let str_ref = mutf8_to_string(&vec)?;
        Ok(StringData { utf16_size, str_ref })
    }
}
