use std::io::{BufWriter, Write};

use java_asm_internal::err::{AsmErr, AsmResult};
use crate::jvms::element::ClassFile;
use crate::jvms::write::bytes::WriteContext;

mod jvms_writer;
mod bytes;
mod attrs;

pub struct JvmsClassWriter {}

impl JvmsClassWriter {
    pub fn write_class_file<T: Write>(write: T, class_file: ClassFile) -> AsmResult<()> {
        let mut writer = BufWriter::new(write);
        let bytes = Self::write_class_bytes(vec![], class_file)?;
        match writer.write(bytes.as_slice()) {
            Ok(_) => { Ok(()) }
            Err(io_err) => { Err(AsmErr::ContentWriteErr(io_err)) }
        }
    }

    pub fn write_class_bytes(bytes: Vec<u8>, class_file: ClassFile) -> AsmResult<Vec<u8>> {
        let mut write_context = WriteContext { bytes };
        write_context.push(class_file)?;
        Ok(write_context.bytes)
    }
}

macro_rules! push_enum {
    (
        $contextExpr:expr, $fromExpr:expr;
        $(@$enumPath1:path {
            $( $fieldIdent1:ident $(,)? )*
        };)*
        $($enumPath2:path {
            $( $fieldIdent2:ident $(,)? )*
        };)*
    ) => {
        match $fromExpr {
            $($enumPath1($($fieldIdent1,)*) => {
                $(
                    $contextExpr.push($fieldIdent1)?;
                )*
            })*
            $($enumPath2{$($fieldIdent2,)*} => {
                $(
                    $contextExpr.push($fieldIdent2)?;
                )*
            })* 
        }
    };
}

pub(crate) use push_enum;

macro_rules! push_items {
    (
        $contextExpr:expr, $fromExpr:expr;
        $($fieldIdent:ident $(,)?)*
    ) => {
        $(
            $contextExpr.push($fromExpr.$fieldIdent)?;
        )*
    };
}

pub(crate) use push_items;
