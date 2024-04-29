use std::io::{BufReader, Read};

use crate::err::{AsmErr, AsmResult};
use crate::jvms::{AttributeInfo, ClassFile, CPInfo, FieldInfo, MethodInfo};
use crate::reader::bytes_reader::{FromReadContext, ReadContext, UReader};

struct JvmsClassReader {}

impl JvmsClassReader {
    fn from_readable<T: Read>(read: T) -> AsmResult<ClassFile> {
        let mut reader = BufReader::new(read);
        let mut str = String::new();
        let read_result = reader.read_to_string(&mut str);
        if let Err(e) = read_result {
            return Err(AsmErr::ContentReadErr { io_error: e });
        };
        let bytes = str.as_bytes();
        ClassFile::from_context(&(bytes, &mut 0))
    }
}


impl FromReadContext<ClassFile> for ClassFile {
    fn from_context(context: &ReadContext) -> AsmResult<ClassFile> {
        let magic: u32 = context.read()?;
        let minor_version: u16 = context.read()?;
        let major_version: u16 = context.read()?;
        let constant_pool_count: u16 = context.read()?;
        let constant_pool: Vec<CPInfo> = context.read_vec(constant_pool_count as usize)?;
        let access_flags: u16 = context.read()?;
        let this_class: u16 = context.read()?;
        let super_class: u16 = context.read()?;
        let interfaces_count: u16 = context.read()?;
        let interfaces: Vec<u16> = context.read_vec::<u16>(interfaces_count as usize)?;
        let fields_count: u16 = context.read()?;
        let fields: Vec<FieldInfo> = context.read_vec(fields_count as usize)?;
        let methods_count: u16 = context.read()?;
        let methods: Vec<MethodInfo> = context.read_vec(methods_count as usize)?;
        let attributes_count: u16 = context.read()?;
        let attributes: Vec<AttributeInfo> = context.read_vec(attributes_count as usize)?;
        let cf = ClassFile {
            magic, minor_version, major_version,
            constant_pool_count, constant_pool,
            access_flags, this_class, super_class,
            interfaces_count, interfaces,
            fields_count, fields,
            methods_count, methods,
            attributes_count, attributes,
        };
        Ok(cf)
    }
}

impl FromReadContext<CPInfo> for CPInfo {
    fn from_context(context: &ReadContext) -> AsmResult<CPInfo> {
        todo!()
    }
}

impl FromReadContext<FieldInfo> for FieldInfo {
    fn from_context(context: &ReadContext) -> AsmResult<FieldInfo> {
        todo!()
    }
}

impl FromReadContext<MethodInfo> for MethodInfo {
    fn from_context(context: &ReadContext) -> AsmResult<MethodInfo> {
        todo!()
    }
}

impl FromReadContext<AttributeInfo> for AttributeInfo {
    fn from_context(context: &ReadContext) -> AsmResult<AttributeInfo> {
        todo!()
    }
}

impl FromReadContext<u8> for u8 {
    fn from_context(context: &ReadContext) -> AsmResult<u8> {
        let (bytes, index) = *context;
        let content = bytes[*index];
        *index += 1;
        Ok(content)
    }
}

impl FromReadContext<u16> for u16 {
    fn from_context(context: &ReadContext) -> AsmResult<u16> {
        let (bytes, index) = *context;
        let h = (bytes[*index] as u16) << 8;
        let l = bytes[*index + 1] as u16;
        *index += 2;
        Ok(h | l)
    }
}

impl FromReadContext<u32> for u32 {
    fn from_context(context: &ReadContext) -> AsmResult<u32> {
        let (bytes, index) = *context;
        let a = (bytes[*index] as u32) << 24;
        let b = (bytes[*index + 1] as u32) << 16;
        let c = (bytes[*index + 2] as u32) << 8;
        let d = bytes[*index + 3] as u32;
        *index += 4;
        Ok(a | b | c | d)
    }
}


