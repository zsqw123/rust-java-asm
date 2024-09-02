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
    
    pub fn read_code(&mut self, code: Vec<u8>) -> AsmResult<Vec<InsnNode>> {
        let mut cur = 0usize;

        let const_index = |high_index: usize| -> u16 {
            let high = (code[high_index] as u16) << 8;
            let low = code[high_index + 1] as u16;
            high | low
        };

        let mut res = vec![];
        while cur < code.len() {
            let opcode = code[cur];
            match opcode {
                Opcodes::GETSTATIC | Opcodes::PUTSTATIC | Opcodes::GETFIELD | Opcodes::PUTFIELD => {
                    let (owner, name, desc) = self.read_member(const_index(cur + 1))?;
                    res.push(FieldInsnNode { opcode, owner, name, desc });
                    cur += 3;
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


