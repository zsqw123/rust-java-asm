// type_annotation {
//     u1 target_type;
//     union {
//         type_parameter_target;
//         supertype_target;
//         type_parameter_bound_target;
//         empty_target;
//         formal_parameter_target;
//         throws_target;
//         localvar_target;
//         catch_target;
//         offset_target;
//         type_argument_target;
//     } target_info;
//     type_path target_path;
//     u2        type_index;
//     u2        num_element_value_pairs;
//     {   u2            element_name_index;
//         element_value value;
//     } element_value_pairs[num_element_value_pairs];
// }
#[derive(Clone, Debug)]
pub struct TypeAnnotation {
    pub target_type: u8,
    pub target_info: TypeAnnotationTargetInfo,
    pub target_path: TypeAnnotationTargetPath,
}

#[derive(Clone, Debug)]
pub enum TypeAnnotationTargetInfo {
    TypeParameter { type_parameter_index: u8 },
    SuperType { supertype_index: u16 },
    TypeParameterBound { type_parameter_index: u8, bound_index: u8 },
    Empty,
    FormalParameter { formal_parameter_index: u8 },
    Throws { throws_type_index: u16 },
    LocalVar { table_length: u16, table: Vec<TypeAnnotationTargetInfoLocalVarTable> },
    Catch { exception_table_index: u16 },
    Offset { offset: u16 },
    TypeArgument { offset: u16, type_argument_index: u8 },
}

// {
//     u2 start_pc;
//     u2 length;
//     u2 index;
// }
#[derive(Clone, Copy, Debug)]
pub struct TypeAnnotationTargetInfoLocalVarTable {
    pub start_pc: u16,
    pub length: u16,
    pub index: u16,
}

// type_path {
//     u1 path_length;
//     {   u1 type_path_kind;
//         u1 type_argument_index;
//     } path[path_length];
// }
#[derive(Clone, Debug)]
pub struct TypeAnnotationTargetPath {
    path_length: u8,
    path: Vec<TypeAnnotationTargetPathInfo>,
}

// {   
//     u1 type_path_kind;
//     u1 type_argument_index;
// }
#[derive(Clone, Copy, Debug)]
pub struct TypeAnnotationTargetPathInfo {
    type_path_kind: u8,
    type_argument_index: u8,
}
