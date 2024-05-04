use java_asm_internal::read::jvms::FromReadContext;

pub mod type_annotation;

// annotation {
//     u2 type_index;
//     u2 num_element_value_pairs;
//     {   u2            element_name_index;
//         element_value value;
//     } element_value_pairs[num_element_value_pairs];
// }
#[derive(Clone, Debug)]
pub struct AnnotationInfo {
    pub type_index: u16,
    pub num_element_value_pairs: u16,
    pub element_value_pairs: Vec<AnnotationElement>,
}

// {   
//     u2            element_name_index;
//     element_value value;
// } 
#[derive(Clone, Debug, FromReadContext)]
pub struct AnnotationElement {
    pub element_name_index: u16,
    pub value: AnnotationElementValueInfo,
}

// element_value {
//     u1 tag;
//     union {
//         u2 const_value_index;
// 
//         {   u2 type_name_index;
//             u2 const_name_index;
//         } enum_const_value;
// 
//         u2 class_info_index;
// 
//         annotation annotation_value;
// 
//         {   u2            num_values;
//             element_value values[num_values];
//         } array_value;
//     } value;
// }
#[derive(Clone, Debug)]
pub struct AnnotationElementValueInfo {
    pub tag: u8,
    pub value: AnnotationElementValue,
}

// union {
//     u2 const_value_index;
// 
//     {   u2 type_name_index; // CONSTANT_Utf8_info 
//         u2 const_name_index; // CONSTANT_Utf8_info 
//     } enum_const_value;
// 
//     u2 class_info_index; // CONSTANT_Utf8_info 
// 
//     annotation annotation_value;
// 
//     {   u2            num_values;
//         element_value values[num_values];
//     } array_value;
// } value;
#[derive(Clone, Debug)]
pub enum AnnotationElementValue {
    Const { const_value_index: u16 },
    EnumConst { type_name_index: u16, const_name_index: u16 },
    Class { class_info_index: u16 },
    Annotation { annotation_value: AnnotationInfo },
    Array { num_values: u16, values: Vec<AnnotationElementValueInfo> },
}

// {   
//     u2         num_annotations;
//     annotation annotations[num_annotations];
// }
#[derive(Clone, Debug)]
pub struct ParameterAnnotationInfo {
    pub num_annotations: u16,
    pub annotations: Vec<AnnotationInfo>,
}
