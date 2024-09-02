use std::collections::HashMap;

use java_asm_internal::err::AsmResult;

use crate::impls::node::r::node_reader::CpCache;
use crate::node::element::{Attribute, CodeAttribute, CodeBodyNode, LocalVariableNode};
use crate::node::insn::InsnNode;
use crate::node::insn::InsnNode::FieldInsnNode;
use crate::node::values::{FrameAttributeValue, LocalVariableInfo, LocalVariableTypeInfo};
use crate::opcodes::Opcodes;

impl CpCache {
    pub fn read_code_body(&mut self, code_attr: CodeAttribute) -> AsmResult<CodeBodyNode> {
        let CodeAttribute { max_stack, max_locals, code, exception_table, attributes } = code_attr;
        let instructions = self.read_code(code)?;

        let mut local_variable_infos = vec![];
        let mut local_variable_type_infos = vec![];

        let mut type_annotations = vec![];
        let mut unknown_attributes = vec![];

        for (attr_info, attr) in attributes {
            match attr {
                Attribute::LocalVariableTable(lv) => local_variable_infos = lv,
                Attribute::LocalVariableTypeTable(lv) => local_variable_type_infos = lv,
                Attribute::RuntimeInvisibleTypeAnnotations(ta) => type_annotations.extend(ta),
                Attribute::RuntimeVisibleTypeAnnotations(ta) => type_annotations.extend(ta),
                Attribute::Unknown(a) => unknown_attributes.push(a),
                _ => unknown_attributes.push(self.unknown_attr(attr_info)?),
            }
        }

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
            attrs: unknown_attributes,
        })
    }

    //noinspection SpellCheckingInspection
    pub fn read_code(&mut self, code: Vec<u8>) -> AsmResult<Vec<InsnNode>> {
        let mut cur = 0usize;

        // read a 16bit const from index, in jvm bytecode, it stores high byte first (big-endian)
        let const_from_index = |high_index: usize| -> u16 {
            let high = (code[high_index] as u16) << 8;
            let low = code[high_index + 1] as u16;
            high | low
        };

        let mut res = vec![];
        while cur < code.len() {
            let opcode = code[cur];
            match opcode {
                // getstatic | indexbyte1 | indexbyte2
                Opcodes::GETSTATIC | Opcodes::PUTSTATIC | Opcodes::GETFIELD | Opcodes::PUTFIELD => {
                    let (owner, name, desc) = self.read_member(const_from_index(cur + 1))?;
                    res.push(FieldInsnNode { opcode, owner, name, desc });
                    cur += 3;
                }
                // iinc | index | const
                Opcodes::IINC => {
                    let var = code[cur + 1];
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
                    let operand = const_from_index(cur + 1) as i16;
                    res.push(InsnNode::SIPushInsnNode { operand });
                    cur += 3;
                }
                // newarray | atype
                Opcodes::NEWARRAY => {
                    let array_type = code[cur + 1];
                    res.push(InsnNode::NewArrayInsnNode { array_type });
                    cur += 2;
                }
                
                _ => {}
            }
        }
        Ok(vec![])
    }
    
    pub fn read_frames(&mut self, code: Vec<u8>) -> AsmResult<Vec<FrameAttributeValue>> {
        // let 
        Ok(vec![])
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


