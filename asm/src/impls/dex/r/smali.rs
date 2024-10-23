#![allow(non_snake_case)]

use crate::dex::insn::DexInsn;
use crate::dex::insn_syntax::*;
use crate::dex::{DexFileAccessor, U4};
use crate::impls::ToStringRef;
use crate::smali::{smali, SmaliNode, ToSmali};
use crate::StrRef;

impl ToSmali for (DexFileAccessor, DexInsn) {
    fn to_smali(&self) -> SmaliNode {
        let (accessor, insn) = self;
        match insn {
            DexInsn::Nop(_) => SmaliNode::new("nop"),
            DexInsn::Move(F12x { vA, vB, .. }) =>
                smali!("move v{}, v{}", vA, vB),
            DexInsn::MoveFrom16(F22x { vA, vB, .. }) =>
                smali!("move/from16 v{}, v{}", vA, vB),
            DexInsn::Move16(F32x { vA, vB, .. }) =>
                smali!("move/16 v{}, v{}", vA, vB),
            DexInsn::MoveWide(F12x { vA, vB, .. }) =>
                smali!("move-wide v{}, v{}", vA, vB),
            DexInsn::MoveWideFrom16(F22x { vA, vB, .. }) =>
                smali!("move-wide/from16 v{}, v{}", vA, vB),
            DexInsn::MoveWide16(F32x { vA, vB, .. }) =>
                smali!("move-wide/16 v{}, v{}", vA, vB),
            DexInsn::MoveObject(F12x { vA, vB, .. }) =>
                smali!("move-object v{}, v{}", vA, vB),
            DexInsn::MoveObjectFrom16(F22x { vA, vB, .. }) =>
                smali!("move-object/from16 v{}, v{}", vA, vB),
            DexInsn::MoveObject16(F32x { vA, vB, .. }) =>
                smali!("move-object/16 v{}, v{}", vA, vB),
            DexInsn::MoveResult(F11x { vA, .. }) =>
                smali!("move-result v{}", vA),
            DexInsn::MoveResultWide(F11x { vA, .. }) =>
                smali!("move-result-wide v{}", vA),
            DexInsn::MoveResultObject(F11x { vA, .. }) =>
                smali!("move-result-object v{}", vA),
            DexInsn::MoveException(F11x { vA, .. }) =>
                smali!("move-exception v{}", vA),
            DexInsn::ReturnVoid(_) => SmaliNode::new("return-void"),
            DexInsn::Return(F11x { vA, .. }) =>
                smali!("return v{}", vA),
            DexInsn::ReturnWide(F11x { vA, .. }) =>
                smali!("return-wide v{}", vA),
            DexInsn::ReturnObject(F11x { vA, .. }) =>
                smali!("return-object v{}", vA),
            DexInsn::Const4(F11n { vA, literalB, .. }) =>
                smali!("const/4 v{}, {}", vA, literalB),
            DexInsn::Const16(F21s { vA, literalB, .. }) =>
                smali!("const/16 v{}, {}", vA, literalB),
            DexInsn::Const(F31i { vA, literalB, .. }) =>
                smali!("const v{}, {}", vA, literalB),
            DexInsn::ConstHigh16(F21h { vA, literalB, .. }) =>
                smali!("const/high16 v{}, {}", vA, literalB),
            DexInsn::ConstWide16(F21s { vA, literalB, .. }) =>
                smali!("const-wide/16 v{}, {}", vA, literalB),
            DexInsn::ConstWide32(F31i { vA, literalB, .. }) =>
                smali!("const-wide/32 v{}, {}", vA, literalB),
            DexInsn::ConstWide(F51l { vA, literalB, .. }) =>
                smali!("const-wide v{}, {}", vA, literalB),
            DexInsn::ConstWideHigh16(F21h { vA, literalB, .. }) =>
                smali!("const-wide/high16 v{}, {}", vA, literalB),
            DexInsn::ConstString(F21c { vA, constB, .. }) =>
                smali!("const-string v{}, {}", vA, accessor.opt_str(*constB)),
            DexInsn::ConstStringJumbo(F31c { vA, constB, .. }) =>
                smali!("const-string/jumbo v{}, {}", vA, accessor.opt_str(*constB)),
            DexInsn::ConstClass(F21c { vA, constB, .. }) =>
                smali!("const-class v{}, {}", vA, accessor.opt_type(*constB)),
            DexInsn::MonitorEnter(F11x { vA, .. }) =>
                smali!("monitor-enter v{}", vA),
            DexInsn::MonitorExit(F11x { vA, .. }) =>
                smali!("monitor-exit v{}", vA),
            DexInsn::CheckCast(F21c { vA, constB, .. }) =>
                smali!("check-cast v{}, {}", vA, accessor.opt_type(*constB)),
            DexInsn::InstanceOf(F22c { vA, vB, constC, .. }) =>
                smali!("instance-of v{}, v{}, {}", vA, vB, accessor.opt_type(*constC)),
            DexInsn::ArrayLength(F12x { vA, vB, .. }) =>
                smali!("array-length v{}, v{}", vA, vB),
            DexInsn::NewInstance(F21c { vA, constB, .. }) =>
                smali!("new-instance v{}, {}", vA, accessor.opt_type(*constB)),
            DexInsn::NewArray(F22c { vA, vB, constC, .. }) =>
                smali!("new-array v{}, v{}, {}", vA, vB, accessor.opt_type(*constC)),
            DexInsn::FilledNewArray(F35c { a, vC, vD, vE, vF, vG, constB, .. }) =>
                render_f35("filled-new-array", *a, accessor.opt_str(*constB), *vC, *vD, *vE, *vF, *vG),
            DexInsn::FilledNewArrayRange(F3rc { a, vC, constB, .. }) =>
                render_f3r("filled-new-array/range", *a, accessor.opt_str(*constB), *vC),
            DexInsn::FillArrayData(F31t { vA, offsetB, .. }) =>
                smali!("fill-array-data v{}, @{:+}", vA, offsetB),
            DexInsn::Throw(F11x { vA, .. }) =>
                smali!("throw v{}", vA),
            DexInsn::Goto(F10t { offsetA, .. }) =>
                smali!("goto @{:+}", offsetA),
            DexInsn::Goto16(F20t { offsetA, .. }) =>
                smali!("goto/16 @{:+}", offsetA),
            DexInsn::Goto32(F30t { offsetA, .. }) =>
                smali!("goto/32 @{:+}", offsetA),
            DexInsn::PackedSwitch(F31t { vA, offsetB, .. }) =>
                smali!("packed-switch v{}, @{:+}", vA, offsetB),
            DexInsn::SparseSwitch(F31t { vA, offsetB, .. }) =>
                smali!("sparse-switch v{}, @{:+}", vA, offsetB),
            DexInsn::Cmpkind(F23x { opcode, vA, vB, vC }) => {
                let op_name = match opcode {
                    0x2d => "cmpl-float",
                    0x2e => "cmpg-float",
                    0x2f => "cmpl-double",
                    0x30 => "cmpg-double",
                    0x31 => "cmp-long",
                    _ => "cmpkind",
                };
                smali!("{op_name} v{}, v{}, v{}", vA, vB, vC)
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
                smali!("{op_name} v{}, v{}, @{:+}", vA, vB, offsetC)
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
                smali!("{op_name} v{vA}, @{offsetB:+}")
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
                smali!("{op_name} v{}, v{}, v{}", vA, vB, vC)
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
                smali!("{op_name} v{}, v{}, {}", vA, vB, accessor.opt_field(*constC))
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
                smali!("{op_name} v{}, {}", vA, accessor.opt_field(*constB))
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
                smali!("{op_name} v{}, v{}", vA, vB)
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
                smali!("{op_name} v{}, v{}, v{}", vA, vB, vC)
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
                smali!("{op_name} v{}, v{}", vA, vB)
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
                smali!("{op_name} v{}, v{}, {}", vA, vB, literalC)
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
                smali!("{op_name} v{}, v{}, {}", vA, vB, literalC)
            }
            DexInsn::InvokePoly(f45cc) =>
                render_invoke_poly(accessor, *f45cc),
            DexInsn::InvokePolyRange(f4rcc) =>
                render_invoke_poly_range(accessor, *f4rcc),
            DexInsn::InvokeCustom(F35c { a, vC, vD, vE, vF, vG, constB, .. }) =>
                render_f35("invoke-custom", *a, render_call_site(accessor, *constB), *vC, *vD, *vE, *vF, *vG),
            DexInsn::InvokeCustomRange(F3rc { a, constB, vC, .. }) =>
                render_f3r("invoke-custom-range", *a, render_call_site(accessor, *constB), *vC),
            DexInsn::ConstMethodHandle(F21c { vA, constB, .. }) =>
                smali!("const-method-handle v{}, {}", vA, render_method_handle(accessor, *constB)),
            DexInsn::ConstMethodType(F21c { vA, constB, .. }) =>
                smali!("const-method-type v{}, {}", vA, render_proto(accessor, *constB)),
            DexInsn::NotUsed(_) => smali!(""),
            DexInsn::PackedSwitchPayload(_) => {}
            DexInsn::SparseSwitchPayload(_) => {}
            DexInsn::FillArrayDataPayload(_) => {}
        }
    }
}

fn render_method(accessor: &DexFileAccessor, method_idx: u16) -> StrRef {
    accessor.get_method(method_idx)
        .map(|m| m.to_string())
        .unwrap_or_else(|_| format!("method_{}", method_idx))
        .to_ref()
}

fn render_proto(accessor: &DexFileAccessor, proto_idx: u16) -> StrRef {
    accessor.get_proto(proto_idx)
        .map(|p| p.to_string())
        .unwrap_or_else(|_| format!("proto_{}", proto_idx))
        .to_ref()
}

fn render_call_site(accessor: &DexFileAccessor, call_site_idx: u16) -> StrRef {
    accessor.get_call_site(call_site_idx)
        .map(|cs| cs.to_string())
        .unwrap_or_else(|_| format!("call_site_{}", call_site_idx))
        .to_ref()
}

fn render_method_handle(accessor: &DexFileAccessor, method_handle_idx: u16) -> StrRef {
    accessor.get_method_handle(method_handle_idx)
        .map(|mh| mh.to_string())
        .unwrap_or_else(|_| format!("method_handle_{}", method_handle_idx))
        .to_ref()
}

fn render_invoke_poly(accessor: &DexFileAccessor, f45cc: F45cc) -> SmaliNode {
    let F45cc {
        a, constB, constH,
        vC, vD, vE, vF, vG, ..
    } = f45cc;
    let a = a.0;
    let method = render_method(accessor, constB);
    let proto = render_proto(accessor, constH);
    let mut registers = Vec::with_capacity(8);
    if a > 0 {
        registers.push(format!("v{}", vC));
    };
    if a > 1 {
        registers.push(format!("v{}", vD));
    };
    if a > 2 {
        registers.push(format!("v{}", vE));
    };
    if a > 3 {
        registers.push(format!("v{}", vF));
    };
    if a > 4 {
        registers.push(format!("v{}", vG));
    };
    let registers = registers.join(", ");
    smali!("{op_name} {{{registers}}} {method} {proto}")
}

fn render_invoke_poly_range(accessor: &DexFileAccessor, f4rcc: F4rcc) -> SmaliNode {
    let F4rcc {
        a, constB, constH, vC, ..
    } = f4rcc;
    let method = render_method(accessor, constB);
    let proto = render_proto(accessor, constH);
    let n = vC + (a as u16) - 1;
    smali!("{op_name} v{vC}..v{n} {method} {proto}")
}

fn render_f35(
    op_name: &'static str, a: U4, constB: StrRef,
    vC: U4, vD: U4, vE: U4, vF: U4, vG: U4,
) -> SmaliNode {
    let a = a.0;
    let mut registers = Vec::with_capacity(a as usize);
    if a > 0 {
        registers.push(format!("v{}", vC));
    };
    if a > 1 {
        registers.push(format!("v{}", vD));
    };
    if a > 2 {
        registers.push(format!("v{}", vE));
    };
    if a > 3 {
        registers.push(format!("v{}", vF));
    };
    if a > 4 {
        registers.push(format!("v{}", vG));
    };
    let registers = registers.join(", ");
    smali!(
        "{op_name} {{{registers}}} {constB}"
    )
}

fn render_f3r(
    op_name: &'static str, a: u8, constB: StrRef, vC: u16,
) -> SmaliNode {
    let vN = vC + (a as u16) - 1;
    smali!(
        "{op_name} v{vC}..v{vN} {constB}"
    )
}

