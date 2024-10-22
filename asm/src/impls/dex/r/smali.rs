use crate::dex::DexFileAccessor;
use crate::dex::insn::DexInsn;
use crate::dex::insn_syntax::*;
use crate::smali::{SmaliNode, ToSmali};

impl ToSmali for (DexFileAccessor, DexInsn) {
    fn to_smali(&self) -> SmaliNode {
        let (accessor, insn) = self;
        match insn {
            DexInsn::Nop(_) => SmaliNode::new("nop"),
            DexInsn::Move(F12x { vA, vB, .. }) =>
                SmaliNode::new(format!("move v{}, v{}", vA, vB)),
            DexInsn::MoveFrom16(F22x { vA, vB, .. }) =>
                SmaliNode::new(format!("move/from16 v{}, v{}", vA, vB)),
            DexInsn::Move16(F32x { vA, vB, .. }) =>
                SmaliNode::new(format!("move/16 v{}, v{}", vA, vB)),
            DexInsn::MoveWide(F12x { vA, vB, .. }) =>
                SmaliNode::new(format!("move-wide v{}, v{}", vA, vB)),
            DexInsn::MoveWideFrom16(F22x { vA, vB, .. }) =>
                SmaliNode::new(format!("move-wide/from16 v{}, v{}", vA, vB)),
            DexInsn::MoveWide16(F32x { vA, vB, .. }) =>
                SmaliNode::new(format!("move-wide/16 v{}, v{}", vA, vB)),
            DexInsn::MoveObject(F12x { vA, vB, .. }) =>
                SmaliNode::new(format!("move-object v{}, v{}", vA, vB)),
            DexInsn::MoveObjectFrom16(F22x { vA, vB, .. }) =>
                SmaliNode::new(format!("move-object/from16 v{}, v{}", vA, vB)),
            DexInsn::MoveObject16(F32x { vA, vB, .. }) =>
                SmaliNode::new(format!("move-object/16 v{}, v{}", vA, vB)),
            DexInsn::MoveResult(F11x { vA, .. }) =>
                SmaliNode::new(format!("move-result v{}", vA)),
            DexInsn::MoveResultWide(F11x { vA, .. }) =>
                SmaliNode::new(format!("move-result-wide v{}", vA)),
            DexInsn::MoveResultObject(F11x { vA, .. }) =>
                SmaliNode::new(format!("move-result-object v{}", vA)),
            DexInsn::MoveException(F11x { vA, .. }) =>
                SmaliNode::new(format!("move-exception v{}", vA)),
            DexInsn::ReturnVoid(_) => SmaliNode::new("return-void"),
            DexInsn::Return(F11x { vA, .. }) =>
                SmaliNode::new(format!("return v{}", vA)),
            DexInsn::ReturnWide(F11x { vA, .. }) =>
                SmaliNode::new(format!("return-wide v{}", vA)),
            DexInsn::ReturnObject(F11x { vA, .. }) =>
                SmaliNode::new(format!("return-object v{}", vA)),
            DexInsn::Const4(F11n { vA, literalB, .. }) =>
                SmaliNode::new(format!("const/4 v{}, #{}", vA, literalB)),
            DexInsn::Const16(F21s { vA, literalB, .. }) =>
                SmaliNode::new(format!("const/16 v{}, #{}", vA, literalB)),
            DexInsn::Const(F31i { vA, literalB, .. }) =>
                SmaliNode::new(format!("const v{}, #{}", vA, literalB)),
            DexInsn::ConstHigh16(F21h { vA, literalB, .. }) =>
                SmaliNode::new(format!("const/high16 v{}, #{}", vA, literalB)),
            DexInsn::ConstWide16(F21s { vA, literalB, .. }) =>
                SmaliNode::new(format!("const-wide/16 v{}, #{}", vA, literalB)),
            DexInsn::ConstWide32(F31i { vA, literalB, .. }) =>
                SmaliNode::new(format!("const-wide/32 v{}, #{}", vA, literalB)),
            DexInsn::ConstWide(F51l { vA, literalB, .. }) =>
                SmaliNode::new(format!("const-wide v{}, #{}", vA, literalB)),
            DexInsn::ConstWideHigh16(F21h { vA, literalB, .. }) =>
                SmaliNode::new(format!("const-wide/high16 v{}, #{}", vA, literalB)),
            DexInsn::ConstString(_) => {}
            DexInsn::ConstStringJumbo(_) => {}
            DexInsn::ConstClass(_) => {}
            DexInsn::MonitorEnter(_) => {}
            DexInsn::MonitorExit(_) => {}
            DexInsn::CheckCast(_) => {}
            DexInsn::InstanceOf(_) => {}
            DexInsn::ArrayLength(_) => {}
            DexInsn::NewInstance(_) => {}
            DexInsn::NewArray(_) => {}
            DexInsn::FilledNewArray(_) => {}
            DexInsn::FilledNewArrayRange(_) => {}
            DexInsn::FillArrayData(_) => {}
            DexInsn::Throw(_) => {}
            DexInsn::Goto(_) => {}
            DexInsn::Goto16(_) => {}
            DexInsn::Goto32(_) => {}
            DexInsn::PackedSwitch(_) => {}
            DexInsn::SparseSwitch(_) => {}
            DexInsn::Cmpkind(_) => {}
            DexInsn::IfTest(_) => {}
            DexInsn::IfTestz(_) => {}
            DexInsn::ArrayOp(_) => {}
            DexInsn::IInstanceOp(_) => {}
            DexInsn::SInstanceOp(_) => {}
            DexInsn::InvokeKind(_) => {}
            DexInsn::InvokeKindRange(_) => {}
            DexInsn::Unop(_) => {}
            DexInsn::Binop(_) => {}
            DexInsn::Binop2Addr(_) => {}
            DexInsn::BinopLit16(_) => {}
            DexInsn::BinopLit8(_) => {}
            DexInsn::InvokePoly(_) => {}
            DexInsn::InvokePolyRange(_) => {}
            DexInsn::InvokeCustom(_) => {}
            DexInsn::InvokeCustomRange(_) => {}
            DexInsn::ConstMethodHandle(_) => {}
            DexInsn::ConstMethodType(_) => {}
            DexInsn::NotUsed(_) => {}
            DexInsn::PackedSwitchPayload(_) => {}
            DexInsn::SparseSwitchPayload(_) => {}
            DexInsn::FillArrayDataPayload(_) => {}
        }
    }
}
