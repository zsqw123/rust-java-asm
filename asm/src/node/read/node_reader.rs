use std::collections::HashMap;
use std::io::Read;
use std::rc::Rc;

use java_asm_internal::err::{AsmErr, AsmResult};

use crate::jvms::element::{ClassFile, Const, MethodInfo};
use crate::jvms::read::JvmsClassReader;
use crate::node::element::ClassNode;
use crate::node::read::impls::ClassNodeFactory;
use crate::node::values::{ConstValue, Descriptor};
use crate::util::mutf8_to_string;

pub struct NodeReader {}

impl NodeReader {
    pub fn read_class_file<T: Read>(read: T) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_file(read)?)
    }

    pub fn read_class_bytes(bytes: &[u8]) -> AsmResult<ClassNode> {
        Self::from_jvms(JvmsClassReader::read_class_bytes(bytes)?)
    }

    pub fn from_jvms(jvms_file: ClassFile) -> AsmResult<ClassNode> {
        ClassNodeFactory::from_jvms(jvms_file)
    }
}

pub(crate) struct ClassNodeContext {
    pub jvms_file: Rc<ClassFile>,
    cp_cache: HashMap<u16, Rc<ConstValue>>,
}

impl ClassNodeContext {
    pub fn read_utf8(&mut self, index: u16) -> AsmResult<Rc<String>> {
        let constant = self.read_const(index)?;
        let ConstValue::String(s) = constant.as_ref() else {
            return AsmErr::IllegalArgument(
                format!("cannot read utf8 from constant pool, cp_index: {}, constant: {:?}", index, constant)
            ).e();
        };
        Ok(Rc::clone(s))
    }

    pub fn read_class_info(&mut self, index: u16) -> AsmResult<Rc<String>> {
        let constant = self.read_const(index)?;
        let ConstValue::Class(name) = constant.as_ref() else {
            return AsmErr::IllegalArgument(
                format!("cannot read class info from constant pool, cp_index: {}, constant: {:?}", index, constant)
            ).e();
        };
        Ok(Rc::clone(name))
    }

    pub fn read_name_and_type(&mut self, index: u16) -> AsmResult<(Rc<String>, Rc<Descriptor>)> {
        let constant = self.read_const(index)?;
        let ConstValue::NameAndType { name, desc } = constant.as_ref() else {
            return AsmErr::IllegalArgument(
                format!("cannot read name and type from constant pool, cp_index: {}, constant: {:?}", index, constant)
            ).e();
        };
        Ok((Rc::clone(name), Rc::clone(desc)))
    }

    pub fn read_const(&mut self, index: u16) -> AsmResult<Rc<ConstValue>> {
        if let Some(constant) = self.read_const_cache(index) {
            return Ok(constant);
        }
        let raw_const = &self.jvms_file.constant_pool[index as usize].info;
        let const_value = match raw_const {
            Const::Invalid => { ConstValue::Invalid },
            Const::Class { name_index } => {
                ConstValue::Class(self.read_utf8(*name_index)?)
            },
            Const::Field { class_index, name_and_type_index }
            | Const::Method { class_index, name_and_type_index }
            | Const::InterfaceMethod { class_index, name_and_type_index } => {
                let class = self.read_class_info(*class_index)?;
                let (name, desc) = self.read_name_and_type(*name_and_type_index)?;
                ConstValue::Member { class, name, desc }
            },
            Const::String { string_index } => {
                ConstValue::String(self.read_utf8(*string_index)?)
            },
            Const::Integer { bytes } => ConstValue::Integer(*bytes as i32),
            Const::Float { bytes } => ConstValue::Float(f32::from_bits(*bytes)),
            Const::Long { high_bytes, low_bytes } => {
                let value = ((*high_bytes as u64) << 32) | (*low_bytes as u64);
                ConstValue::Long(value as i64)
            },
            Const::Double { high_bytes, low_bytes } => {
                let value = ((*high_bytes as u64) << 32) | (*low_bytes as u64);
                ConstValue::Double(f64::from_bits(value))
            },
            Const::NameAndType { name_index, descriptor_index } => {
                let name = self.read_utf8(*name_index)?;
                let desc = self.read_utf8(*descriptor_index)?;
                ConstValue::NameAndType { name, desc }
            },
            Const::Utf8 { bytes, .. } => {
                ConstValue::String(Rc::new(mutf8_to_string(bytes)?))
            },
            Const::MethodHandle { reference_kind, reference_index } => {
                ConstValue::MethodHandle { reference_kind: *reference_kind, reference_index: *reference_index }
            },
            Const::MethodType { descriptor_index } => {
                ConstValue::MethodType(self.read_utf8(*descriptor_index)?)
            },
            Const::Dynamic { bootstrap_method_attr_index, name_and_type_index }
            | Const::InvokeDynamic { bootstrap_method_attr_index, name_and_type_index } => {
                let (name, desc) = self.read_name_and_type(*name_and_type_index)?;
                ConstValue::Dynamic { bootstrap_method_attr_index: *bootstrap_method_attr_index, name, desc }
            },
            Const::Module { name_index } => ConstValue::Module(self.read_utf8(*name_index)?),
            Const::Package { name_index } => ConstValue::Package(self.read_utf8(*name_index)?),
        };
        let const_value = Rc::new(const_value);
        self.put_const_cache(index, Rc::clone(&const_value));
        return Ok(const_value)
    }

    fn read_const_cache(&self, index: u16) -> Option<Rc<ConstValue>> {
        self.cp_cache.get(&index).map(|c| Rc::clone(c))
    }

    fn put_const_cache(&mut self, index: u16, constant: Rc<ConstValue>) {
        self.cp_cache.insert(index, constant);
    }
}

pub(crate) struct MethodNodeContext<'a> {
    pub jvms_file: &'a ClassFile,
    pub method_info: &'a MethodInfo,
}
