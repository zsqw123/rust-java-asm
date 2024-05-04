use java_asm_internal::read::jvms::FromReadContext;

// {   
//     u2 requires_index;
//     u2 requires_flags;
//     u2 requires_version_index;
// }
#[derive(Clone, Copy, Debug, FromReadContext)]
pub struct ModuleRequires {
    pub requires_index: u16,
    pub requires_flags: u16,
    pub requires_version_index: u16,
}

// {   
//     u2 exports_index;
//     u2 exports_flags;
//     u2 exports_to_count;
//     u2 exports_to_index[exports_to_count];
// }
#[derive(Clone, Debug, FromReadContext)]
pub struct ModuleExports {
    pub exports_index: u16,
    pub exports_flags: u16,
    pub exports_to_count: u16,
    #[index(exports_to_count)]
    pub exports_to_index: Vec<u16>,
}

// {   
//     u2 opens_index;
//     u2 opens_flags;
//     u2 opens_to_count;
//     u2 opens_to_index[opens_to_count];
// }
#[derive(Clone, Debug, FromReadContext)]
pub struct ModuleOpens {
    pub opens_index: u16,
    pub opens_flags: u16,
    pub opens_to_count: u16,
    #[index(opens_to_count)]
    pub opens_to_index: Vec<u16>,
}


// {   
//     u2 provides_index;
//     u2 provides_with_count;
//     u2 provides_with_index[opens_to_count];
// }
#[derive(Clone, Debug, FromReadContext)]
pub struct ModuleProvides {
    pub provides_index: u16,
    pub provides_with_count: u16,
    #[index(provides_with_count)]
    pub provides_with_index: Vec<u16>,
}

