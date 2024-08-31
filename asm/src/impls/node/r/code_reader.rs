use java_asm_internal::err::AsmResult;

use crate::impls::node::r::node_reader::CpCache;
use crate::node::insn::InsnNode;
use crate::node::insn::InsnNode::FieldInsnNode;
use crate::node::values::FrameAttributeValue;
use crate::opcodes::Opcodes;

impl CpCache {
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


