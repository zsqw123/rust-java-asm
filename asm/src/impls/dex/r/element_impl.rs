use crate::dex::elements::{CodeItem, DSleb128, EncodedCatchHandler};
use crate::err::AsmResultOkExt;
use crate::impls::jvms::r::{ReadContext, ReadFrom};
use crate::AsmResult;

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
