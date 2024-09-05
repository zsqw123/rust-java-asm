use std::ops::Deref;
use std::rc::Rc;

use java_asm_internal::err::{AsmErr, AsmResult, AsmResultRcExt};

use crate::constants::Constants;
use crate::impls::computable::{CacheableOwner, CacheAccessor};
use crate::impls::jvms::r::util::ToRcRef;
use crate::impls::node::r::node_reader::{ClassNodeContext, ConstComputableMap, CpCache};
use crate::jvms::element::Const;
use crate::node::values::{ConstValue, DescriptorRef, StrRef};
use crate::util::mutf8_to_string;

impl CacheableOwner<u16, ConstValue, AsmErr> for CpCache {
    fn cache_map(&self) -> &ConstComputableMap {
        &self.pool
    }

    fn compute(&self, key: &u16) -> AsmResult<ConstValue> {
        self.read_const(*key)
    }
}

macro_rules! read_const {
    {
        $($name:ident -> $ret:ty {
            $variant:ident($($arg:ident),*)
        })*
    } => {
        $(pub fn $name(&self, index: u16) -> AsmResult<$ret> {
            let constant = self.get(index)?;
            let ConstValue::$variant( $($arg),* ) = constant.as_ref() else {
                return AsmErr::IllegalArgument(
                    format!("cannot read const value from constant pool, cp_index: {}, constant: {:?}, required: ConstValue::{}",
                        index, constant, stringify!($variant))
                ).e();
            };
            Ok(($(Rc::clone($arg)),*))
        })*
    };
}

macro_rules! read_const_curly {
    {
        $($name:ident -> $ret:ty {
            $variant:ident { $($arg:ident),* }
        })*
    } => {
        $(pub fn $name(&self, index: u16) -> AsmResult<$ret> {
            let constant = self.get(index)?;
            let ConstValue::$variant{ $($arg),* } = constant.as_ref() else {
                return AsmErr::IllegalArgument(
                    format!("cannot read const value from constant pool, cp_index: {}, constant: {:?}, required: ConstValue::{}",
                        index, constant, stringify!($variant))
                ).e();
            };
            Ok(($($arg.clone()),*))
        })*
    };
}

/// impls for const reads
impl CpCache {
    pub fn name(&self) -> AsmResult<StrRef> {
        self.read_class_info(self.jvms_file.this_class)
    }

    read_const! {
        read_class_info -> StrRef { Class(name) }
        read_utf8 -> StrRef { String(s) }
        read_module -> StrRef { Module(s) }
        read_package -> StrRef { Package(s) }
    }

    read_const_curly! {
        read_name_and_type -> (StrRef, DescriptorRef) {
            NameAndType { name, desc }
        }
        read_member -> (StrRef, StrRef, DescriptorRef) {
            Member { class, name, desc }
        }
        read_dynamic -> (u16, StrRef, DescriptorRef) {
            Dynamic { bootstrap_method_attr_index, name, desc }
        }
    }

    #[inline]
    pub fn read_class_info_or_default(&self, index: u16) -> StrRef {
        self.read_class_info(index)
            .unwrap_or_else(|_| Constants::OBJECT_INTERNAL_NAME.as_rc())
    }

    pub fn get(&self, index: u16) -> AsmResult<Rc<ConstValue>> {
        self.get_with_ref(&index).clone_if_error()
    }

    fn read_const(&self, index: u16) -> AsmResult<ConstValue> {
        let raw_const = self.jvms_file.constant_pool[index as usize].info.clone();
        let const_value = match raw_const {
            Const::Invalid => { ConstValue::Invalid }
            Const::Class { name_index } => {
                ConstValue::Class(self.read_utf8(name_index)?)
            }
            Const::Field { class_index, name_and_type_index }
            | Const::Method { class_index, name_and_type_index }
            | Const::InterfaceMethod { class_index, name_and_type_index } => {
                let class = self.read_class_info(class_index)?;
                let (name, desc) = self.read_name_and_type(name_and_type_index)?;
                ConstValue::Member { class, name, desc }
            }
            Const::String { string_index } => {
                ConstValue::String(self.read_utf8(string_index)?)
            }
            Const::Integer { bytes } => ConstValue::Integer(bytes as i32),
            Const::Float { bytes } => ConstValue::Float(f32::from_bits(bytes)),
            Const::Long { high_bytes, low_bytes } => {
                let value = ((high_bytes as u64) << 32) | (low_bytes as u64);
                ConstValue::Long(value as i64)
            }
            Const::Double { high_bytes, low_bytes } => {
                let value = ((high_bytes as u64) << 32) | (low_bytes as u64);
                ConstValue::Double(f64::from_bits(value))
            }
            Const::NameAndType { name_index, descriptor_index } => {
                let name = self.read_utf8(name_index)?;
                let desc = self.read_utf8(descriptor_index)?;
                ConstValue::NameAndType { name, desc }
            }
            Const::Utf8 { bytes, .. } => {
                ConstValue::String(mutf8_to_string(&bytes)?)
            }
            Const::MethodHandle { reference_kind, reference_index } => {
                ConstValue::MethodHandle { reference_kind, reference_index }
            }
            Const::MethodType { descriptor_index } => {
                ConstValue::MethodType(self.read_utf8(descriptor_index)?)
            }
            Const::Dynamic { bootstrap_method_attr_index, name_and_type_index }
            | Const::InvokeDynamic { bootstrap_method_attr_index, name_and_type_index } => {
                let (name, desc) = self.read_name_and_type(name_and_type_index)?;
                ConstValue::Dynamic { bootstrap_method_attr_index, name, desc }
            }
            Const::Module { name_index } => ConstValue::Module(self.read_utf8(name_index)?),
            Const::Package { name_index } => ConstValue::Package(self.read_utf8(name_index)?),
        };
        Ok(const_value)
    }
}

impl<T> Deref for ClassNodeContext<T> {
    type Target = CpCache;
    fn deref(&self) -> &CpCache { &self.cp_cache }
}