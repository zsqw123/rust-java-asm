use std::rc::Rc;

use crate::err::{AsmErr, AsmResult};
use crate::impls::node::r::node_reader::ClassNodeContext;
use crate::jvms::element::{ClassFile, FieldInfo, MethodInfo};
use crate::node::element::{Attribute, ClassNode, FieldNode, MethodNode, ModuleNode, UnknownAttribute};
use crate::node::values::{ConstValue, FieldInitialValue, ModuleAttrValue};
use crate::util::VecEx;

pub fn from_jvms_internal(jvms_file: ClassFile) -> AsmResult<ClassNode> {
    let jvms_file = Rc::new(jvms_file);
    let class_context = ClassNodeContext::new(Rc::clone(&jvms_file));

    let mut signature = None;
    let super_name = Some(class_context.read_class_info_or_default(jvms_file.super_class));
    let interfaces = jvms_file.interfaces.iter().map(|&index| {
        class_context.read_class_info_or_default(index)
    }).collect();
    
    
    let mut source_file = None;
    let mut source_debug = None;
    // module stuff
    let mut module_attr = None;
    let mut module_packages = None;
    let mut module_main = None;
    // EnclosingMethod
    let mut outer_class = None;
    let mut outer_method_name = None;
    let mut outer_method_desc = None;

    let mut annotations = vec![];
    let mut type_annotations = vec![];
    let mut attrs = vec![];
    let mut inner_classes = vec![];
    let mut nest_host_class = None;
    let mut nest_members = vec![];
    let mut permitted_subclasses = vec![];
    let mut record_components = vec![];

    let name = class_context.name()?;
    // read raw class attributes
    let class_attrs = class_context.read_class_attrs()?;
    for (attribute_info, attribute) in class_attrs {
        match attribute {
            Attribute::Signature(s) => signature = Some(s),
            Attribute::SourceFile(s) => source_file = Some(s),
            Attribute::SourceDebugExtension(s) => source_debug = Some(s),
            // module stuff
            Attribute::Module(a) => module_attr = Some(a),
            Attribute::ModulePackages(packages) => module_packages = Some(packages),
            Attribute::ModuleMainClass(main) => module_main = Some(main),

            Attribute::EnclosingMethod(enc) => {
                outer_class = Some(enc.class);
                outer_method_name = Some(enc.method_name);
                outer_method_desc = Some(enc.method_desc);
            },

            // annotations
            Attribute::RuntimeVisibleAnnotations(an) => annotations.extend(an),
            Attribute::RuntimeInvisibleAnnotations(an) => annotations.extend(an),
            Attribute::RuntimeVisibleTypeAnnotations(tan) => type_annotations.extend(tan),
            Attribute::RuntimeInvisibleTypeAnnotations(tan) => type_annotations.extend(tan),

            Attribute::InnerClasses(ic) => inner_classes.extend(ic),
            Attribute::NestHost(nh) => nest_host_class = Some(nh),
            Attribute::NestMembers(nm) => nest_members.extend(nm),
            Attribute::PermittedSubclasses(ps) => permitted_subclasses.extend(ps),
            Attribute::Record(rc) => record_components.extend(rc),
            
            Attribute::BootstrapMethods(bm_attrs) => {
                class_context.bootstrap_methods.set(bm_attrs).map_err(|prev| {
                    let err_msg = format!("most one bootstrap methods attribute is allowed, \
                    but found another one: {:?}", prev);
                    AsmErr::ResolveNode(err_msg)
                })?;
            }

            Attribute::Unknown(v) => attrs.push(v),
            _ => attrs.push(UnknownAttribute {
                name: class_context.read_utf8(attribute_info.attribute_name_index)?,
                origin: attribute_info.info.clone(),
            }),
        }
    }

    let module = match module_attr {
        Some(attr) => {
            let ModuleAttrValue {
                name, access, version,
                requires, exports, opens,
                uses, provides
            } = attr;
            let version = if let Some(v) = version { Some(v) } else { None };
            let main_class = module_main;
            let packages = module_packages.unwrap_or_default();

            Some(ModuleNode {
                name, access, version, main_class,
                packages, requires, exports, opens, uses, provides,
            })
        }
        None => None,
    };

  
    let fields = jvms_file.fields.mapping_res(|field_info| {
        field_from_jvms(&class_context, field_info)
    })?;

    let methods = jvms_file.methods.mapping_res(|method_info| {
        method_from_jvms(&class_context, method_info)
    })?;

    let class_node = ClassNode {
        minor_version: *&jvms_file.minor_version,
        major_version: *&jvms_file.major_version,
        access: *&jvms_file.access_flags,
        name, signature, super_name, interfaces, source_file, source_debug,
        module, outer_class, outer_method_name, outer_method_desc,
        annotations, type_annotations, inner_classes, nest_host_class, nest_members,
        permitted_subclasses, record_components, fields, methods, attrs,
    };
    Ok(class_node)
}

fn field_from_jvms(class_context: &ClassNodeContext, field_info: &FieldInfo) -> AsmResult<FieldNode> {
    let name = class_context.read_utf8(field_info.name_index)?;
    let access = field_info.access_flags;
    let desc = class_context.read_utf8(field_info.descriptor_index)?;
    let mut signature = None;
    let mut value = None;
    let mut annotations = vec![];
    let mut type_annotations = vec![];
    let mut attrs = vec![];

    for (attribute_info, attribute) in class_context.read_attrs(&field_info.attributes)? {
        match attribute {
            Attribute::Signature(s) => signature = Some(s),
            Attribute::ConstantValue(v) => {
                value = match v {
                    ConstValue::Integer(i) => Some(FieldInitialValue::Integer(i)),
                    ConstValue::Float(f) => Some(FieldInitialValue::Float(f)),
                    ConstValue::Long(l) => Some(FieldInitialValue::Long(l)),
                    ConstValue::Double(d) => Some(FieldInitialValue::Double(d)),
                    ConstValue::String(s) => Some(FieldInitialValue::String(s)),
                    _ => AsmErr::ResolveNode(
                        format!("invalid constant value {:?} for field: {}", v, name)
                    ).e()?,
                }
            },
            Attribute::RuntimeVisibleAnnotations(an) => annotations.extend(an),
            Attribute::RuntimeInvisibleAnnotations(an) => annotations.extend(an),
            Attribute::RuntimeVisibleTypeAnnotations(tan) => type_annotations.extend(tan),
            Attribute::RuntimeInvisibleTypeAnnotations(tan) => type_annotations.extend(tan),
            Attribute::Unknown(v) => attrs.push(v),
            _ => attrs.push(UnknownAttribute {
                name: class_context.read_utf8(attribute_info.attribute_name_index)?,
                origin: attribute_info.info.clone(),
            }),
        }
    }

    let field_node = FieldNode {
        name, access, desc, signature, value, annotations, type_annotations, attrs,
    };
    Ok(field_node)
}

fn method_from_jvms(class_context: &ClassNodeContext, method_info: &MethodInfo) -> AsmResult<MethodNode> {
    let mut signature = None;
    let mut exceptions = vec![];
    let mut parameters = vec![];

    let mut annotations = vec![];
    let mut type_annotations = vec![];
    let mut parameter_annotations = vec![];
    let mut attrs = vec![];

    let mut annotation_default = None;
    let mut code_body = None;

    let name = class_context.read_utf8(method_info.name_index)?;
    let all_attributes = class_context.read_attrs(&method_info.attributes)?;
    for (attribute_info, attribute) in all_attributes {
        match attribute {
            Attribute::Signature(s) => signature = Some(s),
            Attribute::Exceptions(ex) => exceptions = ex,
            Attribute::MethodParameters(ps) => parameters = ps,

            Attribute::RuntimeVisibleAnnotations(an) => annotations.extend(an),
            Attribute::RuntimeInvisibleAnnotations(an) => annotations.extend(an),
            Attribute::RuntimeVisibleTypeAnnotations(tan) => type_annotations.extend(tan),
            Attribute::RuntimeInvisibleTypeAnnotations(tan) => type_annotations.extend(tan),
            Attribute::RuntimeVisibleParameterAnnotations(pan) => parameter_annotations.extend(pan),
            Attribute::RuntimeInvisibleParameterAnnotations(pan) => parameter_annotations.extend(pan),

            Attribute::AnnotationDefault(v) => annotation_default = Some(v),

            Attribute::Code(code_attribute) => {
                code_body = Some(class_context.read_code_body(code_attribute)?);
            }
            
            Attribute::Unknown(v) => attrs.push(v),
            _ => attrs.push(UnknownAttribute {
                name: class_context.read_utf8(attribute_info.attribute_name_index)?,
                origin: attribute_info.info.clone(),
            }),
        }
    }

    let access = method_info.access_flags;
    let desc = class_context.read_utf8(method_info.descriptor_index)?;
    let method_node = MethodNode {
        name, access, desc, signature, exceptions, parameters,
        annotations, type_annotations, parameter_annotations, attrs,
        annotation_default,
        code_body,
    };
    Ok(method_node)
}
