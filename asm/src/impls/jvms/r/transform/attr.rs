use java_asm_internal::err::AsmResult;
use java_asm_internal::read::jvms::ReadContext;

use crate::constants::Constants;
use crate::impls::jvms::r::transform::transform_attrs;
use crate::impls::jvms::r::util::read_utf8_from_cp;
use crate::jvms::attr::{Attribute, ExceptionTable, StackMapFrame};
use crate::jvms::element::{AttributeInfo, CPInfo};

pub(crate) fn transform_attr(attribute_info: &AttributeInfo, cp: &Vec<CPInfo>) -> AsmResult<AttributeInfo> {
    let attribute_name_index = attribute_info.attribute_name_index;
    let attribute_length = attribute_info.attribute_length;
    let info = attribute_info.info.clone();
    let Attribute::Custom(bytes) = info else { return Ok(attribute_info.clone()); };
    let mut context = ReadContext { bytes: &bytes, index: &mut 0 };
    let utf8 = read_utf8_from_cp(attribute_name_index as usize, cp)?;
    let attr = match utf8.as_str() {
        Constants::CONSTANT_VALUE => Attribute::ConstantValue {
            constantvalue_index: context.read()?,
        },
        Constants::CODE => {
            let max_stack: u16 = context.read()?;
            let max_locals: u16 = context.read()?;
            let code_length: u32 = context.read()?;
            let code: Vec<u8> = context.read_vec(code_length as usize)?;
            let exception_table_length: u16 = context.read()?;
            let exception_table: Vec<ExceptionTable> = context.read_vec(exception_table_length as usize)?;
            let attributes_count: u16 = context.read()?;
            let mut attributes: Vec<AttributeInfo> = context.read_vec(attributes_count as usize)?;
            transform_attrs(&mut attributes, cp)?;
            Attribute::Code {
                max_stack, max_locals,
                code_length, code,
                exception_table_length, exception_table,
                attributes_count, attributes,
            }
        },
        Constants::STACK_MAP_TABLE => {
            let number_of_entries: u16 = context.read()?;
            let entries: Vec<StackMapFrame> = context.read_vec(number_of_entries as usize)?;
            Attribute::StackMapTable { number_of_entries, entries }
        },
        Constants::EXCEPTIONS => {
            let number_of_exceptions = context.read()?;
            let exception_index_table = context.read_vec(number_of_exceptions as usize)?;
            Attribute::Exceptions { number_of_exceptions, exception_index_table }
        },
        Constants::INNER_CLASSES => {
            let number_of_classes = context.read()?;
            let classes = context.read_vec(number_of_classes as usize)?;
            Attribute::InnerClasses { number_of_classes, classes, }
        }
        Constants::ENCLOSING_METHOD => Attribute::EnclosingMethod {
            class_index: context.read()?, method_index: context.read()?,    
        },
        Constants::SYNTHETIC => Attribute::Synthetic,
        Constants::SIGNATURE => Attribute::Signature {
            signature_index: context.read()?,
        },
        Constants::SOURCE_FILE => Attribute::SourceFile {
            sourcefile_index: context.read()?,
        },
        Constants::SOURCE_DEBUG_EXTENSION => Attribute::SourceDebugExtension {
            debug_extension: bytes,
        },
        Constants::LINE_NUMBER_TABLE => {
            let line_number_table_length = context.read()?;
            let line_number_table = context.read_vec(line_number_table_length as usize)?;
            Attribute::LineNumberTable { line_number_table_length, line_number_table }
        },
        Constants::LOCAL_VARIABLE_TABLE => {
            let local_variable_table_length = context.read()?;
            let local_variable_table = context.read_vec(local_variable_table_length as usize)?;
            Attribute::LocalVariableTable { local_variable_table_length, local_variable_table }
        },
        Constants::LOCAL_VARIABLE_TYPE_TABLE => {
            let local_variable_type_table_length = context.read()?;
            let local_variable_table = context.read_vec(local_variable_type_table_length as usize)?;
            Attribute::LocalVariableTypeTable { local_variable_type_table_length, local_variable_table }
        },
        Constants::DEPRECATED => Attribute::Deprecated,
        Constants::RUNTIME_VISIBLE_ANNOTATIONS => {
            let num_annotations = context.read()?;
            let annotations = context.read_vec(num_annotations as usize)?;
            Attribute::RuntimeVisibleAnnotations { num_annotations, annotations }
        },
        Constants::RUNTIME_INVISIBLE_ANNOTATIONS => {
            let num_annotations = context.read()?;
            let annotations = context.read_vec(num_annotations as usize)?;
            Attribute::RuntimeInvisibleAnnotations { num_annotations, annotations }
        },
        Constants::RUNTIME_VISIBLE_PARAMETER_ANNOTATIONS => {
            let num_parameters = context.read()?;
            let parameter_annotations = context.read_vec(num_parameters as usize)?;
            Attribute::RuntimeVisibleParameterAnnotations { num_parameters, parameter_annotations }
        },
        Constants::RUNTIME_INVISIBLE_PARAMETER_ANNOTATIONS => {
            let num_parameters = context.read()?;
            let parameter_annotations = context.read_vec(num_parameters as usize)?;
            Attribute::RuntimeInvisibleParameterAnnotations { num_parameters, parameter_annotations }
        },
        Constants::RUNTIME_VISIBLE_TYPE_ANNOTATIONS => {
            let num_parameters = context.read()?;
            let annotations = context.read_vec(num_parameters as usize)?;
            Attribute::RuntimeVisibleTypeAnnotations { num_parameters, annotations }
        },
        Constants::RUNTIME_INVISIBLE_TYPE_ANNOTATIONS => {
            let num_parameters = context.read()?;
            let annotations = context.read_vec(num_parameters as usize)?;
            Attribute::RuntimeVisibleTypeAnnotations { num_parameters, annotations }
        },
        Constants::ANNOTATION_DEFAULT => Attribute::AnnotationDefault { default_value: context.read()? },
        Constants::BOOTSTRAP_METHODS => {
            let num_bootstrap_methods = context.read()?;
            let bootstrap_methods = context.read_vec(num_bootstrap_methods as usize)?;
            Attribute::BootstrapMethods { num_bootstrap_methods, bootstrap_methods }
        },
        Constants::METHOD_PARAMETERS => {
            let parameters_count = context.read()?;
            let parameters = context.read_vec(parameters_count as usize)?;
            Attribute::MethodParameters { parameters_count, parameters }
        },
        Constants::MODULE => {
            let module_name_index: u16 = context.read()?;
            let module_flags: u16 = context.read()?;
            let module_version_index: u16 = context.read()?;
            let requires_count: u16 = context.read()?;
            let requires = context.read_vec(requires_count as usize)?;
            let exports_count: u16 = context.read()?;
            let exports = context.read_vec(exports_count as usize)?;
            let opens_count: u16 = context.read()?;
            let opens = context.read_vec(opens_count as usize)?;
            let uses_count: u16 = context.read()?;
            let uses_index: Vec<u16> = context.read_vec(uses_count as usize)?;
            let provides_count: u16 = context.read()?;
            let provides = context.read_vec(provides_count as usize)?;
            Attribute::Module {
                module_name_index, module_flags, module_version_index,
                requires_count, requires, exports_count, exports,
                opens_count, opens, uses_count, uses_index, provides_count, provides,
            }
        },
        Constants::MODULE_PACKAGES => {
            let package_count = context.read()?;
            let package_index = context.read_vec(package_count as usize)?;
            Attribute::ModulePackages { package_count, package_index }
        },
        Constants::MODULE_MAIN_CLASS => Attribute::ModuleMainClass { main_class_index: context.read()? },
        Constants::NEST_HOST => Attribute::NestHost { host_class_index: context.read()? },
        Constants::NEST_MEMBERS => {
            let number_of_classes = context.read()?;
            let classes = context.read_vec(number_of_classes as usize)?;
            Attribute::NestMembers { number_of_classes, classes }
        },
        Constants::RECORD => {
            let components_count = context.read()?;
            let components = context.read_vec(components_count as usize)?;
            Attribute::Record { components_count, components }
        },
        Constants::PERMITTED_SUBCLASSES => {
            let number_of_classes = context.read()?;
            let classes = context.read_vec(number_of_classes as usize)?;
            Attribute::PermittedSubclasses { number_of_classes, classes }
        },
        _ => Attribute::Custom(bytes),
    };
    let attribute_info = AttributeInfo {
        attribute_name_index,
        attribute_length,
        info: attr,
    };
    Ok(attribute_info)
}
