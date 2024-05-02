use crate::constant_dynamic::ConstantDynamic;

pub struct ClassReaderOption<'a> {
    header: u32,
    constant_pool_offsets: &'a Vec<u32>,
    constant_utf8_values: Vec<String>,
    max_string_length: u32,
    constant_dynamic_values: Option<Vec<ConstantDynamic<'a>>>,
}

/// consts for [ClassReader]
impl ClassReaderOption<'_> {
    /// A flag to skip the Code attributes. If this flag is set the Code attributes are neither parsed
    /// nor visited.
    const SKIP_CODE: u16 = 1;

    /// A flag to skip the SourceFile, SourceDebugExtension, LocalVariableTable,
    /// LocalVariableTypeTable, LineNumberTable and MethodParameters attributes. If this flag is set
    /// these attributes are neither parsed nor visited (i.e. {@link ClassVisitor#visitSource}, {@link
    /// MethodVisitor#visitLocalVariable}, {@link MethodVisitor#visitLineNumber} and {@link
    /// MethodVisitor#visitParameter} are not called).
    pub const SKIP_DEBUG: u16 = 2;

    /// A flag to skip the StackMap and StackMapTable attributes. If this flag is set these attributes
    /// are neither parsed nor visited (i.e. {@link MethodVisitor#visitFrame} is not called). This flag
    /// is useful when the {@link ClassWriter#COMPUTE_FRAMES} option is used: it avoids visiting frames
    /// that will be ignored and recomputed from scratch.
    pub const SKIP_FRAMES: u16 = 4;

    /// A flag to expand the stack map frames. By default stack map frames are visited in their
    /// original format (i.e. "expanded" for classes whose version is less than V1_6, and "compressed"
    /// for the other classes). If this flag is set, stack map frames are always visited in expanded
    /// format (this option adds a decompression/compression step in ClassReader and ClassWriter which
    /// degrades performance quite a lot).
    pub const EXPAND_FRAMES: u16 = 8;

    /// A flag to expand the ASM specific instructions into an equivalent sequence of standard bytecode
    /// instructions. When resolving a forward jump it may happen that the signed 2 bytes offset
    /// reserved for it is not sufficient to store the bytecode offset. In this case the jump
    /// instruction is replaced with a temporary ASM specific instruction using an unsigned 2 bytes
    /// offset (see {@link Label#resolve}). This internal flag is used to re-read classes containing
    /// such instructions, in order to replace them with standard instructions. In addition, when this
    /// flag is used, goto_w and jsr_w are <i>not</i> converted into goto and jsr, to make sure that
    /// infinite loops where a goto_w is replaced with a goto in ClassReader and converted back to a
    /// goto_w in ClassWriter cannot occur.
    pub const EXPAND_ASM_INSNS: u16 = 256;

    /// The maximum size of array to allocate.
    const MAX_BUFFER_SIZE: u32 = 1024 * 1024;

    /// The size of the temporary byte array used to read class input streams chunk by chunk.
    const INPUT_STREAM_DATA_CHUNK_SIZE: u16 = 4096;
}

impl ClassReaderOption<'_> {
    
}
