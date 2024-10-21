use crate::impls::ToStringRef;
use crate::node::values::{BootstrapMethodArgument, ConstDynamic, ConstValue, Handle};
use crate::node::InsnNode;
use crate::smali::{SmaliNode, ToSmali};
use crate::{ConstContainer, MethodHandleKind, NewArrayTypeOperand, Opcodes, StrRef};
use std::fmt::{Display, Formatter};
use std::rc::Rc;

fn insn_name(opcode: &u8) -> String {
    Opcodes::const_name_or_default(*opcode, "insn")
}

impl ToSmali for InsnNode {
    fn to_smali(&self) -> SmaliNode {
        match self {
            InsnNode::FieldInsnNode {
                opcode, owner, name, desc
            } => SmaliNode::new(format!("{} {owner}.{name} {desc}", insn_name(opcode))),
            InsnNode::IIncInsnNode { var, incr } =>
                SmaliNode::new(format!("iinc {var} {incr}")),
            InsnNode::NoOperand { opcode } =>
                SmaliNode::new(format!("{}", insn_name(opcode))),
            InsnNode::BIPushInsnNode { operand } =>
                SmaliNode::new(format!("bipush {operand}")),
            InsnNode::SIPushInsnNode { operand } =>
                SmaliNode::new(format!("sipush {operand}")),
            InsnNode::InvokeDynamicInsnNode(const_dynamic) => const_dynamic.to_smali(),
            InsnNode::JumpInsnNode { opcode, label } =>
                SmaliNode::new(format!("{} {label}", insn_name(opcode))),
            InsnNode::LdcInsnNode(constant) => {
                let constant_smali = constant.to_smali();
                SmaliNode::new_with_children(
                    format!("ldc {}", constant_smali.prefix), constant_smali.children,
                )
            }
            InsnNode::TableSwitchInsnNode { default, min, max, labels } => {
                let current = format!("tableswitch {default} {min} {max}");
                let children = labels.iter().map(|label| label.to_smali()).collect();
                SmaliNode::new_with_children(current, children)
            }
            InsnNode::LookupSwitchInsnNode { default, keys, labels } => {
                let current = format!("lookupswitch {default}");
                let children = keys.iter().zip(labels.iter())
                    .map(|(key, label)| {
                        SmaliNode::new(format!("{} -> {}", key, label))
                    }).collect();
                SmaliNode::new_with_children(current, children)
            }
            InsnNode::MethodInsnNode { opcode, owner, name, desc } => {
                let opcode_name = insn_name(opcode);
                let raw = format!("{opcode_name} {owner} {name} {desc}");
                SmaliNode::new(raw)
            }
            InsnNode::NewArrayInsnNode { array_type } => {
                let array_type = NewArrayTypeOperand::const_name_or_default(*array_type, "array");
                SmaliNode::new(format!("newarray {}", array_type))
            }
            InsnNode::MultiANewArrayInsnNode { array_type, dims } =>
                SmaliNode::new(format!("multianewarray dim_{} {}", dims, array_type)),
            InsnNode::TypeInsnNode { opcode, type_name } =>
                SmaliNode::new(format!("{} {}", insn_name(opcode), type_name)),
            InsnNode::VarInsnNode { opcode, var_index } =>
                SmaliNode::new(format!("{} {}", insn_name(opcode), var_index)),
        }
    }
}

impl ToSmali for ConstValue {
    fn to_smali(&self) -> SmaliNode {
        match self {
            ConstValue::Invalid => SmaliNode::new("invalid_const".to_ref()),
            ConstValue::Class(v) => v.to_smali(),
            ConstValue::Member { class, name, desc } =>
                SmaliNode::new(format!("member {class} {name} {desc}")),
            ConstValue::String(v) => v.to_smali(),
            ConstValue::Integer(v) => v.to_smali(),
            ConstValue::Float(v) => v.to_smali(),
            ConstValue::Long(v) => v.to_smali(),
            ConstValue::Double(v) => v.to_smali(),
            ConstValue::NameAndType { name, desc } =>
                SmaliNode::new(format!("{name}: {desc}")),
            ConstValue::MethodHandle(v) => v.to_smali(),
            ConstValue::MethodType(v) => v.to_smali(),
            ConstValue::Dynamic { bootstrap_method_attr_index, name, desc } =>
                SmaliNode::new(format!("dynamic {bootstrap_method_attr_index} {name} {desc}")),
            ConstValue::Module(v) => v.to_smali(),
            ConstValue::Package(v) => v.to_smali(),
        }
    }
}

impl ToSmali for ConstDynamic {
    fn to_smali(&self) -> SmaliNode {
        let ConstDynamic { name, desc, bsm, bsm_args } = self;
        let prefix = format!("invoke-dynamic {name} {desc} {bsm}");
        let children = bsm_args.iter().map(|arg| arg.to_smali()).collect();
        SmaliNode::new_with_children(prefix, children)
    }
}

impl ToStringRef for Handle {
    fn to_ref(&self) -> StrRef {
        let ref_kind_name = MethodHandleKind::const_name_or_default(self.reference_kind, "ref");
        let owner = &self.owner;
        let name = &self.name;
        let desc = &self.desc;
        Rc::from(format!("Handle({ref_kind_name}, {owner}, {name}, {desc})"))
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_ref())
    }
}

impl ToSmali for BootstrapMethodArgument {
    fn to_smali(&self) -> SmaliNode {
        match self {
            BootstrapMethodArgument::Integer(v) => SmaliNode::new(*v),
            BootstrapMethodArgument::Float(v) => SmaliNode::new(*v),
            BootstrapMethodArgument::Long(v) => SmaliNode::new(*v),
            BootstrapMethodArgument::Double(v) => SmaliNode::new(*v),
            BootstrapMethodArgument::String(v) => SmaliNode::new(v),
            BootstrapMethodArgument::Class(v) => SmaliNode::new(v),
            BootstrapMethodArgument::Handle(v) => SmaliNode::new(v),
        }
    }
}
