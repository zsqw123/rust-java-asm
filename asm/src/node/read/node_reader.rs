use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::rc::Rc;

use java_asm_internal::err::{AsmErr, AsmResult};

use crate::jvms::attr::annotation::{AnnotationElementValue, AnnotationInfo};
use crate::jvms::attr::annotation::type_annotation::TypeAnnotation;
use crate::jvms::attr::Attribute as JvmsAttribute;
use crate::jvms::element::{AttributeInfo, ClassFile, Const, MethodInfo};
use crate::jvms::read::JvmsClassReader;
use crate::node::element::{AnnotationNode, ClassNode, ExceptionTable, InnerClassNode, TypeAnnotationNode};
use crate::node::element::Attribute as NodeAttribute;
use crate::node::read::impls::ClassNodeFactory;
use crate::node::values::{AnnotationValue, ConstValue, Descriptor, LocalVariableInfo, LocalVariableTypeInfo};
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
    attr_cache: HashMap<u16, Rc<NodeAttribute>>,
}

impl ClassNodeContext {
    pub fn new(jvms_file: Rc<ClassFile>) -> Self {
        Self {
            jvms_file,
            cp_cache: HashMap::new(),
            attr_cache: HashMap::new(),
        }
    }

    pub fn name(&mut self) -> Option<Rc<String>> {
        self.read_class_info(self.jvms_file.this_class).ok()
    }
    
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
        let raw_const = self.jvms_file.constant_pool[index as usize].info.clone();
        let const_value = match raw_const {
            Const::Invalid => { ConstValue::Invalid },
            Const::Class { name_index } => {
                ConstValue::Class(self.read_utf8(name_index)?)
            },
            Const::Field { class_index, name_and_type_index }
            | Const::Method { class_index, name_and_type_index }
            | Const::InterfaceMethod { class_index, name_and_type_index } => {
                let class = self.read_class_info(class_index)?;
                let (name, desc) = self.read_name_and_type(name_and_type_index)?;
                ConstValue::Member { class, name, desc }
            },
            Const::String { string_index } => {
                ConstValue::String(self.read_utf8(string_index)?)
            },
            Const::Integer { bytes } => ConstValue::Integer(bytes as i32),
            Const::Float { bytes } => ConstValue::Float(f32::from_bits(bytes)),
            Const::Long { high_bytes, low_bytes } => {
                let value = ((high_bytes as u64) << 32) | (low_bytes as u64);
                ConstValue::Long(value as i64)
            },
            Const::Double { high_bytes, low_bytes } => {
                let value = ((high_bytes as u64) << 32) | (low_bytes as u64);
                ConstValue::Double(f64::from_bits(value))
            },
            Const::NameAndType { name_index, descriptor_index } => {
                let name = self.read_utf8(name_index)?;
                let desc = self.read_utf8(descriptor_index)?;
                ConstValue::NameAndType { name, desc }
            },
            Const::Utf8 { bytes, .. } => {
                ConstValue::String(mutf8_to_string(&bytes)?.rc())
            },
            Const::MethodHandle { reference_kind, reference_index } => {
                ConstValue::MethodHandle { reference_kind, reference_index }
            },
            Const::MethodType { descriptor_index } => {
                ConstValue::MethodType(self.read_utf8(descriptor_index)?)
            },
            Const::Dynamic { bootstrap_method_attr_index, name_and_type_index }
            | Const::InvokeDynamic { bootstrap_method_attr_index, name_and_type_index } => {
                let (name, desc) = self.read_name_and_type(name_and_type_index)?;
                ConstValue::Dynamic { bootstrap_method_attr_index, name, desc }
            },
            Const::Module { name_index } => ConstValue::Module(self.read_utf8(name_index)?),
            Const::Package { name_index } => ConstValue::Package(self.read_utf8(name_index)?),
        };
        let const_value = const_value.rc();
        self.put_const_cache(index, Rc::clone(&const_value));
        return Ok(const_value)
    }

    fn read_const_cache(&self, index: u16) -> Option<Rc<ConstValue>> {
        self.cp_cache.get(&index).map(Rc::clone)
    }

    fn put_const_cache(&mut self, index: u16, constant: Rc<ConstValue>) {
        self.cp_cache.insert(index, constant);
    }

    pub fn read_attr_from_index(&mut self, index: u16) -> AsmResult<Rc<NodeAttribute>> {
        if let Some(attr) = self.read_attr_cache(index) {
            return Ok(attr);
        };
        let attribute_info = self.jvms_file.attributes.get(index as usize);
        let Some(attribute_info) = attribute_info else {
            return Err(self.err(format!("cannot find attribute info, index: {}", index)));
        };
        let node_attribute = self.read_attr(&attribute_info.clone())?;
        self.put_attr_cache(index, Rc::clone(&node_attribute));
        Ok(node_attribute)
    }

    pub fn read_attr(&mut self, attribute_info: &AttributeInfo) -> AsmResult<Rc<NodeAttribute>> {
        let attr = match &attribute_info.info {
            JvmsAttribute::Custom(bytes) => NodeAttribute::Custom(bytes.clone()),
            JvmsAttribute::Code { max_stack, max_locals, code, exception_table, attributes, .. } => {
                let exception_table = exception_table.mapping(|et| {
                    ExceptionTable {
                        start: et.start_pc,
                        end: et.end_pc,
                        handler: et.handler_pc,
                        catch_type: self.read_class_info(et.catch_type).ok(),
                    }
                });
                let attributes = attributes.mapping_res(|attr| self.read_attr(attr))?;
                NodeAttribute::Code {
                    max_stack: *max_stack,
                    max_locals: *max_locals,
                    code: code.clone(),
                    exception_table,
                    attributes,
                }
            },
            JvmsAttribute::StackMapTable { entries, .. } => {
                NodeAttribute::StackMapTable(entries.clone())
            },
            JvmsAttribute::Exceptions { exception_index_table, .. } => {
                let exceptions = exception_index_table.mapping_res(|index| self.read_class_info(*index))?;
                NodeAttribute::Exceptions(exceptions)
            },
            JvmsAttribute::InnerClasses { classes, .. } => {
                let classes = classes.mapping_res(|inner_class| {
                    let name = self.read_class_info(inner_class.inner_class_info_index)?;
                    let outer_name = self.read_class_info(inner_class.outer_class_info_index).ok();
                    let inner_name = self.read_utf8(inner_class.inner_name_index)?;
                    let access = inner_class.inner_class_access_flags;
                    Ok(InnerClassNode { name, outer_name, inner_name, access })
                })?;
                NodeAttribute::InnerClasses(classes)
            },
            JvmsAttribute::EnclosingMethod { class_index, method_index } => {
                let class = self.read_class_info(*class_index)?;
                let (method_name, method_desc) = self.read_name_and_type(*method_index)?;
                NodeAttribute::EnclosingMethod { class, method_name, method_desc }
            },
            JvmsAttribute::Synthetic => NodeAttribute::Synthetic,
            JvmsAttribute::Signature { signature_index } => NodeAttribute::Signature(self.read_utf8(*signature_index)?),
            JvmsAttribute::SourceFile { sourcefile_index } => NodeAttribute::SourceFile(self.read_utf8(*sourcefile_index)?),
            JvmsAttribute::SourceDebugExtension { debug_extension } => NodeAttribute::SourceDebugExtension(debug_extension.clone()),
            JvmsAttribute::LineNumberTable { line_number_table, .. } => {
                NodeAttribute::LineNumberTable(line_number_table.clone())
            },
            JvmsAttribute::LocalVariableTable { local_variable_table, .. } => {
                let local_variables = local_variable_table.mapping_res(|local_variable| {
                    let start = local_variable.start_pc;
                    let length = local_variable.length;
                    let name = self.read_utf8(local_variable.name_index)?;
                    let desc = self.read_utf8(local_variable.descriptor_index)?;
                    let index = local_variable.index;
                    Ok(LocalVariableInfo { start, length, name, desc, index })
                })?;
                NodeAttribute::LocalVariableTable(local_variables)
            },
            JvmsAttribute::LocalVariableTypeTable { local_variable_table, .. } => {
                let local_variables = local_variable_table.mapping_res(|local_variable| {
                    let start = local_variable.start_pc;
                    let length = local_variable.length;
                    let name = self.read_utf8(local_variable.name_index)?;
                    let signature = self.read_utf8(local_variable.signature_index)?;
                    let index = local_variable.index;
                    Ok(LocalVariableTypeInfo { start, length, name, signature, index })
                })?;
                NodeAttribute::LocalVariableTypeTable(local_variables)
            },
            JvmsAttribute::Deprecated => NodeAttribute::Deprecated,
            JvmsAttribute::RuntimeVisibleAnnotations { annotations, .. } => {
                let annotations = annotations.mapping_res(|annotation|
                    self.read_annotation_info(true, annotation))?;
                NodeAttribute::RuntimeVisibleAnnotations(annotations)
            },
            JvmsAttribute::RuntimeInvisibleAnnotations { annotations, .. } => {
                let annotations = annotations.mapping_res(|annotation|
                    self.read_annotation_info(false, annotation))?;
                NodeAttribute::RuntimeInvisibleAnnotations(annotations)
            },
            JvmsAttribute::RuntimeVisibleParameterAnnotations { parameter_annotations, .. } => {
                let parameter_annotations = parameter_annotations.mapping_res(|parameter|
                    parameter.annotations.mapping_res(|annotation| self.read_annotation_info(true, annotation)))?;
                NodeAttribute::RuntimeVisibleParameterAnnotations(parameter_annotations)
            },
            JvmsAttribute::RuntimeInvisibleParameterAnnotations { parameter_annotations, .. } => {
                let parameter_annotations = parameter_annotations.mapping_res(|parameter|
                    parameter.annotations.mapping_res(|annotation| self.read_annotation_info(false, annotation)))?;
                NodeAttribute::RuntimeInvisibleParameterAnnotations(parameter_annotations)
            },
            JvmsAttribute::RuntimeVisibleTypeAnnotations { annotations, .. } => {
                let annotations = annotations.mapping_res(|annotation|
                    self.read_type_annotation(true, annotation))?;
                NodeAttribute::RuntimeVisibleTypeAnnotations(annotations)
            },
            JvmsAttribute::RuntimeInvisibleTypeAnnotations { annotations, .. } => {
                let annotations = annotations.mapping_res(|annotation|
                    self.read_type_annotation(false, annotation))?;
                NodeAttribute::RuntimeInvisibleTypeAnnotations(annotations)
            },
            // todo supports more
            _ => return Err(self.err(format!("unsupported attribute: {:?}", attribute_info))),
        };
        Ok(attr.rc())
    }

    fn read_attr_cache(&self, index: u16) -> Option<Rc<NodeAttribute>> {
        self.attr_cache.get(&index).map(Rc::clone)
    }

    fn put_attr_cache(&mut self, index: u16, attr: Rc<NodeAttribute>) {
        self.attr_cache.insert(index, attr);
    }

    fn read_type_annotation(&mut self, visible: bool, type_annotation: &TypeAnnotation) -> AsmResult<TypeAnnotationNode> {
        let annotation_attr = AnnotationInfo {
            type_index: type_annotation.type_index,
            num_element_value_pairs: type_annotation.num_element_value_pairs,
            element_value_pairs: type_annotation.element_value_pairs.clone(),
        };
        let annotation_node = self.read_annotation_info(visible, &annotation_attr)?;
        Ok(TypeAnnotationNode {
            target_info: type_annotation.target_info.clone(),
            target_path: type_annotation.target_path.clone(),
            annotation_node,
        })
    }

    fn read_annotation_info(&mut self, visible: bool, annotation: &AnnotationInfo) -> AsmResult<AnnotationNode> {
        let type_name = self.read_class_info(annotation.type_index)?;
        let values = annotation.element_value_pairs.mapping_res(|pair| {
            let element_name = self.read_utf8(pair.element_name_index)?;
            let value = self.read_annotation_value(visible, &pair.value.value)?;
            Ok((element_name, value))
        })?;
        Ok(AnnotationNode { visible, type_name, values })
    }

    fn read_annotation_value(&mut self, visible: bool, annotation: &AnnotationElementValue) -> AsmResult<AnnotationValue> {
        let value = match annotation {
            AnnotationElementValue::Const { const_value_index } => {
                let value = self.read_const(*const_value_index)?;
                AnnotationValue::Const(value)
            },
            AnnotationElementValue::EnumConst { type_name_index, const_name_index } => {
                let name = self.read_utf8(*type_name_index)?;
                let value = self.read_utf8(*const_name_index)?;
                AnnotationValue::Enum(name, value)
            },
            AnnotationElementValue::Class { class_info_index } => {
                let name = self.read_utf8(*class_info_index)?;
                AnnotationValue::Class(name)
            },
            AnnotationElementValue::Annotation { annotation_value } => {
                let annotation = self.read_annotation_info(visible, annotation_value)?;
                AnnotationValue::Annotation(annotation)
            },
            AnnotationElementValue::Array { values, .. } => {
                let values = values.mapping_res(|value|
                    self.read_annotation_value(visible, &value.value))?;
                AnnotationValue::Array(values)
            },
        };
        Ok(value)
    }

    fn err<D: Display>(&mut self, msg: D) -> AsmErr {
        match self.name() {
            Some(name) => AsmErr::ResolveNode(format!("class: {}, {}", name, msg)),
            None => AsmErr::ResolveNode(msg.to_string()),
        }
    }
}

trait ToRc<T> {
    fn rc(self) -> Rc<T>;
}

impl<T> ToRc<T> for T {
    fn rc(self) -> Rc<T> { Rc::new(self) }
}

trait VecEx<T> {
    fn mapping_res<R>(&self, f: impl FnMut(&T) -> AsmResult<R>) -> AsmResult<Vec<R>>;
    fn mapping<R>(&self, f: impl FnMut(&T) -> R) -> Vec<R>;
}

impl<T> VecEx<T> for Vec<T> {
    #[inline]
    fn mapping_res<R>(&self, mut f: impl FnMut(&T) -> AsmResult<R>) -> AsmResult<Vec<R>> {
        let mut new = Vec::with_capacity(self.len());
        for item in self { new.push(f(item)?); }
        Ok(new)
    }

    #[inline]
    fn mapping<R>(&self, mut f: impl FnMut(&T) -> R) -> Vec<R> {
        let mut new = Vec::with_capacity(self.len());
        for item in self { new.push(f(item)); }
        new
    }
}

pub(crate) struct MethodNodeContext {
    pub jvms_file: Rc<ClassFile>,
    pub method_info: Rc<MethodInfo>,
}
