use crate::err::{AsmResult, AsmResultOkExt};
use crate::impls::node::r::node_reader::{ClassNodeContext, ConstPool};
use crate::jvms::attr::annotation::{AnnotationElementValue, AnnotationInfo};
use crate::jvms::attr::Attribute as JvmsAttribute;
use crate::jvms::attr::RecordComponentInfo;
use crate::jvms::attr::type_annotation::TypeAnnotation;
use crate::jvms::element::AttributeInfo;
use crate::node::element::{AnnotationNode, BootstrapMethodAttr, CodeAttribute, EnclosingMethodAttribute, ExceptionTable, InnerClassNode, ParameterNode, RecordComponentNode, TypeAnnotationNode, UnknownAttribute};
use crate::node::element::Attribute as NodeAttribute;
use crate::node::values::{AnnotationValue, LocalVariableInfo, LocalVariableTypeInfo, ModuleAttrValue, ModuleExportValue, ModuleOpenValue, ModuleProvidesValue, ModuleRequireValue};
use crate::util::{mutf8_to_string, VecEx};

impl ClassNodeContext {
    pub fn read_class_attrs(&self) -> AsmResult<Vec<(AttributeInfo, NodeAttribute)>> {
        let jvms_attrs = &self.jvms_file.attributes;
        let attributes = self.cp.read_attrs(jvms_attrs)?;
        let mut result = Vec::with_capacity(attributes.len());
        for (attr_info, attr) in attributes {
            result.push((attr_info.clone(), attr));
        }
        result.ok()
    }
}

impl ConstPool {
    /// converts jvms attributes [JvmsAttribute] to node attributes [NodeAttribute]
    pub fn read_attrs(&self, attrs: &Vec<AttributeInfo>) -> AsmResult<Vec<(AttributeInfo, NodeAttribute)>> {
        let mut result = Vec::with_capacity(attrs.len());
        for attr_info in attrs {
            let attribute = self.read_attr(attr_info)?;
            result.push((attr_info.clone(), attribute));
        };
        Ok(result)
    }

    pub fn read_attr(&self, attribute_info: &AttributeInfo) -> AsmResult<NodeAttribute> {
        let attr = match &attribute_info.info {
            JvmsAttribute::Code {
                max_stack, max_locals, code, exception_table,
                attributes: jvms_attributes, ..
            } => {
                let exception_table = exception_table.mapping(|et| {
                    ExceptionTable {
                        start: et.start_pc,
                        end: et.end_pc,
                        handler: et.handler_pc,
                        catch_type: self.read_class_info(et.catch_type).ok(),
                    }
                });
                let mut attributes = vec![];
                for attr in jvms_attributes {
                    let attribute_info = attr.clone();
                    let attr = self.read_attr(attr)?;
                    attributes.push((attribute_info, attr));
                }
                NodeAttribute::Code(CodeAttribute {
                    max_stack: *max_stack,
                    max_locals: *max_locals,
                    code: code.clone(),
                    exception_table,
                    attributes,
                })
            },
            JvmsAttribute::StackMapTable { entries, .. } => NodeAttribute::StackMapTable(entries.clone()),
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
                NodeAttribute::EnclosingMethod(
                    EnclosingMethodAttribute { class, method_name, method_desc })
            },
            JvmsAttribute::Synthetic => NodeAttribute::Synthetic,
            JvmsAttribute::Signature { signature_index } => NodeAttribute::Signature(self.read_utf8(*signature_index)?),
            JvmsAttribute::SourceFile { sourcefile_index } => NodeAttribute::SourceFile(self.read_utf8(*sourcefile_index)?),
            JvmsAttribute::SourceDebugExtension { debug_extension } => NodeAttribute::SourceDebugExtension(
                mutf8_to_string(debug_extension)?,
            ),
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
                    let method_handle = self.get(method.bootstrap_method_ref)?;
                    let arguments = method.bootstrap_arguments.mapping_res(|arg| self.get(*arg))?;
                    BootstrapMethodAttr { method_handle, arguments }.ok()
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
            JvmsAttribute::PermittedSubclasses { classes, .. } => {
                let classes = classes.mapping_res(|index| self.read_class_info(*index))?;
                NodeAttribute::PermittedSubclasses(classes)
            },
            _ => NodeAttribute::Unknown(UnknownAttribute {
                name: self.read_utf8(attribute_info.attribute_name_index)?,
                origin: attribute_info.info.clone(),
            }),
        };
        Ok(attr)
    }

    fn read_record(&self, component: &RecordComponentInfo) -> AsmResult<RecordComponentNode> {
        let name = self.read_utf8(component.name_index)?;
        let desc = self.read_utf8(component.descriptor_index)?;
        let mut signature = None;
        let mut annotations = vec![];
        let mut type_annotations = vec![];
        let mut unknown_attrs = vec![];
        for attr_info in &component.attributes {
            let attr = self.read_attr(attr_info)?;
            match attr {
                NodeAttribute::Signature(s) => signature = Some(s.clone()),
                NodeAttribute::RuntimeVisibleAnnotations(s) => annotations = s.clone(),
                NodeAttribute::RuntimeInvisibleAnnotations(s) => annotations = s.clone(),
                NodeAttribute::RuntimeVisibleTypeAnnotations(s) => type_annotations = s.clone(),
                NodeAttribute::RuntimeInvisibleTypeAnnotations(s) => type_annotations = s.clone(),
                NodeAttribute::Unknown(info) => unknown_attrs.push(info),
                _ => return Err(self.err(format!("unsupported record component attribute: {:?}", attr))),
            }
        }
        Ok(RecordComponentNode {
            name, desc, signature, annotations, type_annotations, attrs: unknown_attrs,
        })
    }

    fn read_type_annotation(&self, visible: bool, type_annotation: &TypeAnnotation) -> AsmResult<TypeAnnotationNode> {
        let annotation_attr = AnnotationInfo {
            type_index: type_annotation.type_index,
            num_element_value_pairs: type_annotation.num_element_value_pairs,
            element_value_pairs: type_annotation.element_value_pairs.clone(),
        };
        let annotation_node = self.read_annotation_info(visible, &annotation_attr)?;
        Ok(TypeAnnotationNode {
            visible,
            target_info: type_annotation.target_info.clone(),
            target_path: type_annotation.target_path.clone(),
            annotation_node,
        })
    }

    fn read_annotation_info(&self, visible: bool, annotation: &AnnotationInfo) -> AsmResult<AnnotationNode> {
        let type_name = self.read_class_info(annotation.type_index)?;
        let values = annotation.element_value_pairs.mapping_res(|pair| {
            let element_name = self.read_utf8(pair.element_name_index)?;
            let value = self.read_annotation_value(visible, &pair.value.value)?;
            Ok((element_name, value))
        })?;
        Ok(AnnotationNode { visible, type_name, values })
    }

    fn read_annotation_value(&self, visible: bool, annotation: &AnnotationElementValue) -> AsmResult<AnnotationValue> {
        let value = match annotation {
            AnnotationElementValue::Const { const_value_index } => {
                let value = self.get(*const_value_index)?;
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

    pub fn unknown_attr(&self, attribute_info: AttributeInfo) -> AsmResult<UnknownAttribute> {
        Ok(UnknownAttribute {
            name: self.read_utf8(attribute_info.attribute_name_index)?,
            origin: attribute_info.info.clone(),
        })
    }
}
