#![allow(non_snake_case)]

use crate::dex::element::{ClassContentElement, FieldElement, MethodElement};
use crate::dex::insn::{DexInsn, FillArrayDataPayload, PackedSwitchPayload, SparseSwitchPayload};
use crate::dex::insn_syntax::*;
use crate::dex::{ClassAccessFlags, ClassDef, CodeItem, DebugInfoItem, DexFileAccessor, EncodedAnnotation, EncodedAnnotationAttribute, EncodedArray, EncodedValue, FieldAccessFlags, InsnContainer, MethodAccessFlags, MethodHandle, MethodHandleType, NO_INDEX, U4};
use crate::impls::dex::r::element::DebugInfoMap;
use crate::impls::ToStringRef;
use crate::smali::{stb, tokens_to_raw, Dex2Smali, SmaliNode};
use crate::{raw_smali, AsmResult, ConstContainer, DescriptorRef, StrRef};
use std::collections::HashMap;

impl InsnContainer {
    fn to_smali(&self, accessor: &DexFileAccessor, mut debug_info: DebugInfoMap) -> SmaliNode {
        let mut current_offset = 0usize;
        let (payloads, insns): (Vec<_>, Vec<_>) = self.insns.iter()
            .map(|insn| {
                let insn_width = insn.insn_width();
                let mapped = (current_offset, insn);
                current_offset += insn_width;
                mapped
            })
            .partition(|(_, insn)| match insn {
                DexInsn::PackedSwitchPayload(_) |
                DexInsn::SparseSwitchPayload(_) |
                DexInsn::FillArrayDataPayload(_) => true,
                _ => false,
            });
        let payload_map: HashMap<usize, &DexInsn> = HashMap::from_iter(payloads);
        let payload_map = PayloadMap { payload_map };

        let mut insn_list: Vec<SmaliNode> = Vec::with_capacity(
            insns.len() + debug_info.records.lines.len() + debug_info.local_vars.lines.len()
        );
        insn_list.shrink_to_fit();
        for (offset, insn) in insns {
            let line_info = debug_info.records.move_to(offset as u32);
            let local_var_info = debug_info.local_vars.move_to(offset as u32);

            for (src_line, src_file_name_idx) in line_info {
                let mut stb = stb().raw(".source-line").other(src_line.to_ref());
                if let Some(src_file_name_idx) = src_file_name_idx.value() {
                    let src_file_name = accessor.opt_str(src_file_name_idx as usize);
                    stb = stb.l(src_file_name);
                }
                insn_list.push(SmaliNode::empty());
                insn_list.push(stb.s());
            }

            for var_info in local_var_info {
                let mut stb = stb().raw(".local").v(var_info.register.value() as u16);
                if let Some(end_addr) = var_info.end_addr {
                    let relative = end_addr - offset as u32;
                    stb = stb.off(offset as u16, relative as u16);
                }
                if let Some(name_idx) = var_info.name_idx.value() {
                    stb = stb.other(accessor.opt_str(name_idx as usize));
                }
                if let Some(type_idx) = var_info.type_idx.value() {
                    stb = stb.d(accessor.opt_type(type_idx as usize));
                }
                if let Some(sig_idx) = var_info.sig_idx.value() {
                    stb = stb.d(accessor.opt_type(sig_idx as usize));
                }
                insn_list.push(stb.s());
            }

            let mut insn = insn.to_smali(accessor, offset, &payload_map);
            insn.offset_hint = Some(offset as u32);
            insn_list.push(insn);
        };

        SmaliNode {
            tag: Some(".code"),
            end_tag: Some(".end code"),
            children: insn_list,
            ..Default::default()
        }
    }
}

struct PayloadMap<'a> {
    payload_map: HashMap<usize, &'a DexInsn>,
}

impl DexInsn {
    fn to_smali(
        &self, accessor: &DexFileAccessor, current_offset: usize,
        payload_map: &PayloadMap,
    ) -> SmaliNode {
        let cur = current_offset as u32;
        let tb = stb();
        let insn = self;
        match insn {
            DexInsn::Nop(_) => tb.op("nop").s(),
            DexInsn::Move(F12x { vA, vB, .. }) =>
                tb.op("move").v(*vA).v(*vB).s(),
            DexInsn::MoveFrom16(F22x { vA, vB, .. }) =>
                tb.op("move/from16").v(*vA).v(*vB).s(),
            DexInsn::Move16(F32x { vA, vB, .. }) =>
                tb.op("move/16").v(*vA).v(*vB).s(),
            DexInsn::MoveWide(F12x { vA, vB, .. }) =>
                tb.op("move-wide").v(*vA).v(*vB).s(),
            DexInsn::MoveWideFrom16(F22x { vA, vB, .. }) =>
                tb.op("move-wide/from16").v(*vA).v(*vB).s(),
            DexInsn::MoveWide16(F32x { vA, vB, .. }) =>
                tb.op("move-wide/16").v(*vA).v(*vB).s(),
            DexInsn::MoveObject(F12x { vA, vB, .. }) =>
                tb.op("move-object").v(*vA).v(*vB).s(),
            DexInsn::MoveObjectFrom16(F22x { vA, vB, .. }) =>
                tb.op("move-object/from16").v(*vA).v(*vB).s(),
            DexInsn::MoveObject16(F32x { vA, vB, .. }) =>
                tb.op("move-object/16").v(*vA).v(*vB).s(),
            DexInsn::MoveResult(F11x { vA, .. }) =>
                tb.op("move-result").v(*vA).s(),
            DexInsn::MoveResultWide(F11x { vA, .. }) =>
                tb.op("move-result-wide").v(*vA).s(),
            DexInsn::MoveResultObject(F11x { vA, .. }) =>
                tb.op("move-result-object").v(*vA).s(),
            DexInsn::MoveException(F11x { vA, .. }) =>
                tb.op("move-exception").v(*vA).s(),
            DexInsn::ReturnVoid(_) => tb.op("return-void").s(),
            DexInsn::Return(F11x { vA, .. }) =>
                tb.op("return").v(*vA).s(),
            DexInsn::ReturnWide(F11x { vA, .. }) =>
                tb.op("return-wide").v(*vA).s(),
            DexInsn::ReturnObject(F11x { vA, .. }) =>
                tb.op("return-object").v(*vA).s(),
            DexInsn::Const4(F11n { vA, literalB, .. }) =>
                tb.op("const/4").v(*vA).l(literalB.0.to_ref()).s(),
            DexInsn::Const16(F21s { vA, literalB, .. }) =>
                tb.op("const/16").v(*vA).l(literalB.to_ref()).s(),
            DexInsn::Const(F31i { vA, literalB, .. }) =>
                tb.op("const").v(*vA).l(literalB.to_ref()).s(),
            DexInsn::ConstHigh16(F21h { vA, literalB, .. }) =>
                tb.op("const/high16").v(*vA).l(literalB.to_ref()).s(),
            DexInsn::ConstWide16(F21s { vA, literalB, .. }) =>
                tb.op("const-wide/16").v(*vA).l(literalB.to_ref()).s(),
            DexInsn::ConstWide32(F31i { vA, literalB, .. }) =>
                tb.op("const-wide/32").v(*vA).l(literalB.to_ref()).s(),
            DexInsn::ConstWide(F51l { vA, literalB, .. }) =>
                tb.op("const-wide").v(*vA).l(literalB.to_ref()).s(),
            DexInsn::ConstWideHigh16(F21h { vA, literalB, .. }) =>
                tb.op("const-wide/high16").v(*vA).l(literalB.to_ref()).s(),
            DexInsn::ConstString(F21c { vA, constB, .. }) =>
                tb.op("const-string").v(*vA).l(accessor.opt_str(*constB)).s(),
            DexInsn::ConstStringJumbo(F31c { vA, constB, .. }) =>
                tb.op("const-string/jumbo").v(*vA).l(accessor.opt_str(*constB)).s(),
            DexInsn::ConstClass(F21c { vA, constB, .. }) =>
                tb.op("const-class").v(*vA).d(accessor.opt_type(*constB)).s(),
            DexInsn::MonitorEnter(F11x { vA, .. }) =>
                tb.op("monitor-enter").v(*vA).s(),
            DexInsn::MonitorExit(F11x { vA, .. }) =>
                tb.op("monitor-exit").v(*vA).s(),
            DexInsn::CheckCast(F21c { vA, constB, .. }) =>
                tb.op("check-cast").v(*vA).d(accessor.opt_type(*constB)).s(),
            DexInsn::InstanceOf(F22c { vA, vB, constC, .. }) =>
                tb.op("instance-of").v(*vA).v(*vB).d(accessor.opt_type(*constC)).s(),
            DexInsn::ArrayLength(F12x { vA, vB, .. }) =>
                tb.op("array-length").v(*vA).v(*vB).s(),
            DexInsn::NewInstance(F21c { vA, constB, .. }) =>
                tb.op("new-instance").v(*vA).d(accessor.opt_type(*constB)).s(),
            DexInsn::NewArray(F22c { vA, vB, constC, .. }) =>
                tb.op("new-array").v(*vA).v(*vB).d(accessor.opt_type(*constC)).s(),
            DexInsn::FilledNewArray(F35c { a, vC, vD, vE, vF, vG, constB, .. }) =>
                render_f35("filled-new-array", *a, tb.l(accessor.opt_str(*constB)).s(),
                           *vC, *vD, *vE, *vF, *vG),
            DexInsn::FilledNewArrayRange(F3rc { a, vC, constB, .. }) =>
                render_f3r("filled-new-array/range", *a, tb.l(accessor.opt_str(*constB)).s(), *vC),
            DexInsn::FillArrayData(F31t { vA, offsetB, .. }) => {
                let payload = payload_map.read(cur, *offsetB);
                tb.op("fill-array-data").v(*vA).append(payload.content).s_with_children(payload.children)
            }
            DexInsn::Throw(F11x { vA, .. }) =>
                tb.op("throw").v(*vA).s(),
            DexInsn::Goto(F10t { offsetA, .. }) =>
                tb.op("goto").off(cur, *offsetA).s(),
            DexInsn::Goto16(F20t { offsetA, .. }) =>
                tb.op("goto/16").off(cur, *offsetA).s(),
            DexInsn::Goto32(F30t { offsetA, .. }) =>
                tb.op("goto/32").off(cur, *offsetA).s(),
            DexInsn::PackedSwitch(F31t { vA, offsetB, .. }) => {
                let payload = payload_map.read(cur, *offsetB);
                tb.op("packed-switch").v(*vA)
                    .append(payload.content).s_with_children(payload.children)
            }
            DexInsn::SparseSwitch(F31t { vA, offsetB, .. }) => {
                let payload = payload_map.read(cur, *offsetB);
                tb.op("sparse-switch").v(*vA)
                    .append(payload.content).s_with_children(payload.children)
            }
            DexInsn::Cmpkind(F23x { opcode, vA, vB, vC }) => {
                let op_name = match opcode {
                    0x2d => "cmpl-float",
                    0x2e => "cmpg-float",
                    0x2f => "cmpl-double",
                    0x30 => "cmpg-double",
                    0x31 => "cmp-long",
                    _ => "cmpkind",
                };
                tb.op(op_name).v(*vA).v(*vB).v(*vC).s()
            }
            DexInsn::IfTest(F22t { opcode, vA, vB, offsetC }) => {
                let op_name = match opcode {
                    0x32 => "if-eq",
                    0x33 => "if-ne",
                    0x34 => "if-lt",
                    0x35 => "if-ge",
                    0x36 => "if-gt",
                    0x37 => "if-le",
                    _ => "if-test",
                };
                tb.op(op_name).v(*vA).v(*vB).off(cur, *offsetC).s()
            }
            DexInsn::IfTestz(F21t { opcode, vA, offsetB }) => {
                let op_name = match opcode {
                    0x38 => "if-eqz",
                    0x39 => "if-nez",
                    0x3a => "if-ltz",
                    0x3b => "if-gez",
                    0x3c => "if-gtz",
                    0x3d => "if-lez",
                    _ => "if-testz"
                };
                tb.op(op_name).v(*vA).off(cur, *offsetB).s()
            }
            DexInsn::ArrayOp(F23x { opcode, vA, vB, vC }) => {
                let op_name = match opcode {
                    0x44 => "aget",
                    0x45 => "aget-wide",
                    0x46 => "aget-object",
                    0x47 => "aget-boolean",
                    0x48 => "aget-byte",
                    0x49 => "aget-char",
                    0x4a => "aget-short",
                    0x4b => "aput",
                    0x4c => "aput-wide",
                    0x4d => "aput-object",
                    0x4e => "aput-boolean",
                    0x4f => "aput-byte",
                    0x50 => "aput-char",
                    0x51 => "aput-short",
                    _ => "arrayop",
                };
                tb.op(op_name).v(*vA).v(*vB).v(*vC).s()
            }
            DexInsn::IInstanceOp(F22c { opcode, vA, vB, constC }) => {
                let op_name = match opcode {
                    0x52 => "iget",
                    0x53 => "iget-wide",
                    0x54 => "iget-object",
                    0x55 => "iget-boolean",
                    0x56 => "iget-byte",
                    0x57 => "iget-char",
                    0x58 => "iget-short",
                    0x59 => "iput",
                    0x5a => "iput-wide",
                    0x5b => "iput-object",
                    0x5c => "iput-boolean",
                    0x5d => "iput-byte",
                    0x5e => "iput-char",
                    0x5f => "iput-short",
                    _ => "instanceop",
                };
                tb.op(op_name).v(*vA).v(*vB).append(render_field(accessor, *constC).content).s()
            }
            DexInsn::SStaticOp(F21c { opcode, vA, constB }) => {
                let op_name = match opcode {
                    0x60 => "sget",
                    0x61 => "sget-wide",
                    0x62 => "sget-object",
                    0x63 => "sget-boolean",
                    0x64 => "sget-byte",
                    0x65 => "sget-char",
                    0x66 => "sget-short",
                    0x67 => "sput",
                    0x68 => "sput-wide",
                    0x69 => "sput-object",
                    0x6a => "sput-boolean",
                    0x6b => "sput-byte",
                    0x6c => "sput-char",
                    0x6d => "sput-short",
                    _ => "staticop",
                };
                tb.op(op_name).v(*vA).append(render_field(accessor, *constB).content).s()
            }
            DexInsn::InvokeKind(F35c { opcode, a, vC, vD, vE, vF, vG, constB }) => {
                let op_name = match opcode {
                    0x6e => "invoke-virtual",
                    0x6f => "invoke-super",
                    0x70 => "invoke-direct",
                    0x71 => "invoke-static",
                    0x72 => "invoke-interface",
                    _ => "invokekind",
                };
                let constB = render_method(accessor, *constB);
                render_f35(op_name, *a, constB, *vC, *vD, *vE, *vF, *vG)
            }
            DexInsn::InvokeKindRange(F3rc { opcode, a, constB, vC }) => {
                let op_name = match opcode {
                    0x74 => "invoke-virtual/range",
                    0x75 => "invoke-super/range",
                    0x76 => "invoke-direct/range",
                    0x77 => "invoke-static/range",
                    0x78 => "invoke-interface/range",
                    _ => "invokekindrange",
                };
                let constB = render_method(accessor, *constB);
                render_f3r(op_name, *a, constB, *vC)
            }
            DexInsn::Unop(F12x { opcode, vA, vB }) => {
                let op_name = match opcode {
                    0x7b => "neg-int",
                    0x7c => "not-int",
                    0x7d => "neg-long",
                    0x7e => "not-long",
                    0x7f => "neg-float",
                    0x80 => "neg-double",
                    0x81 => "int-to-long",
                    0x82 => "int-to-float",
                    0x83 => "int-to-double",
                    0x84 => "long-to-int",
                    0x85 => "long-to-float",
                    0x86 => "long-to-double",
                    0x87 => "float-to-int",
                    0x88 => "float-to-long",
                    0x89 => "float-to-double",
                    0x8a => "double-to-int",
                    0x8b => "double-to-long",
                    0x8c => "double-to-float",
                    0x8d => "int-to-byte",
                    0x8e => "int-to-char",
                    0x8f => "int-to-short",
                    _ => "unop",
                };
                tb.op(op_name).v(*vA).v(*vB).s()
            }
            DexInsn::Binop(F23x { opcode, vA, vB, vC }) => {
                let op_name = match opcode {
                    0x90 => "add-int",
                    0x91 => "sub-int",
                    0x92 => "mul-int",
                    0x93 => "div-int",
                    0x94 => "rem-int",
                    0x95 => "and-int",
                    0x96 => "or-int",
                    0x97 => "xor-int",
                    0x98 => "shl-int",
                    0x99 => "shr-int",
                    0x9a => "ushr-int",
                    0x9b => "add-long",
                    0x9c => "sub-long",
                    0x9d => "mul-long",
                    0x9e => "div-long",
                    0x9f => "rem-long",
                    0xa0 => "and-long",
                    0xa1 => "or-long",
                    0xa2 => "xor-long",
                    0xa3 => "shl-long",
                    0xa4 => "shr-long",
                    0xa5 => "ushr-long",
                    0xa6 => "add-float",
                    0xa7 => "sub-float",
                    0xa8 => "mul-float",
                    0xa9 => "div-float",
                    0xaa => "rem-float",
                    0xab => "add-double",
                    0xac => "sub-double",
                    0xad => "mul-double",
                    0xae => "div-double",
                    0xaf => "rem-double",
                    _ => "binop",
                };
                tb.op(op_name).v(*vA).v(*vB).v(*vC).s()
            }
            DexInsn::Binop2Addr(F12x { opcode, vA, vB }) => {
                let op_name = match opcode {
                    0xb0 => "add-int/2addr",
                    0xb1 => "sub-int/2addr",
                    0xb2 => "mul-int/2addr",
                    0xb3 => "div-int/2addr",
                    0xb4 => "rem-int/2addr",
                    0xb5 => "and-int/2addr",
                    0xb6 => "or-int/2addr",
                    0xb7 => "xor-int/2addr",
                    0xb8 => "shl-int/2addr",
                    0xb9 => "shr-int/2addr",
                    0xba => "ushr-int/2addr",
                    0xbb => "add-long/2addr",
                    0xbc => "sub-long/2addr",
                    0xbd => "mul-long/2addr",
                    0xbe => "div-long/2addr",
                    0xbf => "rem-long/2addr",
                    0xc0 => "and-long/2addr",
                    0xc1 => "or-long/2addr",
                    0xc2 => "xor-long/2addr",
                    0xc3 => "shl-long/2addr",
                    0xc4 => "shr-long/2addr",
                    0xc5 => "ushr-long/2addr",
                    0xc6 => "add-float/2addr",
                    0xc7 => "sub-float/2addr",
                    0xc8 => "mul-float/2addr",
                    0xc9 => "div-float/2addr",
                    0xca => "rem-float/2addr",
                    0xcb => "add-double/2addr",
                    0xcc => "sub-double/2addr",
                    0xcd => "mul-double/2addr",
                    0xce => "div-double/2addr",
                    0xcf => "rem-double/2addr",
                    _ => "binop2addr",
                };
                tb.op(op_name).v(*vA).v(*vB).s()
            }
            DexInsn::BinopLit16(F22s { opcode, vA, vB, literalC }) => {
                let op_name = match opcode {
                    0xd0 => "add-int/lit16",
                    0xd1 => "rsub-int",
                    0xd2 => "mul-int/lit16",
                    0xd3 => "div-int/lit16",
                    0xd4 => "rem-int/lit16",
                    0xd5 => "and-int/lit16",
                    0xd6 => "or-int/lit16",
                    0xd7 => "xor-int/lit16",
                    _ => "binoplit16",
                };
                tb.op(op_name).v(*vA).v(*vB).l(literalC.to_ref()).s()
            },
            DexInsn::BinopLit8(F22b { opcode, vA, vB, literalC }) => {
                let op_name = match opcode {
                    0xd8 => "add-int/lit8",
                    0xd9 => "rsub-int/lit8",
                    0xda => "mul-int/lit8",
                    0xdb => "div-int/lit8",
                    0xdc => "rem-int/lit8",
                    0xdd => "and-int/lit8",
                    0xde => "or-int/lit8",
                    0xdf => "xor-int/lit8",
                    0xe0 => "shl-int/lit8",
                    0xe1 => "shr-int/lit8",
                    0xe2 => "ushr-int/lit8",
                    _ => "binoplit8",
                };
                tb.op(op_name).v(*vA).v(*vB).l(literalC.to_ref()).s()
            }
            DexInsn::InvokePoly(f45cc) =>
                render_invoke_poly(accessor, *f45cc),
            DexInsn::InvokePolyRange(f4rcc) =>
                render_invoke_poly_range(accessor, *f4rcc),
            DexInsn::InvokeCustom(F35c { a, vC, vD, vE, vF, vG, constB, .. }) =>
                render_f35_smali("invoke-custom", *a, render_call_site(accessor, *constB), *vC, *vD, *vE, *vF, *vG),
            DexInsn::InvokeCustomRange(F3rc { a, constB, vC, .. }) =>
                render_f3r_smali("invoke-custom-range", *a, render_call_site(accessor, *constB), *vC),
            DexInsn::ConstMethodHandle(F21c { vA, constB, .. }) =>
                tb.op("const-method-handle").v(*vA).other(render_method_handle_str(accessor, *constB)).s(),
            DexInsn::ConstMethodType(F21c { vA, constB, .. }) =>
                tb.op("const-method-type").v(*vA).d(render_proto(accessor, *constB)).s(),
            DexInsn::NotUsed(_) => SmaliNode::empty(),
            DexInsn::PackedSwitchPayload(p) => p.to_smali(cur),
            DexInsn::SparseSwitchPayload(p) => p.to_smali(cur),
            DexInsn::FillArrayDataPayload(p) => p.to_smali(),
        }
    }
}

impl PayloadMap<'_> {
    pub fn read(&self, current: u32, offset: i32) -> SmaliNode {
        let payload_offset = (current as i32 + offset) as usize;
        let tb = stb();
        let payload = match self.payload_map.get(&payload_offset) {
            Some(p) => p,
            None => return tb.raw("payload").off(current, offset).s(),
        };
        match payload {
            DexInsn::PackedSwitchPayload(p) => p.to_smali(current),
            DexInsn::SparseSwitchPayload(p) => p.to_smali(current),
            DexInsn::FillArrayDataPayload(p) => p.to_smali(),
            _ => tb.raw("payload").off(current, offset).s(),
        }
    }
}

// guaranteed have no children
fn render_field(accessor: &DexFileAccessor, field_idx: u16) -> SmaliNode {
    accessor.get_field(field_idx)
        .map(|f| stb()
            .d(f.class_type).other(f.field_name).d(f.field_type).s()
        ).unwrap_or_else(|_| raw_smali!("field@{}", field_idx))
}

// guaranteed have no children
fn render_method(accessor: &DexFileAccessor, method_idx: u16) -> SmaliNode {
    accessor.get_method(method_idx)
        .map(|m| stb()
            .d(m.class_type).other(m.method_name).d(m.desc).s()
        ).unwrap_or_else(|_| raw_smali!("method@{}", method_idx))
}

fn render_proto(accessor: &DexFileAccessor, proto_idx: u16) -> DescriptorRef {
    accessor.get_proto(proto_idx)
        .map(|p| p.to_string())
        .unwrap_or_else(|_| format!("proto@{}", proto_idx))
        .to_ref()
}

fn render_call_site(accessor: &DexFileAccessor, call_site_idx: u16) -> SmaliNode {
    accessor.get_call_site(call_site_idx)
        .map(|cs| cs.to_smali(accessor))
        .unwrap_or_else(|_| raw_smali!("call_site@{}", call_site_idx))
}

fn render_method_handle(accessor: &DexFileAccessor, method_handle_idx: u16) -> SmaliNode {
    accessor.get_method_handle(method_handle_idx)
        .map(|mh| mh.to_smali(accessor))
        .unwrap_or_else(|_| raw_smali!("method_handle@{}", method_handle_idx))
}

fn render_method_handle_str(accessor: &DexFileAccessor, method_handle_idx: u16) -> StrRef {
    let tokens = render_method_handle(accessor, method_handle_idx).content;
    tokens_to_raw(&tokens).to_ref()
}

fn render_invoke_poly(accessor: &DexFileAccessor, f45cc: F45cc) -> SmaliNode {
    let F45cc {
        a, constB, constH,
        vC, vD, vE, vF, vG, ..
    } = f45cc;
    let a = a.0;
    let mut tb = stb();
    tb = tb.op("invoke-polymorphic");
    let method = render_method(accessor, constB);
    let proto = render_proto(accessor, constH);
    if a > 0 { tb = tb.v(vC) };
    if a > 1 { tb = tb.v(vD) };
    if a > 2 { tb = tb.v(vE) };
    if a > 3 { tb = tb.v(vF) };
    if a > 4 { tb = tb.v(vG) };
    tb.append(method.content).d(proto).s()
}

fn render_invoke_poly_range(accessor: &DexFileAccessor, f4rcc: F4rcc) -> SmaliNode {
    let F4rcc {
        a, constB, constH, vC, ..
    } = f4rcc;
    let method = render_method(accessor, constB);
    let proto = render_proto(accessor, constH);
    let n = vC + (a as u16) - 1;
    stb().op("invoke-polymorphic/range")
        .vr(vC, n).append(method.content).d(proto).s()
}

fn render_f35_smali(
    op_name: &'static str, a: U4, constB: SmaliNode,
    vC: U4, vD: U4, vE: U4, vF: U4, vG: U4,
) -> SmaliNode {
    let a = a.0;
    let mut tb = stb();
    tb = tb.op(op_name);
    if a > 0 { tb = tb.v(vC) };
    if a > 1 { tb = tb.v(vD) };
    if a > 2 { tb = tb.v(vE) };
    if a > 3 { tb = tb.v(vF) };
    if a > 4 { tb = tb.v(vG) };
    tb.append(constB.content).s_with_children(constB.children)
}

fn render_f35(
    op_name: &'static str, a: U4, constB: SmaliNode,
    vC: U4, vD: U4, vE: U4, vF: U4, vG: U4,
) -> SmaliNode {
    render_f35_smali(op_name, a, constB, vC, vD, vE, vF, vG)
}

fn render_f3r_smali(
    op_name: &'static str, a: u8, constB: SmaliNode, vC: u16,
) -> SmaliNode {
    let vN = vC + (a as u16) - 1;
    stb()
        .op(op_name).vr(vC, vN).append(constB.content)
        .s_with_children(constB.children)
}

fn render_f3r(
    op_name: &'static str, a: u8, constB: SmaliNode, vC: u16,
) -> SmaliNode {
    render_f3r_smali(op_name, a, constB, vC)
}

impl Dex2Smali for MethodHandle {
    fn to_smali(&self, dex_file_accessor: &DexFileAccessor) -> SmaliNode {
        let handle_type = MethodHandleType::const_name_or_default(self.method_handle_type, "method_h");
        let member_id = self.field_or_method_id;
        let member = match self.method_handle_type {
            0x00..=0x03 => render_field(dex_file_accessor, member_id),
            0x04..=0x08 => render_method(dex_file_accessor, member_id),
            _ => return raw_smali!("{handle_type}@{member_id}"),
        };
        stb().other(handle_type.to_ref()).append(member.content).s()
    }
}

impl Dex2Smali for EncodedArray {
    fn to_smali(&self, dex_file_accessor: &DexFileAccessor) -> SmaliNode {
        let mut values = Vec::with_capacity(self.values.len());
        for value in self.values.iter() {
            values.push(value.to_smali(dex_file_accessor));
        }
        SmaliNode {
            children: values,
            tag: Some(".array"),
            end_tag: Some(".end array"),
            ..Default::default()
        }
    }
}

impl Dex2Smali for EncodedValue {
    fn to_smali(&self, dex_file_accessor: &DexFileAccessor) -> SmaliNode {
        let tb = stb();
        match self {
            EncodedValue::Byte(v) => tb.l(v.to_ref()).s(),
            EncodedValue::Short(v) => tb.l(v.to_ref()).s(),
            EncodedValue::Char(v) => tb.l(v.to_ref()).s(),
            EncodedValue::Int(v) => tb.l(v.to_ref()).s(),
            EncodedValue::Long(v) => tb.l(v.to_ref()).s(),
            EncodedValue::Float(v) => tb.l(f32::from_be_bytes(*v).to_ref()).s(),
            EncodedValue::Double(v) => tb.l(f64::from_be_bytes(*v).to_ref()).s(),
            EncodedValue::MethodType(v) => tb.l(render_proto(dex_file_accessor, v.0 as u16)).s(),
            EncodedValue::MethodHandle(v) => render_method_handle(dex_file_accessor, v.0 as u16),
            EncodedValue::String(v) => tb.l(dex_file_accessor.opt_str(*v)).s(),
            EncodedValue::Type(v) => tb.d(dex_file_accessor.opt_type(*v)).s(),
            EncodedValue::Field(v) => render_field(dex_file_accessor, v.0 as u16),
            EncodedValue::Method(v) => render_method(dex_file_accessor, v.0 as u16),
            EncodedValue::Enum(v) => render_field(dex_file_accessor, v.0 as u16),
            EncodedValue::Array(v) => v.to_smali(dex_file_accessor),
            EncodedValue::Annotation(v) => v.to_smali(dex_file_accessor),
            EncodedValue::Null => SmaliNode::NULL,
            EncodedValue::Boolean(v) => if *v { SmaliNode::TRUE } else { SmaliNode::FALSE }
        }
    }
}

impl Dex2Smali for EncodedAnnotation {
    fn to_smali(&self, dex_file_accessor: &DexFileAccessor) -> SmaliNode {
        let annotation_type = dex_file_accessor.opt_type(self.type_idx);
        let res: Vec<_> = self.elements.iter().map(|e| e.to_smali(dex_file_accessor)).collect();
        if res.is_empty() {
            stb().raw("annotation").d(annotation_type).s()
        } else {
            stb().raw(".annotation").d(annotation_type)
                .into_smali(res, ".end annotation")
        }
    }
}

impl Dex2Smali for EncodedAnnotationAttribute {
    fn to_smali(&self, dex_file_accessor: &DexFileAccessor) -> SmaliNode {
        let name = dex_file_accessor.opt_str(self.name_idx);
        let value = self.value.to_smali(dex_file_accessor);
        stb().other(name).append(value.content)
            .s_with_children(value.children)
    }
}

impl PackedSwitchPayload {
    fn to_smali(&self, current_offset: u32) -> SmaliNode {
        let mut children = Vec::with_capacity(self.size as usize);
        let size = self.size as u32;
        let first_key = self.first_key;
        for i in 0..size {
            let target_offset = self.targets[i as usize];
            let key = first_key + i as i32;
            let child = stb().l(key.to_ref()).raw("->").off(current_offset, target_offset);
            children.push(child.s())
        }
        SmaliNode {
            children,
            tag: Some(".packed-switch"),
            end_tag: Some(".end packed-switch"),
            ..Default::default()
        }
    }
}

impl SparseSwitchPayload {
    fn to_smali(&self, current_offset: u32) -> SmaliNode {
        let mut children = Vec::with_capacity(self.size as usize);
        for i in 0..self.size {
            let key = self.keys[i as usize];
            let target_offset = self.targets[i as usize];
            let child = stb().l(key.to_ref()).raw("->").off(current_offset, target_offset);
            children.push(child.s())
        }
        SmaliNode {
            children,
            tag: Some(".sparse-switch"),
            end_tag: Some(".end sparse-switch"),
            ..Default::default()
        }
    }
}

impl FillArrayDataPayload {
    fn to_smali(&self) -> SmaliNode {
        let size = self.size.0;
        let data = self.data.iter().map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>().join(" ");
        stb().raw("array-data").other(format!("size={size}").to_ref()).other(data.to_ref()).s()
    }
}

impl ClassDef {
    pub fn to_smali(&self, accessor: &DexFileAccessor) -> AsmResult<SmaliNode> {
        let mut tb = stb();
        let access_flags = self.access_flags;
        tb = ClassAccessFlags::render(access_flags, tb);
        let class_type = accessor.opt_type(self.class_idx);
        let mut smali = tb.d(class_type).s();

        if self.source_file_idx.0 != NO_INDEX {
            let source_file = accessor.opt_str(self.source_file_idx);
            smali.add_child(stb().raw(".source").other(source_file).s());
        };
        
        if self.superclass_idx.0 != NO_INDEX {
            let super_type = accessor.opt_type(self.superclass_idx);
            smali.add_child(stb().raw(".super").d(super_type).s());
        };

        let interfaces = accessor.get_type_list(self.interfaces_off)?;
        for interface in interfaces {
            smali.add_child(stb().raw(".implements").d(interface).s());
        }

        if self.class_data_off != 0 {
            let class_element = accessor.get_class_element(self.class_data_off)?;
            // transparent for children, no more level
            smali.children.extend(class_element.to_smali(accessor)?.children);
        };
        Ok(smali)
    }
}

impl ClassContentElement {
    pub fn to_smali(&self, accessor: &DexFileAccessor) -> AsmResult<SmaliNode> {
        let mut smali = SmaliNode::empty();
        for field in self.static_fields.iter() {
            smali.add_child(field.to_smali());
        }
        for field in self.instance_fields.iter() {
            smali.add_child(field.to_smali());
        }
        for method in self.direct_methods.iter() {
            smali.add_child(method.to_smali(accessor)?);
        }
        for method in self.virtual_methods.iter() {
            smali.add_child(method.to_smali(accessor)?);
        }
        Ok(smali)
    }
}

impl FieldElement {
    pub fn to_smali(&self) -> SmaliNode {
        let mut tb = stb();
        let access_flags = self.access_flags;
        tb = FieldAccessFlags::render(access_flags, tb);
        let name = self.name.clone();
        let descriptor = self.descriptor.clone();
        let smali = tb.other(name).d(descriptor).s();
        smali
    }
}

impl MethodElement {
    pub fn to_smali(&self, accessor: &DexFileAccessor) -> AsmResult<SmaliNode> {
        let mut tb = stb();
        let access_flags = self.access_flags;
        tb = MethodAccessFlags::render(access_flags, tb);
        let name = self.name.clone();
        let descriptor = format!("({}){}", self.parameters.join(""), self.return_type);
        let mut smali = tb.other(name).d(descriptor.to_ref()).s();
        let code = accessor.get_code_item(self.code_off)?;
        if let Some(code) = code {
            smali.children.extend(code.to_smali(accessor).children);
        }
        Ok(smali)
    }
}

impl CodeItem {
    pub fn to_smali(&self, accessor: &DexFileAccessor) -> SmaliNode {
        let mut smali = SmaliNode::empty();
        let debug_info_item: Option<DebugInfoItem> = accessor.get_data_impl(self.debug_info_off).ok();

        let registers_size = self.registers_size;
        smali.add_child(stb().raw(".registers").l(registers_size.to_ref()).s());

        self.add_parameters(accessor, &mut smali, &debug_info_item);

        let debug_info = DebugInfoMap::from_raw(debug_info_item);
        let insn_container_smali = self.insn_container.to_smali(accessor, debug_info);
        smali.children.extend(insn_container_smali.children);

        smali
    }

    fn add_parameters(
        &self, accessor: &DexFileAccessor, smali_node: &mut SmaliNode, debug_info: &Option<DebugInfoItem>,
    ) {
        let Some(debug_info) = debug_info else { return; };
        let parameter_names = &debug_info.parameter_names;
        // some dex modification magic might shares same debug info with different method
        // to minimize the package size. We only takes debug info that we need.
        let parameter_count = self.ins_size as usize;
        for (i, name) in parameter_names.iter().enumerate().take(parameter_count) {
            let Some(name_idx) = name.value() else { continue };
            let name = accessor.opt_str(name_idx as usize);
            smali_node.add_child(stb().raw(".parameter").v(i as u16).l(name).s());
        }
    }
}




