use std::collections::HashMap;
use std::fmt::Display;
use std::io::Read;
use std::rc::Rc;

use java_asm_internal::err::{AsmErr, AsmResult};

use crate::jvms::attr::{Attribute as JvmsAttribute, RecordComponentInfo};
use crate::jvms::attr::annotation::{AnnotationElementValue, AnnotationInfo};
use crate::jvms::attr::annotation::type_annotation::TypeAnnotation;
use crate::jvms::element::{AttributeInfo, ClassFile, MethodInfo};
use crate::jvms::read::JvmsClassReader;
use crate::node::element::{AnnotationNode, Attribute, BootstrapMethodNode, ClassNode, ExceptionTable, InnerClassNode, ParameterNode, RecordComponentNode, TypeAnnotationNode, UnknownAttribute};
use crate::node::element::Attribute as NodeAttribute;
use crate::node::read::impls::ClassNodeFactory;
use crate::node::values::{AnnotationValue, ConstValue, LocalVariableInfo, LocalVariableTypeInfo, ModuleAttrValue, ModuleExportValue, ModuleOpenValue, ModuleProvidesValue, ModuleRequireValue};
use crate::util::{ToRc, VecEx};

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
    pub(crate) cp_cache: HashMap<u16, Rc<ConstValue>>,
    pub(crate) attr_cache: HashMap<u16, Rc<NodeAttribute>>,
}

impl ClassNodeContext {
    pub fn new(jvms_file: Rc<ClassFile>) -> Self {
        Self {
            jvms_file,
            cp_cache: HashMap::new(),
            attr_cache: HashMap::new(),
        }
    }

    pub fn read_attr_from_index(&mut self, index: u16) -> AsmResult<Rc<NodeAttribute>> {
        if let Some(attr) = self.read_attr_cache(index) {
            return Ok(attr);
        };
        let attribute_info = self.jvms_file.attributes.get(index as usize);
        let Some(attribute_info) = attribute_info else {
            return Err(self.err(format!("cannot find attribute info, index: {}", index)));
        };
        let node_attribute = self.read_attr(&attribute_info.clone())?.rc();
        self.put_attr_cache(index, Rc::clone(&node_attribute));
        Ok(node_attribute)
    }

    pub fn read_attr(&mut self, attribute_info: &AttributeInfo) -> AsmResult<NodeAttribute> {
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
                let attributes = attributes.mapping_res(|attr| Ok(self.read_attr(attr)?.rc()))?;
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
            JvmsAttribute::AnnotationDefault { default_value } => {
                let value = self.read_annotation_value(true, &default_value.value)?;
                NodeAttribute::AnnotationDefault(value)
            },
            JvmsAttribute::BootstrapMethods { bootstrap_methods, .. } => {
                let methods = bootstrap_methods.mapping_res(|method| {
                    let method_handle = self.read_const(method.bootstrap_method_ref)?;
                    let arguments = method.bootstrap_arguments.mapping_res(|arg| self.read_const(*arg))?;
                    Ok(BootstrapMethodNode { method_handle, arguments })
                })?;
                NodeAttribute::BootstrapMethods(methods)
            },
            JvmsAttribute::MethodParameters { parameters, .. } => {
                let parameters = parameters.mapping_res(|parameter| {
                    let name = self.read_utf8(parameter.name_index).ok();
                    let access = parameter.access_flags;
                    Ok(ParameterNode { name, access })
                })?;
                NodeAttribute::MethodParameters(parameters)
            },
            JvmsAttribute::Module {
                module_name_index, module_flags, module_version_index,
                requires, exports, opens, uses_index, provides, ..
            } => {
                let name = self.read_utf8(*module_name_index)?;
                let access = *module_flags;
                let version = self.read_utf8(*module_version_index).ok();
                let requires = requires.mapping_res(|require| {
                    let module = self.read_module(require.requires_index)?;
                    let access = require.requires_flags;
                    let version = self.read_utf8(require.requires_version_index).ok();
                    Ok(ModuleRequireValue { module, access, version })
                })?;
                let exports = exports.mapping_res(|export| {
                    let package = self.read_package(export.exports_index)?;
                    let access = export.exports_flags;
                    let modules = export.exports_to_index.mapping_res(|index| self.read_module(*index))?;
                    Ok(ModuleExportValue { package, access, modules })
                })?;
                let opens = opens.mapping_res(|open| {
                    let package = self.read_package(open.opens_index)?;
                    let access = open.opens_flags;
                    let modules = open.opens_to_index.mapping_res(|index| self.read_module(*index))?;
                    Ok(ModuleOpenValue { package, access, modules })
                })?;
                let uses = uses_index.mapping_res(|index| self.read_class_info(*index))?;
                let provides = provides.mapping_res(|provide| {
                    let service = self.read_class_info(provide.provides_index)?;
                    let providers = provide.provides_with_index.mapping_res(|index| self.read_class_info(*index))?;
                    Ok(ModuleProvidesValue { service, providers })
                })?;
                let attr_value = ModuleAttrValue {
                    name, access, version, requires, exports, opens, uses, provides
                };
                NodeAttribute::Module(attr_value)
            }
            JvmsAttribute::ModulePackages { package_index, .. } => {
                let packages = package_index.mapping_res(|index| self.read_package(*index))?;
                NodeAttribute::ModulePackages(packages)
            },
            JvmsAttribute::ModuleMainClass { main_class_index } => {
                let main_class = self.read_class_info(*main_class_index)?;
                NodeAttribute::ModuleMainClass(main_class)
            },
            JvmsAttribute::NestHost { host_class_index } => {
                let host_class = self.read_class_info(*host_class_index)?;
                NodeAttribute::NestHost(host_class)
            },
            JvmsAttribute::NestMembers { classes, .. } => {
                let classes = classes.mapping_res(|index| self.read_class_info(*index))?;
                NodeAttribute::NestMembers(classes)
            },
            JvmsAttribute::Record { components, .. } => {
                let components = components.mapping_res(|component| {
                    self.read_record(component)
                })?;
                NodeAttribute::Record(components)
            },
            // todo supports more
            _ => return Err(self.err(format!("unsupported attribute: {:?}", attribute_info))),
        };
        Ok(attr)
    }

    fn read_attr_cache(&self, index: u16) -> Option<Rc<NodeAttribute>> {
        self.attr_cache.get(&index).map(Rc::clone)
    }

    fn put_attr_cache(&mut self, index: u16, attr: Rc<NodeAttribute>) {
        self.attr_cache.insert(index, attr);
    }

    fn read_record(&mut self, component: &RecordComponentInfo) -> AsmResult<RecordComponentNode> {
        let name = self.read_utf8(component.name_index)?;
        let desc = self.read_utf8(component.descriptor_index)?;
        let mut signature = None;
        let mut annotations = vec![];
        let mut type_annotations = vec![];
        let mut unknown_attrs = vec![];
        for (index, attr_info) in component.attributes.iter().enumerate() {
            let name = self.read_utf8(attr_info.attribute_name_index)?;
            let attr = self.read_attr(attr_info)?;
            match attr {
                Attribute::Signature(s) => signature = Some(s.clone()),
                Attribute::RuntimeVisibleAnnotations(s) => annotations = s.clone(),
                Attribute::RuntimeInvisibleAnnotations(s) => annotations = s.clone(),
                Attribute::RuntimeVisibleTypeAnnotations(s) => type_annotations = s.clone(),
                Attribute::RuntimeInvisibleTypeAnnotations(s) => type_annotations = s.clone(),
                Attribute::Custom(info) => unknown_attrs.push(UnknownAttribute { 
                    name, info, index: index as u16, 
                }),
                _ => return Err(self.err(format!("unsupported record component attribute: {:?}", attr))),
            }
        }
        Ok(RecordComponentNode {
            name, desc, signature, annotations, type_annotations, attrs: unknown_attrs,
        })
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

pub(crate) struct MethodNodeContext {
    pub jvms_file: Rc<ClassFile>,
    pub method_info: Rc<MethodInfo>,
}
