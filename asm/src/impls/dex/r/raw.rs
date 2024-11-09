use crate::dex::{CodeItem, DSleb128, DUByte, DUInt, DULeb128, DULeb128P1, DebugInfoItem, DexFile, EncodedCatchHandler, EncodedValue, EncodedValueType, Header, InsnContainer, LocalVar, StringData};
use crate::err::AsmResultOkExt;
use crate::impls::jvms::r::*;
use crate::{mutf8_to_string, AsmErr, AsmResult};
use std::collections::HashMap;
use std::io::Read;

impl ReadFrom for CodeItem {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        context.align(4);
        let registers_size = context.read()?;
        let ins_size = context.read()?;
        let outs_size = context.read()?;
        let tries_size = context.read()?;
        let debug_info_off = context.read()?;
        let insn_container = context.read()?;
        let tries;
        let handlers;
        if tries_size > 0 {
            // padding to makes `tries` is 4-byte aligned
            context.align(4);
            tries = context.read_vec(tries_size)?;
            handlers = context.read()?;
        } else {
            handlers = Default::default();
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

impl DebugInfoItem {
    const DBG_END_SEQUENCE: u8 = 0x00;
    const DBG_ADVANCE_PC: u8 = 0x01;
    const DBG_ADVANCE_LINE: u8 = 0x02;
    const DBG_START_LOCAL: u8 = 0x03;
    const DBG_START_LOCAL_EXTENDED: u8 = 0x04;
    const DBG_END_LOCAL: u8 = 0x05;
    const DBG_RESTART_LOCAL: u8 = 0x06;
    const DBG_SET_PROLOGUE_END: u8 = 0x07;
    const DBG_SET_EPILOGUE_BEGIN: u8 = 0x08;
    const DBG_SET_FILE: u8 = 0x09;

    // the smallest special opcode
    const DBG_FIRST_SPECIAL: u8 = 0x0a;
    // the smallest line number increment
    const DBG_LINE_BASE: i8 = -4;
    // the number of line increments represented
    const DBG_LINE_RANGE: u8 = 15;
}

impl ReadFrom for DebugInfoItem {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let line_start: DULeb128 = context.read()?;
        let parameters_size: DULeb128 = context.read()?;
        let parameter_names: Vec<DULeb128P1> = context.read_vec(parameters_size)?;
        let mut records = Vec::new();
        let mut local_vars: Vec<LocalVar> = Vec::new();
        let mut local_var_map: HashMap<DULeb128, LocalVar> = HashMap::new();

        let mut cur_line = line_start.value();
        let mut cur_addr = 0;
        let mut cur_source: DULeb128P1 = DULeb128P1::ZERO;
        loop {
            let opcode: u8 = context.read()?;
            match opcode {
                DebugInfoItem::DBG_END_SEQUENCE => break,

                DebugInfoItem::DBG_ADVANCE_PC => {
                    let addr_diff: DULeb128 = context.read()?;
                    cur_addr += addr_diff.value();
                }
                DebugInfoItem::DBG_ADVANCE_LINE => {
                    let line_diff: DSleb128 = context.read()?;
                    cur_line = (cur_line as i32 + line_diff.value()) as u32;
                }
                DebugInfoItem::DBG_START_LOCAL => {
                    let register: DULeb128 = context.read()?;
                    let name_idx: DULeb128P1 = context.read()?;
                    let type_idx: DULeb128P1 = context.read()?;
                    let sig_idx = DULeb128P1::ZERO;
                    let start_addr = Some(cur_addr);
                    let end_addr = None;
                    let local_var = LocalVar { register, name_idx, type_idx, sig_idx, start_addr, end_addr };
                    local_var_map.insert(register, local_var);
                }
                DebugInfoItem::DBG_START_LOCAL_EXTENDED => {
                    let register: DULeb128 = context.read()?;
                    let name_idx: DULeb128P1 = context.read()?;
                    let type_idx: DULeb128P1 = context.read()?;
                    let sig_idx: DULeb128P1 = context.read()?;
                    let start_addr = Some(cur_addr);
                    let end_addr = None;
                    let local_var = LocalVar { register, name_idx, type_idx, sig_idx, start_addr, end_addr };
                    local_var_map.insert(register, local_var);
                }
                DebugInfoItem::DBG_END_LOCAL => {
                    let reg_num: DULeb128 = context.read()?;
                    if let Some(mut local_var) = local_var_map.remove(&reg_num) {
                        local_var.end_addr = Some(cur_addr);
                        local_vars.push(local_var);
                    }
                }
                DebugInfoItem::DBG_RESTART_LOCAL => {
                    let reg_num: DULeb128 = context.read()?;
                    if let Some(mut old_var) = local_var_map.remove(&reg_num) {
                        old_var.end_addr = Some(cur_addr);
                        let mut new_var = old_var.clone();
                        new_var.start_addr = Some(cur_addr);
                        local_vars.push(old_var); // push old one
                        local_var_map.insert(reg_num, new_var); // insert new one
                    }
                }
                DebugInfoItem::DBG_SET_PROLOGUE_END | DebugInfoItem::DBG_SET_EPILOGUE_BEGIN => continue,
                DebugInfoItem::DBG_SET_FILE => cur_source = context.read()?,
                _ => {
                    let adjusted_opcode = opcode - DebugInfoItem::DBG_FIRST_SPECIAL;
                    let line_diff = DebugInfoItem::DBG_LINE_BASE as i16 + (adjusted_opcode % DebugInfoItem::DBG_LINE_RANGE) as i16;
                    let addr_diff = adjusted_opcode / DebugInfoItem::DBG_LINE_RANGE;
                    cur_line = (cur_line as i32 + line_diff as i32) as u32;
                    cur_addr += addr_diff as u32;
                    records.push((cur_addr, cur_line, cur_source));
                }
            }
        }
        
        if !local_var_map.is_empty() {
            local_vars.extend(local_var_map.into_values());
        }

        DebugInfoItem { line_start, parameter_names, records, local_vars }.ok()
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

impl ReadFrom for EncodedValue {
    fn read_from(context: &mut ReadContext) -> AsmResult<Self> {
        let header_byte: u8 = context.read()?;
        let value_arg = header_byte & 0x1F;
        let value_type = header_byte & 0xE0;
        let encoded_value = match value_type {
            EncodedValueType::VALUE_BYTE =>
                EncodedValue::Byte(context.read()?),
            EncodedValueType::VALUE_SHORT =>
                EncodedValue::Short(read_i16(context, value_arg)?),
            EncodedValueType::VALUE_CHAR =>
                EncodedValue::Char(read_u16(context, value_arg)?),
            EncodedValueType::VALUE_INT =>
                EncodedValue::Int(read_i32(context, value_arg)?),
            EncodedValueType::VALUE_LONG =>
                EncodedValue::Long(read_i64(context, value_arg)?),
            EncodedValueType::VALUE_FLOAT =>
                EncodedValue::Float(read_f32(context, value_arg)?),
            EncodedValueType::VALUE_DOUBLE =>
                EncodedValue::Double(read_f64(context, value_arg)?),
            EncodedValueType::VALUE_METHOD_TYPE =>
                EncodedValue::MethodType(read_u32_based_size(context, value_arg)?),
            EncodedValueType::VALUE_METHOD_HANDLE =>
                EncodedValue::MethodHandle(read_u32_based_size(context, value_arg)?),
            EncodedValueType::VALUE_STRING =>
                EncodedValue::String(read_u32_based_size(context, value_arg)?),
            EncodedValueType::VALUE_TYPE =>
                EncodedValue::Type(read_u32_based_size(context, value_arg)?),
            EncodedValueType::VALUE_FIELD =>
                EncodedValue::Field(read_u32_based_size(context, value_arg)?),
            EncodedValueType::VALUE_METHOD =>
                EncodedValue::Method(read_u32_based_size(context, value_arg)?),
            EncodedValueType::VALUE_ENUM =>
                EncodedValue::Enum(read_u32_based_size(context, value_arg)?),
            EncodedValueType::VALUE_ARRAY =>
                EncodedValue::Array(context.read()?),
            EncodedValueType::VALUE_ANNOTATION =>
                EncodedValue::Annotation(context.read()?),
            EncodedValueType::VALUE_NULL =>
                EncodedValue::Null,
            EncodedValueType::VALUE_BOOLEAN =>
                EncodedValue::Boolean(value_arg != 0),
            _ => return AsmErr::IllegalFormat(
                format!("Unknown encoded value type: {:#X} at offset {:#X} of dex file.", value_type, context.index)
            ).e(),
        };
        Ok(encoded_value)
    }
}

fn read_u16(context: &mut ReadContext, value_arg: u8) -> AsmResult<u16> {
    let value = if value_arg == 0 { // 1 byte
        u8::read_from(context)? as u16
    } else { // 2 bytes
        u16::read_from(context)?
    };
    Ok(value)
}

fn read_u32(context: &mut ReadContext, value_arg: u8) -> AsmResult<u32> {
    let value = if value_arg == 0 { // 1 byte
        u8::read_from(context)? as u32
    } else if value_arg == 1 { // 2 bytes
        u16::read_from(context)? as u32
    } else if value_arg == 2 { // 3 bytes
        U24::read_from(context)?.0
    } else { // 4 bytes
        u32::read_from(context)?
    };
    Ok(value)
}

fn read_u32_based_size(context: &mut ReadContext, value_arg: u8) -> AsmResult<U32BasedSize> {
    let value = U32BasedSize(read_u32(context, value_arg)?);
    Ok(value)
}

fn read_u64(context: &mut ReadContext, value_arg: u8) -> AsmResult<u64> {
    let value = if value_arg == 0 { // 1 byte
        u8::read_from(context)? as u64
    } else if value_arg == 1 { // 2 bytes
        u16::read_from(context)? as u64
    } else if value_arg == 2 { // 3 bytes
        U24::read_from(context)?.0 as u64
    } else if value_arg == 3 { // 4 bytes
        u32::read_from(context)? as u64
    } else if value_arg == 4 { // 5 bytes
        U40::read_from(context)?.0
    } else if value_arg == 5 { // 6 bytes
        U48::read_from(context)?.0
    } else if value_arg == 6 { // 7 bytes
        U56::read_from(context)?.0
    } else { // 8 bytes
        u64::read_from(context)?
    };
    Ok(value)
}

fn read_i16(context: &mut ReadContext, value_arg: u8) -> AsmResult<i16> {
    let value = if value_arg == 0 { // 1 byte
        i8::read_from(context)? as i16
    } else { // 2 bytes
        i16::read_from(context)?
    };
    Ok(value)
}

fn read_i32(context: &mut ReadContext, value_arg: u8) -> AsmResult<i32> {
    let value = if value_arg == 0 { // 1 byte
        i8::read_from(context)? as i32
    } else if value_arg == 1 { // 2 bytes
        i16::read_from(context)? as i32
    } else if value_arg == 2 { // 3 bytes
        I24::read_from(context)?.0
    } else { // 4 bytes
        i32::read_from(context)?
    };
    Ok(value)
}

fn read_i64(context: &mut ReadContext, value_arg: u8) -> AsmResult<i64> {
    let value = if value_arg == 0 { // 1 byte
        i8::read_from(context)? as i64
    } else if value_arg == 1 { // 2 bytes
        i16::read_from(context)? as i64
    } else if value_arg == 2 { // 3 bytes
        I24::read_from(context)?.0 as i64
    } else if value_arg == 3 { // 4 bytes
        i32::read_from(context)? as i64
    } else if value_arg == 4 { // 5 bytes
        I40::read_from(context)?.0
    } else if value_arg == 5 { // 6 bytes
        I48::read_from(context)?.0
    } else if value_arg == 6 { // 7 bytes
        I56::read_from(context)?.0
    } else { // 8 bytes
        i64::read_from(context)?
    };
    Ok(value)
}

fn read_f32(context: &mut ReadContext, value_arg: u8) -> AsmResult<[DUByte; 4]> {
    let mut res = [0u8; 4];
    res[0] = context.read()?;
    if value_arg > 0 {
        res[1] = context.read()?;
    }
    if value_arg > 1 {
        res[2] = context.read()?;
    }
    if value_arg > 2 {
        res[3] = context.read()?;
    }
    Ok(res)
}

fn read_f64(context: &mut ReadContext, value_arg: u8) -> AsmResult<[DUByte; 8]> {
    let mut res = [0u8; 8];
    res[0] = context.read()?;
    if value_arg > 0 {
        res[1] = context.read()?;
    }
    if value_arg > 1 {
        res[2] = context.read()?;
    }
    if value_arg > 2 {
        res[3] = context.read()?;
    }
    if value_arg > 3 {
        res[4] = context.read()?;
    }
    if value_arg > 4 {
        res[5] = context.read()?;
    }
    if value_arg > 5 {
        res[6] = context.read()?;
    }
    if value_arg > 6 {
        res[7] = context.read()?;
    }
    Ok(res)
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
