use std::rc::Rc;
use java_asm_internal::err::{AsmErr, AsmResult};
use crate::jvms::element::{ClassFile, FieldInfo, MethodInfo};
use crate::node::element::{Attribute, ClassNode, FieldNode, MethodNode, ModuleNode, UnknownAttribute};
use crate::node::read::node_reader::{ClassNodeContext, MethodNodeContext};
use crate::node::values::{ConstValue, FieldInitialValue, ModuleAttrValue};
use crate::util::ToRc;

pub fn from_jvms_internal(jvms_file: Rc<ClassFile>) -> AsmResult<ClassNode> {
    let mut class_context = ClassNodeContext::new(Rc::clone(&jvms_file));
    let name = class_context.name()?;
    let mut signature = None;
    let mut super_name = None;
    let mut interfaces = Vec::with_capacity(jvms_file.interfaces_count as usize);
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
    let mut fields = Vec::with_capacity(jvms_file.fields_count as usize);
    let mut methods = Vec::with_capacity(jvms_file.methods_count as usize);

    for (attribute_info, attribute) in class_context.read_class_attrs()? {
        match attribute.as_ref() {
            Attribute::Signature(s) => signature = Some(Rc::clone(s)),
            Attribute::SourceFile(s) => source_file = Some(Rc::clone(s)),
            Attribute::SourceDebugExtension(s) => source_debug = Some(Rc::clone(s)),
            // module stuff
            Attribute::Module(a) => module_attr = Some(a),
            Attribute::ModulePackages(packages) => module_packages = Some(packages.clone()),
            Attribute::ModuleMainClass(main) => module_main = Some(main.clone()),

            Attribute::EnclosingMethod { class, method_name, method_desc } => {
                outer_class = Some(Rc::clone(class));
                outer_method_name = Some(Rc::clone(method_name));
                outer_method_desc = Some(Rc::clone(method_desc));
            }

            // annotations
            Attribute::RuntimeVisibleAnnotations(an) => annotations.extend(an.clone()),
            Attribute::RuntimeInvisibleAnnotations(an) => annotations.extend(an.clone()),
            Attribute::RuntimeVisibleTypeAnnotations(tan) => type_annotations.extend(tan.clone()),
            Attribute::RuntimeInvisibleTypeAnnotations(tan) => type_annotations.extend(tan.clone()),

            Attribute::InnerClasses(ic) => inner_classes.extend(ic.clone()),
            Attribute::NestHost(nh) => nest_host_class = Some(Rc::clone(nh)),
            Attribute::NestMembers(nm) => nest_members.extend(nm.clone()),
            Attribute::PermittedSubclasses(ps) => permitted_subclasses.extend(ps.clone()),
            Attribute::Record(rc) => record_components.extend(rc.clone()),

            Attribute::Unknown(v) => attrs.push(v.clone()),
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
            let name = Rc::clone(name);
            let access = *access;
            let version = if let Some(v) = version { Some(Rc::clone(v)) } else { None };
            let main_class = module_main;
            let packages = module_packages.unwrap_or_default();
            let requires = requires.clone();
            let exports = exports.clone();
            let opens = opens.clone();
            let uses = uses.clone();
            let provides = provides.clone();

            Some(ModuleNode {
                name, access, version, main_class,
                packages, requires, exports, opens, uses, provides,
            })
        }
        None => None,
    };

    for field_info in jvms_file.fields.iter() {
        fields.push(field_from_jvms(&mut class_context, field_info)?);
    }

    for method_info in jvms_file.methods.iter() {
        let method_info = method_info.clone().rc();
        methods.push(method_from_jvms(&mut class_context, method_info)?);
    }

    let class_node = ClassNode {
        minor_version: jvms_file.minor_version,
        major_version: jvms_file.major_version,
        access: jvms_file.access_flags,
        name, signature, super_name, interfaces, source_file, source_debug,
        module, outer_class, outer_method_name, outer_method_desc,
        annotations, type_annotations, inner_classes, nest_host_class, nest_members,
        permitted_subclasses, record_components, fields, methods, attrs,
    };
    Ok(class_node)
}

fn field_from_jvms(class_context: &mut ClassNodeContext, field_info: &FieldInfo) -> AsmResult<FieldNode> {
    let name = class_context.read_utf8(field_info.name_index)?;
    let access = field_info.access_flags;
    let desc = class_context.read_utf8(field_info.descriptor_index)?;
    let mut signature = None;
    let mut value = None;
    let mut annotations = vec![];
    let mut type_annotations = vec![];
    let mut attrs = vec![];
    
    for (attribute_info, attribute) in class_context.read_attrs(&field_info.attributes)? {
        match attribute.as_ref() {
            Attribute::Signature(s) => signature = Some(Rc::clone(s)),
            Attribute::ConstantValue(v) => {
                value = match v {
                    ConstValue::Integer(i) => Some(FieldInitialValue::Integer(*i)),
                    ConstValue::Float(f) => Some(FieldInitialValue::Float(*f)),
                    ConstValue::Long(l) => Some(FieldInitialValue::Long(*l)),
                    ConstValue::Double(d) => Some(FieldInitialValue::Double(*d)),
                    ConstValue::String(s) => Some(FieldInitialValue::String(s.to_string())),
                    _ => AsmErr::ResolveNode(
                        format!("invalid constant value {:?} for field: {}", v, name)
                    ).e()?,
                }
            },
            Attribute::RuntimeVisibleAnnotations(an) => annotations.extend(an.clone()),
            Attribute::RuntimeInvisibleAnnotations(an) => annotations.extend(an.clone()),
            Attribute::RuntimeVisibleTypeAnnotations(tan) => type_annotations.extend(tan.clone()),
            Attribute::RuntimeInvisibleTypeAnnotations(tan) => type_annotations.extend(tan.clone()),
            Attribute::Unknown(v) => attrs.push(v.clone()),
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

fn method_from_jvms(class_context: &mut ClassNodeContext, method_info: Rc<MethodInfo>) -> AsmResult<MethodNode> {
    let jvms_file = Rc::clone(&class_context.jvms_file);
    let method_context = MethodNodeContext { jvms_file, method_info };
    let method_node = MethodNode {
        access: method_info.access_flags,
        name: class_context.read_utf8(method_info.name_index)?,
        desc: class_context.read_utf8(method_info.descriptor_index)?,
    };
    Ok(method_node)
}
