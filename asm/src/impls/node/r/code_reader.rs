use std::collections::HashMap;

use crate::err::{AsmErr, AsmResult, AsmResultExt};
use crate::impls::node::r::node_reader::ClassNodeContext;
use crate::impls::node::r::util::{once_vec_builder, once_vec_unpack};
use crate::impls::node::r::util::OnceAsmVec;
use crate::jvms::attr::StackMapFrame;
use crate::node::element::{Attribute, CodeAttribute, CodeBodyNode, LocalVariableNode, TypeAnnotationNode};
use crate::node::insn::InsnNode;
use crate::node::insn::InsnNode::FieldInsnNode;
use crate::node::values::{BootstrapMethodArgument, ConstDynamic, ConstValue, LocalVariableInfo, LocalVariableTypeInfo};
use crate::opcodes::Opcodes;
use crate::util::VecEx;

impl ClassNodeContext {
    pub fn read_code_body(&self, code_attr: CodeAttribute) -> AsmResult<CodeBodyNode> {
        let CodeAttribute { max_stack, max_locals, code, exception_table, attributes } = code_attr;
        let instructions = self.read_code(code)?;

        once_vec_builder! {
            let local_variable_infos: LocalVariableInfo;
            let local_variable_type_infos: LocalVariableTypeInfo;
            let type_annotations: TypeAnnotationNode;
            let stack_map_table: StackMapFrame;
        }

        let mut unknown_attributes = vec![];

        for (attr_info, attr) in attributes {
            match attr {
                Attribute::LocalVariableTable(lv) => local_variable_infos.put(lv)?,
                Attribute::LocalVariableTypeTable(lv) => local_variable_type_infos.put(lv)?,
                Attribute::RuntimeInvisibleTypeAnnotations(ta) => type_annotations.put(ta)?,
                Attribute::RuntimeVisibleTypeAnnotations(ta) => type_annotations.put(ta)?,
                Attribute::StackMapTable(table) => stack_map_table.put(table)?,
                Attribute::Unknown(a) => unknown_attributes.push(a),
                _ => unknown_attributes.push(self.unknown_attr(attr_info)?),
            }
        }

        once_vec_unpack!(local_variable_infos, local_variable_type_infos, type_annotations, stack_map_table);

        let local_variables = merge_local_variables(
            local_variable_infos, local_variable_type_infos,
        );

        Ok(CodeBodyNode {
            instructions,
            exception_table,
            local_variables,
            max_stack,
            max_locals,
            type_annotations,
            stack_map_table,
            unknown_attributes,
        })
    }

    //noinspection SpellCheckingInspection
    pub fn read_code(&self, code: Vec<u8>) -> AsmResult<Vec<InsnNode>> {
        let mut cur = 0usize;

        // read a 16bit const from index, in jvm bytecode, it stores high byte first (big-endian)
        let read_u16 = |high_index: usize| -> u16 {
            let high = (code[high_index] as u16) << 8;
            let low = code[high_index + 1] as u16;
            high | low
        };

        let read_i16 = |high_index: usize| -> i16 {
            let high = (code[high_index] as u16) << 8;
            let low = code[high_index + 1] as u16;
            (high | low) as i16
        };

        let read_i32 = |high_index: usize| -> i32 {
            let a = (code[high_index] as u32) << 24;
            let b = (code[high_index + 1] as u32) << 16;
            let c = (code[high_index + 2] as u32) << 8;
            let d = code[high_index + 3] as u32;
            (a | b | c | d) as i32
        };

        let mut res = vec![];
        while cur < code.len() {
            let opcode = code[cur];
            match opcode {
                // getstatic | indexbyte1 | indexbyte2
                Opcodes::GETSTATIC | Opcodes::PUTSTATIC | Opcodes::GETFIELD | Opcodes::PUTFIELD => {
                    let (owner, name, desc) = self.read_member(read_u16(cur + 1))?;
                    res.push(FieldInsnNode { opcode, owner, name, desc });
                    cur += 3;
                }
                // iinc | index | const
                Opcodes::IINC => {
                    let var = code[cur + 1] as u16;
                    let incr = code[cur + 2] as i8 as i16;
                    res.push(InsnNode::IIncInsnNode { var, incr });
                    cur += 3;
                }
                // const
                Opcodes::ACONST_NULL | Opcodes::ICONST_M1 | Opcodes::ICONST_0 | Opcodes::ICONST_1 |
                Opcodes::ICONST_2 | Opcodes::ICONST_3 | Opcodes::ICONST_4 | Opcodes::ICONST_5 |
                Opcodes::LCONST_0 | Opcodes::LCONST_1 | Opcodes::FCONST_0 | Opcodes::FCONST_1 |
                Opcodes::FCONST_2 | Opcodes::DCONST_0 | Opcodes::DCONST_1 |
                // load
                Opcodes::IALOAD | Opcodes::LALOAD | Opcodes::FALOAD | Opcodes::DALOAD | Opcodes::AALOAD |
                Opcodes::BALOAD | Opcodes::CALOAD | Opcodes::SALOAD |
                // store
                Opcodes::IASTORE | Opcodes::LASTORE | Opcodes::FASTORE | Opcodes::DASTORE |
                Opcodes::AASTORE | Opcodes::BASTORE | Opcodes::CASTORE | Opcodes::SASTORE |
                // stack operations
                Opcodes::POP | Opcodes::POP2 | Opcodes::DUP | Opcodes::DUP_X1 | Opcodes::DUP_X2 |
                Opcodes::DUP2 | Opcodes::DUP2_X1 | Opcodes::DUP2_X2 | Opcodes::SWAP |
                // math calculations
                Opcodes::IADD | Opcodes::LADD | Opcodes::FADD | Opcodes::DADD | Opcodes::ISUB |
                Opcodes::LSUB | Opcodes::FSUB | Opcodes::DSUB | Opcodes::IMUL | Opcodes::LMUL |
                Opcodes::FMUL | Opcodes::DMUL | Opcodes::IDIV | Opcodes::LDIV | Opcodes::FDIV |
                Opcodes::DDIV | Opcodes::IREM | Opcodes::LREM | Opcodes::FREM | Opcodes::DREM |
                Opcodes::INEG | Opcodes::LNEG | Opcodes::FNEG | Opcodes::DNEG |
                // bit operations
                Opcodes::ISHL | Opcodes::LSHL | Opcodes::ISHR | Opcodes::LSHR | Opcodes::IUSHR |
                Opcodes::LUSHR | Opcodes::IAND | Opcodes::LAND | Opcodes::IOR | Opcodes::LOR |
                Opcodes::IXOR | Opcodes::LXOR |
                // type conversions
                Opcodes::I2L | Opcodes::I2F | Opcodes::I2D | Opcodes::L2I | Opcodes::L2F |
                Opcodes::L2D | Opcodes::F2I | Opcodes::F2L | Opcodes::F2D | Opcodes::D2I |
                Opcodes::D2L | Opcodes::D2F | Opcodes::I2B | Opcodes::I2C | Opcodes::I2S |
                // comparison
                Opcodes::LCMP | Opcodes::FCMPL | Opcodes::FCMPG | Opcodes::DCMPL | Opcodes::DCMPG |
                // returns
                Opcodes::IRETURN | Opcodes::LRETURN | Opcodes::FRETURN | Opcodes::DRETURN |
                Opcodes::ARETURN | Opcodes::RETURN |
                // others
                Opcodes::ARRAYLENGTH | Opcodes::ATHROW |
                // synchronization monitor
                Opcodes::MONITORENTER | Opcodes::MONITOREXIT => {
                    res.push(InsnNode::NoOperand { opcode });
                    cur += 1;
                }
                // bipush | byte
                Opcodes::BIPUSH => {
                    let operand = code[cur + 1] as i8;
                    res.push(InsnNode::BIPushInsnNode { operand });
                    cur += 2;
                }
                // sipush | byte1 | byte2
                Opcodes::SIPUSH => {
                    let operand = read_u16(cur + 1) as i16;
                    res.push(InsnNode::SIPushInsnNode { operand });
                    cur += 3;
                }
                // newarray | atype
                Opcodes::NEWARRAY => {
                    let array_type = code[cur + 1];
                    res.push(InsnNode::NewArrayInsnNode { array_type });
                    cur += 2;
                }
                // invokedynamic | indexbyte1 | indexbyte2 | 0 | 0
                Opcodes::INVOKEDYNAMIC => {
                    let (bootstrap_method_attr_index, name, desc) =
                        self.read_dynamic(read_u16(cur + 1))?;
                    let bsm_attr = self.require_bms().get(bootstrap_method_attr_index as usize).ok_or_error(|| {
                        let error_message = format!("cannot find bootstrap method attribute at index: {}", bootstrap_method_attr_index);
                        Err(self.err(error_message))
                    })?;
                    let bsm_args = bsm_attr.arguments.map_res(|arg|
                        const_to_bsm_arg((**arg).clone())
                    )?;
                    let bm_handle = &bsm_attr.method_handle;
                    let ConstValue::MethodHandle(handle) = (*bm_handle).as_ref() else {
                        let err_msg = "MethodHandle in BootstrapMethodAttr must be a MethodHandle";
                        AsmErr::IllegalArgument(err_msg.to_string()).e()?
                    };
                    let bsm = handle.clone();
                    let const_dynamic = ConstDynamic { name, desc, bsm, bsm_args };
                    res.push(InsnNode::InvokeDynamicInsnNode(const_dynamic));
                    cur += 5;
                }
                // if<cond> | branchbyte1 | branchbyte2
                Opcodes::IFLT | Opcodes::IFGE | Opcodes::IFGT | Opcodes::IFLE |
                Opcodes::IF_ICMPEQ | Opcodes::IF_ICMPNE | Opcodes::IF_ICMPLT | Opcodes::IF_ICMPGE |
                Opcodes::IF_ICMPGT | Opcodes::IF_ICMPLE | Opcodes::IF_ACMPEQ | Opcodes::IF_ACMPNE |
                Opcodes::GOTO | Opcodes::JSR | Opcodes::IFNULL | Opcodes::IFNONNULL => {
                    let offset = read_i16(cur + 1) as i32;
                    let label = ((cur as i32) + offset) as u16;
                    res.push(InsnNode::JumpInsnNode { opcode, label });
                    cur += 3;
                }
                // ldc | index
                Opcodes::LDC => {
                    let const_value = self.cp.get_res(code[cur + 1] as u16)?;
                    res.push(InsnNode::LdcInsnNode(const_value));
                    cur += 2;
                }
                // lookupswitch | <0-3 bytes padding> |
                // defaultbyte1 | defaultbyte2 | defaultbyte3 | defaultbyte4 |
                // npairs1 | npairs2 | npairs3 | npairs4 |
                // match-offset pairs...
                Opcodes::LOOKUPSWITCH => {
                    let lookup_start = cur as u16;
                    cur += 1;
                    let df_start = cur + 4 - (cur & 3);
                    let default = read_i32(df_start) as u16;
                    let npairs = read_i32(df_start + 4);
                    let mut keys = vec![];
                    let mut labels = vec![];
                    cur += 8;
                    for _ in 0..npairs {
                        keys.push(read_i32(cur));
                        let offset = read_i32(cur + 4) as u16;
                        labels.push(lookup_start + offset);
                        cur += 8;
                    }
                    res.push(InsnNode::LookupSwitchInsnNode { default, keys, labels });
                }
                // invoke | indexbyte1 | indexbyte2
                Opcodes::INVOKEVIRTUAL | Opcodes::INVOKESPECIAL |
                Opcodes::INVOKESTATIC | Opcodes::INVOKEINTERFACE => {
                    let (owner, name, desc) = self.read_member(read_u16(cur + 1))?;
                    res.push(InsnNode::MethodInsnNode { opcode, owner, name, desc });
                    cur += 3;
                }
                // multianewarray | indexbyte1 | indexbyte2 | dimensions
                Opcodes::MULTIANEWARRAY => {
                    let array_type = self.read_class_info(read_u16(cur + 1))?;
                    let dims = code[cur + 3];
                    res.push(InsnNode::MultiANewArrayInsnNode { array_type, dims });
                    cur += 4;
                }
                // tableswitch | <0-3 bytes padding> |
                // defaultbyte1 | defaultbyte2 | defaultbyte3 | defaultbyte4 |
                // lowbyte1 | lowbyte2 | lowbyte3 | lowbyte4 |
                // highbyte1 | highbyte2 | highbyte3 | highbyte4 |
                // jump offsets...
                Opcodes::TABLESWITCH => {
                    let table_start = cur as u16;
                    cur += 1;
                    let df_start = cur + 4 - (cur & 3);
                    let default = read_i32(df_start) as u16;
                    let min = read_i32(df_start + 4);
                    let max = read_i32(df_start + 8);
                    let mut labels = vec![];
                    cur += 12;
                    for _ in min..=max {
                        let offset = read_i32(cur) as u16;
                        labels.push(table_start + offset);
                        cur += 4;
                    }
                    res.push(InsnNode::TableSwitchInsnNode { default, min, max, labels });
                }
                // instanceof | indexbyte1 | indexbyte2
                Opcodes::NEW | Opcodes::ANEWARRAY | Opcodes::CHECKCAST | Opcodes::INSTANCEOF => {
                    let type_name = self.read_class_info(read_u16(cur + 1))?;
                    res.push(InsnNode::TypeInsnNode { opcode, type_name });
                    cur += 3;
                }
                // iload | index
                Opcodes::ILOAD | Opcodes::LLOAD | Opcodes::FLOAD | Opcodes::DLOAD | Opcodes::ALOAD |
                Opcodes::ISTORE | Opcodes::LSTORE | Opcodes::FSTORE | Opcodes::DSTORE | Opcodes::ASTORE => {
                    let var_index = code[cur + 1] as u16;
                    res.push(InsnNode::VarInsnNode { opcode, var_index });
                    cur += 2;
                }
                // iload_<n> | fload_<n> | aload_<n> | istore_<n> | fstore_<n> | astore_<n>
                Opcodes::ILOAD_0..=Opcodes::ALOAD_3 | Opcodes::ISTORE_0..=Opcodes::ASTORE_3 => {
                    res.push(InsnNode::NoOperand { opcode });
                    cur += 1;
                }
                // wide
                Opcodes::WIDE => {
                    let opcode = code[cur + 1];
                    if opcode == Opcodes::IINC {
                        // wide | iinc | indexbyte1 | indexbyte2 | constbyte1 | constbyte2
                        let var = read_u16(cur + 2);
                        let incr = read_i16(cur + 4);
                        res.push(InsnNode::IIncInsnNode { var, incr });
                        cur += 6;
                    } else {
                        // wide | opcode | indexbyte1 | indexbyte2
                        let var_index = read_u16(cur + 2);
                        res.push(InsnNode::VarInsnNode { opcode, var_index });
                        cur += 4;
                    }
                }
                // goto_w | branchbyte1 | branchbyte2 | branchbyte3 | branchbyte4
                Opcodes::GOTO_W | Opcodes::JSR_W => {
                    let offset = read_i32(cur + 1);
                    let label = ((cur as i32) + offset) as u16;
                    res.push(InsnNode::JumpInsnNode { opcode, label });
                    cur += 5;
                }
                // ldc_w | indexbyte1 | indexbyte2
                Opcodes::LDC_W | Opcodes::LDC2_W => {
                    let const_value = self.cp.get_res(read_u16(cur + 1))?;
                    res.push(InsnNode::LdcInsnNode(const_value));
                    cur += 3;
                }
                _ => {
                    return AsmErr::UnknownInsn(opcode).e();
                }
            }
        }
        Ok(res)
    }
}

fn const_to_bsm_arg(c: ConstValue) -> AsmResult<BootstrapMethodArgument> {
    match c {
        ConstValue::Integer(i) => Ok(BootstrapMethodArgument::Integer(i)),
        ConstValue::Float(f) => Ok(BootstrapMethodArgument::Float(f)),
        ConstValue::Long(l) => Ok(BootstrapMethodArgument::Long(l)),
        ConstValue::Double(d) => Ok(BootstrapMethodArgument::Double(d)),
        ConstValue::String(s) => Ok(BootstrapMethodArgument::String(s)),
        ConstValue::Class(t) => Ok(BootstrapMethodArgument::Class(t)),
        ConstValue::MethodHandle(h) => Ok(BootstrapMethodArgument::Handle(h)),
        _ => {
            let err_msg = format!("cannot convert correspond const value to bootstrap method argument: {:?}", c);
            Err(AsmErr::IllegalArgument(err_msg))
        },
    }
}

fn merge_local_variables(
    infos: Vec<LocalVariableInfo>,
    type_infos: Vec<LocalVariableTypeInfo>,
) -> Vec<LocalVariableNode> {
    let mut local_variables = vec![];
    let mut type_map = HashMap::with_capacity(type_infos.len());
    for info in type_infos {
        let LocalVariableTypeInfo { start, length, signature, index, .. } = info;
        type_map.insert((start, length, index), signature);
    }

    for info in infos {
        let LocalVariableInfo { name, desc, start, length, index } = info;
        let signature = type_map.get(&(start, length, index)).cloned();
        local_variables.push(LocalVariableNode {
            name, desc, signature,
            start, end: start + length, index,
        });
    }
    local_variables
}

