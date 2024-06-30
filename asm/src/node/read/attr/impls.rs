use std::collections::HashMap;
use std::rc::Rc;

use java_asm_internal::err::{AsmErr, AsmResult};
use crate::jvms::attr::annotation::type_annotation::TypeAnnotationTargetInfo;

use crate::jvms::element::{ClassFile, FieldInfo, MethodInfo};
use crate::node::element::{Attribute, ClassNode, FieldNode, LocalVariableNode, MethodNode, ModuleNode, TypeAnnotationNode, UnknownAttribute};
use crate::node::read::node_reader::{ClassNodeContext, MethodNodeContext};
use crate::node::values::{ConstValue, FieldInitialValue, LocalVariableInfo, LocalVariableTypeInfo, ModuleAttrValue};
use crate::util::ToRc;

