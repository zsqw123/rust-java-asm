use crate::attribute::Attribute;
use crate::{asm_visitor, asm_visitor_impl};

asm_visitor! {
    pub struct MethodVisitor<'a>
}

pub enum LocalVariableType<'a> {
    Primitive { opcodes: i8 },
    Object { type_str: &'a str },
}

// todo pub fn visitTypeAnnotation
asm_visitor_impl! {
    impl MethodVisitor<'_> {
        // -----------------------------------------------------------------------------------------------
        // Parameters, annotations and non standard attributes
        // -----------------------------------------------------------------------------------------------

        //
        // Visits a parameter of this method.
        //
        // - name, parameter name or {@literal null} if none is provided.
        // - access, the parameter's access flags, only {@code ACC_FINAL}, {@code ACC_SYNTHETIC}
        //     or/and {@code ACC_MANDATED} are allowed (see {@link Opcodes}).
        pub fn visit_parameter(&self, name: &str, access: u8) -> Option<()>;

        // Visits the default value of this annotation interface method.
        //
        // returns a visitor to the visit the actual default value of this annotation interface method, or
        //     {@literal null} if this visitor is not interested in visiting this default value. The
        //     'name' parameters passed to the methods of this annotation visitor are ignored. Moreover,
        //     exactly one visit method must be called on this annotation visitor, followed by visitEnd.
        pub fn visit_annotation_default(&self) -> Option<MethodVisitor>;

        // Visits an annotation of this method.
        //
        // @param descriptor the class descriptor of the annotation class.
        // @param visible {@literal true} if the annotation is visible at runtime.
        // @return a visitor to visit the annotation values, or {@literal null} if this visitor is not
        //     interested in visiting this annotation.
        pub fn visit_annotation(&self, descriptor: &str) -> Option<MethodVisitor>;

        // Visits the number of method parameters that can have annotations. By default (i.e. when this
        // method is not called), all the method parameters defined by the method descriptor can have
        // annotations.
        //
        // @param parameterCount the number of method parameters than can have annotations. This number
        //     must be less or equal than the number of parameter types in the method descriptor. It can
        //     be strictly less when a method has synthetic parameters and when these parameters are
        //     ignored when computing parameter indices for the purpose of parameter annotations (see
        //     https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.7.18).
        // @param visible {@literal true} to define the number of method parameters that can have
        //     annotations visible at runtime, {@literal false} to define the number of method parameters
        //     that can have annotations invisible at runtime.
        pub fn visit_annotable_parameter_count(&self, parameter_count: u32, visible: bool) -> Option<()>;

        // Visits an annotation of a parameter this method.
        //
        // @param parameter the parameter index. This index must be strictly smaller than the number of
        //     parameters in the method descriptor, and strictly smaller than the parameter count
        //     specified in {@link #visitAnnotableParameterCount}. Important note: <i>a parameter index i
        //     is not required to correspond to the i'th parameter descriptor in the method
        //     descriptor</i>, in particular in case of synthetic parameters (see
        //     https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.7.18).
        // @param descriptor the class descriptor of the annotation class.
        // @param visible {@literal true} if the annotation is visible at runtime.
        // @return a visitor to visit the annotation values, or {@literal null} if this visitor is not
        //     interested in visiting this annotation.
        pub fn visit_parameter_annotation(&self, parameter: u8, descriptor: &str, visable:bool) -> Option<MethodVisitor>;

        pub fn visit_attribute(&self, attribute: &Attribute) -> Option<()>;

        pub fn visit_code(&self, attribute: &Attribute) -> Option<()>;

        //
        // Visits the current state of the local variables and operand stack elements. This method must(*)
        // be called <i>just before</i> any instruction <b>i</b> that follows an unconditional branch
        // instruction such as GOTO or THROW, that is the target of a jump instruction, or that starts an
        // exception handler block. The visited types must describe the values of the local variables and
        // of the operand stack elements <i>just before</i> <b>i</b> is executed.<br>
        // <br>
        // (*) this is mandatory only for classes whose version is greater than or equal to {@link
        // Opcodes#V1_6}. <br>
        // <br>
        // The frames of a method must be given either in expanded form, or in compressed form (all frames
        // must use the same format, i.e. you must not mix expanded and compressed frames within a single
        // method):
        //
        // <ul>
        //   <li>In expanded form, all frames must have the F_NEW type.
        //   <li>In compressed form, frames are basically "deltas" from the state of the previous frame:
        //       <ul>
        //         <li>{@link Opcodes#F_SAME} representing frame with exactly the same locals as the
        //             previous frame and with the empty stack.
        //         <li>{@link Opcodes#F_SAME1} representing frame with exactly the same locals as the
        //             previous frame and with single value on the stack ( <code>numStack</code> is 1 and
        //             <code>stack[0]</code> contains value for the type of the stack item).
        //         <li>{@link Opcodes#F_APPEND} representing frame with current locals are the same as the
        //             locals in the previous frame, except that additional locals are defined (<code>
        //             numLocal</code> is 1, 2 or 3 and <code>local</code> elements contains values
        //             representing added types).
        //         <li>{@link Opcodes#F_CHOP} representing frame with current locals are the same as the
        //             locals in the previous frame, except that the last 1-3 locals are absent and with
        //             the empty stack (<code>numLocal</code> is 1, 2 or 3).
        //         <li>{@link Opcodes#F_FULL} representing complete frame data.
        //       </ul>
        // </ul>
        //
        // <br>
        // In both cases the first frame, corresponding to the method's parameters and access flags, is
        // implicit and must not be visited. Also, it is illegal to visit two or more frames for the same
        // code location (i.e., at least one instruction must be visited between two calls to visitFrame).
        //
        // @param type the type of this stack map frame. Must be {@link Opcodes#F_NEW} for expanded
        //     frames, or {@link Opcodes#F_FULL}, {@link Opcodes#F_APPEND}, {@link Opcodes#F_CHOP}, {@link
        //     Opcodes#F_SAME} or {@link Opcodes#F_APPEND}, {@link Opcodes#F_SAME1} for compressed frames.
        // @param numLocal the number of local variables in the visited frame. Long and double values
        //     count for one variable.
        // @param local the local variable types in this frame. This array must not be modified. Primitive
        //     types are represented by {@link Opcodes#TOP}, {@link Opcodes#INTEGER}, {@link
        //     Opcodes#FLOAT}, {@link Opcodes#LONG}, {@link Opcodes#DOUBLE}, {@link Opcodes#NULL} or
        //     {@link Opcodes#UNINITIALIZED_THIS} (long and double are represented by a single element).
        //     Reference types are represented by String objects (representing internal names, see {@link
        //     Type#getInternalName()}), and uninitialized types by Label objects (this label designates
        //     the NEW instruction that created this uninitialized value).
        // @param numStack the number of operand stack elements in the visited frame. Long and double
        //     values count for one stack element.
        // @param stack the operand stack types in this frame. This array must not be modified. Its
        //     content has the same format as the "local" array.
        // @throws IllegalStateException if a frame is visited just after another one, without any
        //     instruction between the two (unless this frame is a Opcodes#F_SAME frame, in which case it
        //     is silently ignored).
        //
        pub fn visit_frame(&self, frame_type: i8, num_local: u32,
            local: &[LocalVariableType], num_stack: u32, stack: &[LocalVariableType]) -> Option<()>;

        // Visits a zero operand instruction.
        //
        // @param opcode the opcode of the instruction to be visited. This opcode is either NOP,
        //     ACONST_NULL, ICONST_M1, ICONST_0, ICONST_1, ICONST_2, ICONST_3, ICONST_4, ICONST_5,
        //     LCONST_0, LCONST_1, FCONST_0, FCONST_1, FCONST_2, DCONST_0, DCONST_1, IALOAD, LALOAD,
        //     FALOAD, DALOAD, AALOAD, BALOAD, CALOAD, SALOAD, IASTORE, LASTORE, FASTORE, DASTORE,
        //     AASTORE, BASTORE, CASTORE, SASTORE, POP, POP2, DUP, DUP_X1, DUP_X2, DUP2, DUP2_X1, DUP2_X2,
        //     SWAP, IADD, LADD, FADD, DADD, ISUB, LSUB, FSUB, DSUB, IMUL, LMUL, FMUL, DMUL, IDIV, LDIV,
        //     FDIV, DDIV, IREM, LREM, FREM, DREM, INEG, LNEG, FNEG, DNEG, ISHL, LSHL, ISHR, LSHR, IUSHR,
        //     LUSHR, IAND, LAND, IOR, LOR, IXOR, LXOR, I2L, I2F, I2D, L2I, L2F, L2D, F2I, F2L, F2D, D2I,
        //     D2L, D2F, I2B, I2C, I2S, LCMP, FCMPL, FCMPG, DCMPL, DCMPG, IRETURN, LRETURN, FRETURN,
        //     DRETURN, ARETURN, RETURN, ARRAYLENGTH, ATHROW, MONITORENTER, or MONITOREXIT.
        pub fn visit_insn(&self, opcode: u8) -> Option<()>;

        // Visits an instruction with a single int operand.
        // @param opcode the opcode of the instruction to be visited. This opcode is either BIPUSH, SIPUSH
        //     or NEWARRAY.
        // @param operand the operand of the instruction to be visited.<br>
        //     When opcode is BIPUSH, operand value should be between Byte.MIN_VALUE and Byte.MAX_VALUE.
        //     <br>
        //     When opcode is SIPUSH, operand value should be between Short.MIN_VALUE and Short.MAX_VALUE.
        //     <br>
        //     When opcode is NEWARRAY, operand value should be one of {@link Opcodes#T_BOOLEAN}, {@link
        //     Opcodes#T_CHAR}, {@link Opcodes#T_FLOAT}, {@link Opcodes#T_DOUBLE}, {@link Opcodes#T_BYTE},
        //     {@link Opcodes#T_SHORT}, {@link Opcodes#T_INT} or {@link Opcodes#T_LONG}.
        pub fn visit_int_insn(&self, opcode: u8, operand: i32) -> Option<()>;

        // Visits a local variable instruction. A local variable instruction is an instruction that
        // loads or stores the value of a local variable.
        // @param opcode the opcode of the local variable instruction to be visited. This opcode is either
        //     ILOAD, LLOAD, FLOAD, DLOAD, ALOAD, ISTORE, LSTORE, FSTORE, DSTORE, ASTORE or RET.
        // @param varIndex the operand of the instruction to be visited. This operand is the index of a
        //     local variable.
        pub fn visit_var_insn(&self, opcode: u8, var_index: u32) -> Option<()>;

        // Visits a type instruction. A type instruction is an instruction that takes the internal name of
        // a class as parameter (see {@link Type#getInternalName()}).
        //
        // @param opcode the opcode of the type instruction to be visited. This opcode is either NEW,
        //     ANEWARRAY, CHECKCAST or INSTANCEOF.
        // @param type the operand of the instruction to be visited. This operand must be the internal
        //     name of an object or array class (see {@link Type#getInternalName()}).
        pub fn visit_type_insn(&self, opcode: u8, insn_type: &str) -> Option<()>;
        
        // Visits a field instruction. A field instruction is an instruction that loads or stores the
        // value of a field of an object.
        // 
        // @param opcode the opcode of the type instruction to be visited. This opcode is either
        // GETSTATIC, PUTSTATIC, GETFIELD or PUTFIELD.
        // @param owner the internal name of the field's owner class (see {@link Type#getInternalName()}).
        // @param name the field's name.
        // @param descriptor the field's descriptor (see {@link Type}).
        pub fn visit_field_insn(&self,opcode: u8, owner: &str, name: &str) -> Option<()>;
        
        
    }
}
