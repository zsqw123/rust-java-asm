use crate::{asm_visitor, asm_visitor_impl};
use crate::annotation::AnnotationVisitor;
use crate::attribute::Attribute;
use crate::constant_dynamic::ConstantDynamic;
use crate::handle::Handle;
use crate::label::Label;
use crate::type_path::TypePath;

asm_visitor! {
    pub struct MethodVisitor<'a>
}

pub enum LocalVariableType<'a> {
    Primitive { opcodes: i8 },
    Object { type_str: &'a str },
}

pub enum BootstrapMethodArgument<'a> {
    ConstantDynamic(ConstantDynamic<'a>)
}

pub enum LdcValue {
    Integer,
    Float,
    Class,
}

// todo pub fn visitTypeAnnotation
asm_visitor_impl! {
    impl MethodVisitor<'_> {
        // -----------------------------------------------------------------------------------------------
        // Parameters, annotations and non standard attributes
        // -----------------------------------------------------------------------------------------------

        /// Visits a parameter of this method.
        ///
        /// - name, parameter name or {@literal null} if none is provided.
        /// - access, the parameter's access flags, only {@code ACC_FINAL}, {@code ACC_SYNTHETIC}
        ///     or/and {@code ACC_MANDATED} are allowed (see {@link Opcodes}).
        pub fn visit_parameter(&self, name: &str, access: u8) -> Option<()>;

        /// Visits the default value of this annotation interface method.
        ///
        /// returns a visitor to the visit the actual default value of this annotation interface method, or
        ///     {@literal null} if this visitor is not interested in visiting this default value. The
        ///     'name' parameters passed to the methods of this annotation visitor are ignored. Moreover,
        ///     exactly one visit method must be called on this annotation visitor, followed by visitEnd.
        pub fn visit_annotation_default(&self) -> Option<MethodVisitor>;

        /// Visits an annotation of this method.
        ///
        /// - descriptor the class descriptor of the annotation class.
        /// - visible {@literal true} if the annotation is visible at runtime.
        ///
        /// returns a visitor to visit the annotation values, or {@literal null} if this visitor is not
        ///     interested in visiting this annotation.
        pub fn visit_annotation(&self, descriptor: &str) -> Option<MethodVisitor>;

        /// Visits the number of method parameters that can have annotations. By default (i.e. when this
        /// method is not called), all the method parameters defined by the method descriptor can have
        /// annotations.
        ///
        /// - parameterCount the number of method parameters than can have annotations. This number
        ///     must be less or equal than the number of parameter types in the method descriptor. It can
        ///     be strictly less when a method has synthetic parameters and when these parameters are
        ///     ignored when computing parameter indices for the purpose of parameter annotations (see
        ///     https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.7.18).
        /// - visible {@literal true} to define the number of method parameters that can have
        ///     annotations visible at runtime, {@literal false} to define the number of method parameters
        ///     that can have annotations invisible at runtime.
        pub fn visit_annotable_parameter_count(&self, parameter_count: u32, visible: bool) -> Option<()>;

        /// Visits an annotation of a parameter this method.
        ///
        /// - parameter the parameter index. This index must be strictly smaller than the number of
        ///     parameters in the method descriptor, and strictly smaller than the parameter count
        ///     specified in {@link #visitAnnotableParameterCount}. Important note: <i>a parameter index i
        ///     is not required to correspond to the i'th parameter descriptor in the method
        ///     descriptor</i>, in particular in case of synthetic parameters (see
        ///     https://docs.oracle.com/javase/specs/jvms/se9/html/jvms-4.html#jvms-4.7.18).
        /// - descriptor the class descriptor of the annotation class.
        /// - visible {@literal true} if the annotation is visible at runtime.
        ///
        /// returns a visitor to visit the annotation values, or {@literal null} if this visitor is not
        ///     interested in visiting this annotation.
        pub fn visit_parameter_annotation(&self, parameter: u8, descriptor: &str, visable:bool) -> Option<MethodVisitor>;

        pub fn visit_attribute(&self, attribute: &Attribute) -> Option<()>;

        pub fn visit_code(&self, attribute: &Attribute) -> Option<()>;

        /// Visits the current state of the local variables and operand stack elements. This method must(*)
        /// be called <i>just before</i> any instruction <b>i</b> that follows an unconditional branch
        /// instruction such as GOTO or THROW, that is the target of a jump instruction, or that starts an
        /// exception handler block. The visited types must describe the values of the local variables and
        /// of the operand stack elements <i>just before</i> <b>i</b> is executed.<br>
        /// <br>
        /// (*) this is mandatory only for classes whose version is greater than or equal to {@link
        /// Opcodes#V1_6}. <br>
        /// <br>
        /// The frames of a method must be given either in expanded form, or in compressed form (all frames
        /// must use the same format, i.e. you must not mix expanded and compressed frames within a single
        /// method):
        ///
        /// <ul>
        ///   <li>In expanded form, all frames must have the F_NEW type.
        ///   <li>In compressed form, frames are basically "deltas" from the state of the previous frame:
        ///       <ul>
        ///         <li>{@link Opcodes#F_SAME} representing frame with exactly the same locals as the
        ///             previous frame and with the empty stack.
        ///         <li>{@link Opcodes#F_SAME1} representing frame with exactly the same locals as the
        ///             previous frame and with single value on the stack ( <code>numStack</code> is 1 and
        ///             <code>stack[0]</code> contains value for the type of the stack item).
        ///         <li>{@link Opcodes#F_APPEND} representing frame with current locals are the same as the
        ///             locals in the previous frame, except that additional locals are defined (<code>
        ///             numLocal</code> is 1, 2 or 3 and <code>local</code> elements contains values
        ///             representing added types).
        ///         <li>{@link Opcodes#F_CHOP} representing frame with current locals are the same as the
        ///             locals in the previous frame, except that the last 1-3 locals are absent and with
        ///             the empty stack (<code>numLocal</code> is 1, 2 or 3).
        ///         <li>{@link Opcodes#F_FULL} representing complete frame data.
        ///       </ul>
        /// </ul>
        ///
        /// <br>
        /// In both cases the first frame, corresponding to the method's parameters and access flags, is
        /// implicit and must not be visited. Also, it is illegal to visit two or more frames for the same
        /// code location (i.e., at least one instruction must be visited between two calls to visitFrame).
        ///
        /// - type the type of this stack map frame. Must be {@link Opcodes#F_NEW} for expanded
        ///     frames, or {@link Opcodes#F_FULL}, {@link Opcodes#F_APPEND}, {@link Opcodes#F_CHOP}, {@link
        ///     Opcodes#F_SAME} or {@link Opcodes#F_APPEND}, {@link Opcodes#F_SAME1} for compressed frames.
        /// - numLocal the number of local variables in the visited frame. Long and double values
        ///     count for one variable.
        /// - local the local variable types in this frame. This array must not be modified. Primitive
        ///     types are represented by {@link Opcodes#TOP}, {@link Opcodes#INTEGER}, {@link
        ///     Opcodes#FLOAT}, {@link Opcodes#LONG}, {@link Opcodes#DOUBLE}, {@link Opcodes#NULL} or
        ///     {@link Opcodes#UNINITIALIZED_THIS} (long and double are represented by a single element).
        ///     Reference types are represented by String objects (representing internal names, see {@link
        ///     Type#getInternalName()}), and uninitialized types by Label objects (this label designates
        ///     the NEW instruction that created this uninitialized value).
        /// - numStack the number of operand stack elements in the visited frame. Long and double
        ///     values count for one stack element.
        /// - stack the operand stack types in this frame. This array must not be modified. Its
        ///     content has the same format as the "local" array.
        ///
        /// @throws IllegalStateException if a frame is visited just after another one, without any
        ///     instruction between the two (unless this frame is a Opcodes#F_SAME frame, in which case it
        ///     is silently ignored).
        pub fn visit_frame(&self, frame_type: i8, num_local: u32,
            local: &[LocalVariableType], num_stack: u32, stack: &[LocalVariableType]) -> Option<()>;

        // -----------------------------------------------------------------------------------------------
        // Normal instructions
        // -----------------------------------------------------------------------------------------------

        /// Visits a zero operand instruction.
        ///
        /// - opcode, the opcode of the instruction to be visited. This opcode is either NOP,
        ///     ACONST_NULL, ICONST_M1, ICONST_0, ICONST_1, ICONST_2, ICONST_3, ICONST_4, ICONST_5,
        ///     LCONST_0, LCONST_1, FCONST_0, FCONST_1, FCONST_2, DCONST_0, DCONST_1, IALOAD, LALOAD,
        ///     FALOAD, DALOAD, AALOAD, BALOAD, CALOAD, SALOAD, IASTORE, LASTORE, FASTORE, DASTORE,
        ///     AASTORE, BASTORE, CASTORE, SASTORE, POP, POP2, DUP, DUP_X1, DUP_X2, DUP2, DUP2_X1, DUP2_X2,
        ///     SWAP, IADD, LADD, FADD, DADD, ISUB, LSUB, FSUB, DSUB, IMUL, LMUL, FMUL, DMUL, IDIV, LDIV,
        ///     FDIV, DDIV, IREM, LREM, FREM, DREM, INEG, LNEG, FNEG, DNEG, ISHL, LSHL, ISHR, LSHR, IUSHR,
        ///     LUSHR, IAND, LAND, IOR, LOR, IXOR, LXOR, I2L, I2F, I2D, L2I, L2F, L2D, F2I, F2L, F2D, D2I,
        ///     D2L, D2F, I2B, I2C, I2S, LCMP, FCMPL, FCMPG, DCMPL, DCMPG, IRETURN, LRETURN, FRETURN,
        ///     DRETURN, ARETURN, RETURN, ARRAYLENGTH, ATHROW, MONITORENTER, or MONITOREXIT.
        pub fn visit_insn(&self, opcode: u8) -> Option<()>;

        /// Visits an instruction with a single int operand.
        /// - opcode, the opcode of the instruction to be visited. This opcode is either BIPUSH, SIPUSH
        ///     or NEWARRAY.
        /// - operand, the operand of the instruction to be visited.<br>
        ///     When opcode is BIPUSH, operand value should be between Byte.MIN_VALUE and Byte.MAX_VALUE.
        ///     <br>
        ///     When opcode is SIPUSH, operand value should be between Short.MIN_VALUE and Short.MAX_VALUE.
        ///     <br>
        ///     When opcode is NEWARRAY, operand value should be one of {@link Opcodes#T_BOOLEAN}, {@link
        ///     Opcodes#T_CHAR}, {@link Opcodes#T_FLOAT}, {@link Opcodes#T_DOUBLE}, {@link Opcodes#T_BYTE},
        ///     {@link Opcodes#T_SHORT}, {@link Opcodes#T_INT} or {@link Opcodes#T_LONG}.
        pub fn visit_int_insn(&self, opcode: u8, operand: i32) -> Option<()>;

        /// Visits a local variable instruction. A local variable instruction is an instruction that
        /// loads or stores the value of a local variable.
        /// - opcode, the opcode of the local variable instruction to be visited. This opcode is either
        ///     ILOAD, LLOAD, FLOAD, DLOAD, ALOAD, ISTORE, LSTORE, FSTORE, DSTORE, ASTORE or RET.
        /// - varIndex, the operand of the instruction to be visited. This operand is the index of a
        ///     local variable.
        pub fn visit_var_insn(&self, opcode: u8, var_index: u32) -> Option<()>;

        /// Visits a type instruction. A type instruction is an instruction that takes the internal name of
        /// a class as parameter (see {@link Type#getInternalName()}).
        ///
        /// - opcode, the opcode of the type instruction to be visited. This opcode is either NEW,
        ///     ANEWARRAY, CHECKCAST or INSTANCEOF.
        /// - insn_type, the operand of the instruction to be visited. This operand must be the internal
        ///     name of an object or array class (see {@link Type#getInternalName()}).
        pub fn visit_type_insn(&self, opcode: u8, insn_type: &str) -> Option<()>;

        /// Visits a field instruction. A field instruction is an instruction that loads or stores the
        /// value of a field of an object.
        ///
        /// - opcode, the opcode of the type instruction to be visited. This opcode is either
        /// GETSTATIC, PUTSTATIC, GETFIELD or PUTFIELD.
        /// - owner, the internal name of the field's owner class (see {@link Type#getInternalName()}).
        /// - name, the field's name.
        /// - descriptor, the field's descriptor (see {@link Type}).
        pub fn visit_field_insn(&self, opcode: u8, owner: &str, name: &str) -> Option<()>;

        /// Visits a method instruction. A method instruction is an instruction that invokes a method.
        ///
        /// - opcode the opcode of the type instruction to be visited. This opcode is either
        ///     INVOKEVIRTUAL, INVOKESPECIAL, INVOKESTATIC or INVOKEINTERFACE.
        /// - owner the internal name of the method's owner class (see {@link
        ///     Type#getInternalName()}).
        /// - name the method's name.
        /// - descriptor the method's descriptor (see {@link Type}).
        /// - isInterface if the method's owner class is an interface.
        pub fn visit_method_insn(&self, opcode: u8, owner: &str, name: &str,
            descriptor: &str, is_interface: bool) -> Option<()>;

        /// Visits an invokedynamic instruction.
        ///
        /// - name the method's name.
        /// - descriptor the method's descriptor (see {@link Type}).
        /// - bootstrap_method_handle the bootstrap method.
        /// - bootstrap_method_arguments the bootstrap method constant arguments. Each argument must be
        ///     an {@link Integer}, {@link Float}, {@link Long}, {@link Double}, {@link String}, {@link
        ///     Type}, {@link Handle} or {@link ConstantDynamic} value. This method is allowed to modify
        ///     the content of the array so a caller should expect that this array may change.
        pub fn visit_invoke_dynamic_insn(&self, name: &str, descriptor: &str,
            bootstrap_method_handle: &Handle, bootstrap_method_arguments: &[&BootstrapMethodArgument]) -> Option<()>;

        /// Visits a jump instruction. A jump instruction is an instruction that may jump to another
        /// instruction.
        ///
        /// - opcode the opcode of the type instruction to be visited. This opcode is either IFEQ,
        /// IFNE, IFLT, IFGE, IFGT, IFLE, IF_ICMPEQ, IF_ICMPNE, IF_ICMPLT, IF_ICMPGE, IF_ICMPGT,
        /// IF_ICMPLE, IF_ACMPEQ, IF_ACMPNE, GOTO, JSR, IFNULL or IFNONNULL.
        /// - label the operand of the instruction to be visited. This operand is a label that
        /// designates the instruction to which the jump instruction may jump.
        pub fn visit_jump_insn(&self, opcode: u8, label: &Label) -> Option<()>;

        /// Visits a label. A label designates the instruction that will be visited just after it.
        ///
        /// - label a [Label] object.
        pub fn visit_label(&self, label: &Label) -> Option<()>;

        // -----------------------------------------------------------------------------------------------
        // Special instructions
        // -----------------------------------------------------------------------------------------------

        /// Visits a LDC instruction. Note that new constant types may be added in future versions of the
        /// Java Virtual Machine. To easily detect new constant types, implementations of this method
        /// should check for unexpected constant types, like this:
        ///
        /// ```java
        /// if (cst instanceof Integer) {
        ///     // ...
        /// } else if (cst instanceof Float) {
        ///     // ...
        /// } else if (cst instanceof Long) {
        ///     // ...
        /// } else if (cst instanceof Double) {
        ///     // ...
        /// } else if (cst instanceof String) {
        ///     // ...
        /// } else if (cst instanceof Type) {
        ///     int sort = ((Type) cst).getSort();
        ///     if (sort == Type.OBJECT) {
        ///         // ...
        ///     } else if (sort == Type.ARRAY) {
        ///         // ...
        ///     } else if (sort == Type.METHOD) {
        ///         // ...
        ///     } else {
        ///         // throw an exception
        ///     }
        /// } else if (cst instanceof Handle) {
        ///     // ...
        /// } else if (cst instanceof ConstantDynamic) {
        ///     // ...
        /// } else {
        ///     // throw an exception
        /// }
        /// ```
        ///
        /// - value the constant to be loaded on the stack. This parameter must be a non null {@link
        ///     Integer}, a {@link Float}, a {@link Long}, a {@link Double}, a {@link String}, a {@link
        ///     Type} of OBJECT or ARRAY sort for {@code .class} constants, for classes whose version is
        ///     49, a {@link Type} of METHOD sort for MethodType, a {@link Handle} for MethodHandle
        ///     constants, for classes whose version is 51 or a {@link ConstantDynamic} for a constant
        ///     dynamic for classes whose version is 55.
        pub fn visit_ldc_insn(&self, value: &LdcValue) -> Option<()>;

        /// Visits an IINC instruction.
        ///
        /// - var_index index of the local variable to be incremented.
        /// - increment amount to increment the local variable by.
        pub fn visit_iinc_insn(&self, var_index: u32, increment: u32) -> Option<()>;

        /// Visits a TABLESWITCH instruction.
        ///
        /// - min the minimum key value.
        /// - max the maximum key value.
        /// - dflt beginning of the default handler block.
        /// - labels beginnings of the handler blocks. {@code labels[i]} is the beginning of the
        ///     handler block for the {@code min + i} key.
        pub fn visit_table_switch_insn(&self, min: i32, max: i32, dflt: &Label, labels: &[&Label]) -> Option<()>;

        /// Visits a LOOKUPSWITCH instruction.
        ///
        /// - dflt beginning of the default handler block.
        /// - keys the values of the keys.
        /// - labels beginnings of the handler blocks. {@code labels[i]} is the beginning of the
        ///     handler block for the {@code keys[i]} key.
        pub fn visit_lookup_switch_insn(&self, dflt: &Label, keys: &[i32], labels: &[&Label]) -> Option<()>;

        /// Visits a MULTIANEWARRAY instruction.
        ///
        /// - descriptor an array type descriptor (see {@link Type}).
        /// - num_dimensions the number of dimensions of the array to allocate.
        pub fn visit_multi_anew_array_insn(&self, descriptor: &str, num_dimensions: i32) -> Option<()>;

        /// Visits an annotation on an instruction. This method must be called just <i>after</i> the
        /// annotated instruction. It can be called several times for the same instruction.
        ///
        /// - type_ref a reference to the annotated type. The sort of this type reference must be
        ///     {@link TypeReference#INSTANCEOF}, {@link TypeReference#NEW}, {@link
        ///     TypeReference#CONSTRUCTOR_REFERENCE}, {@link TypeReference#METHOD_REFERENCE}, {@link
        ///     TypeReference#CAST}, {@link TypeReference#CONSTRUCTOR_INVOCATION_TYPE_ARGUMENT}, {@link
        ///     TypeReference#METHOD_INVOCATION_TYPE_ARGUMENT}, {@link
        ///     TypeReference#CONSTRUCTOR_REFERENCE_TYPE_ARGUMENT}, or {@link
        ///     TypeReference#METHOD_REFERENCE_TYPE_ARGUMENT}. See {@link TypeReference}.
        /// - type_path the path to the annotated type argument, wildcard bound, array element type, or
        ///     static inner type within 'typeRef'. May be {@literal null} if the annotation targets
        ///     'typeRef' as a whole.
        /// - descriptor the class descriptor of the annotation class.
        /// - visible {@literal true} if the annotation is visible at runtime.
        ///
        /// returns a visitor to visit the annotation values, or {@literal null} if this visitor is not
        ///     interested in visiting this annotation.
        pub fn visit_insn_annotation(&self, type_ref: i32, type_path: &TypePath,
            descriptor: &str, visible: bool) -> Option<&AnnotationVisitor>;

        // -----------------------------------------------------------------------------------------------
        // Exceptions table entries, debug information, max stack and max locals
        // -----------------------------------------------------------------------------------------------

        ///
        /// Visits a try catch block.
        ///
        /// - start the beginning of the exception handler's scope (inclusive).
        /// - end the end of the exception handler's scope (exclusive).
        /// - handler the beginning of the exception handler's code.
        /// - exception_type the internal name of the type of exceptions handled by the handler (see {@link
        ///     Type#getInternalName()}), or {@literal null} to catch any exceptions (for "finally"
        ///     blocks).
        ///
        /// @throws IllegalArgumentException if one of the labels has already been visited by this visitor
        ///     (by the {@link #visitLabel} method).
        ///
        pub fn visit_try_catch_block(&self, start: &Label, end: &Label, 
            handler: &Label, exception_type: &str) -> Option<()>;

        /// Visits an annotation on an exception handler type. This method must be called <i>after</i> the
        /// {@link #visitTryCatchBlock} for the annotated exception handler. It can be called several times
        /// for the same exception handler.
        ///
        /// - type_ref a reference to the annotated type. The sort of this type reference must be
        ///     {@link TypeReference#EXCEPTION_PARAMETER}. See {@link TypeReference}.
        /// - type_path the path to the annotated type argument, wildcard bound, array element type, or
        ///     static inner type within 'typeRef'. May be {@literal null} if the annotation targets
        ///     'typeRef' as a whole.
        /// - descriptor the class descriptor of the annotation class.
        /// - visible {@literal true} if the annotation is visible at runtime.
        ///
        /// returns a visitor to visit the annotation values, or {@literal null} if this visitor is not
        ///     interested in visiting this annotation.
        pub fn visit_try_catch_annotation(&self, type_ref: i32, type_path: &TypePath, 
            descriptor: &str, visible: bool) -> Option<&AnnotationVisitor>;

        /// Visits a local variable declaration.
        ///
        /// - name the name of a local variable.
        /// - descriptor the type descriptor of this local variable.
        /// - signature the type signature of this local variable. May be {@literal null} if the local
        ///     variable type does not use generic types.
        /// - start the first instruction corresponding to the scope of this local variable
        ///     (inclusive).
        /// - end the last instruction corresponding to the scope of this local variable (exclusive).
        /// - index the local variable's index.
        ///
        /// @throws IllegalArgumentException if one of the labels has not already been visited by this
        ///     visitor (by the {@link #visitLabel} method).
        pub fn visit_local_variable(&self,
            name: &str,descriptor: &str,signature: &str,
            start: &Label,end: &Label,index: i32) -> Option<()>;

        ///
        /// Visits an annotation on a local variable type.
        ///
        /// - type_ref a reference to the annotated type. The sort of this type reference must be
        ///     {@link TypeReference#LOCAL_VARIABLE} or {@link TypeReference#RESOURCE_VARIABLE}. See {@link
        ///     TypeReference}.
        /// - type_path the path to the annotated type argument, wildcard bound, array element type, or
        ///     static inner type within 'typeRef'. May be {@literal null} if the annotation targets
        ///     'typeRef' as a whole.
        /// - start the fist instructions corresponding to the continuous ranges that make the scope
        ///     of this local variable (inclusive).
        /// - end the last instructions corresponding to the continuous ranges that make the scope of
        ///     this local variable (exclusive). This array must have the same size as the 'start' array.
        /// - index the local variable's index in each range. This array must have the same size as
        ///     the 'start' array.
        /// - descriptor the class descriptor of the annotation class.
        /// - visible {@literal true} if the annotation is visible at runtime.
        ///
        /// returns a visitor to visit the annotation values, or {@literal null} if this visitor is not
        ///     interested in visiting this annotation.
        pub fn visit_local_variable_annotation(&self,
            type_ref: i32,
            type_path: &TypePath,
            start: &[&Label],
            end: &[&Label],
            index: &[i32],
            descriptor: &str,
            visible: bool) -> Option<&AnnotationVisitor>;

        /// Visits a line number declaration.
        ///
        /// - line a line number. This number refers to the source file from which the class was
        ///     compiled.
        /// - start the first instruction corresponding to this line number.
        ///
        /// @throws IllegalArgumentException if {@code start} has not already been visited by this visitor
        ///     (by the {@link #visitLabel} method).
        pub fn visit_line_number(&self, line: i32, start: &Label) -> Option<()>;

        /// Visits the maximum stack size and the maximum number of local variables of the method.
        ///
        /// - maxStack maximum stack size of the method.
        /// - maxLocals maximum number of local variables for the method.
        pub fn visit_maxs(&self, max_stack: i32, max_locals: i32) -> Option<()>;

        /// Visits the end of the method. This method, which is the last one to be called, is used to
        /// inform the visitor that all the annotations and attributes of the method have been visited.
        pub fn visit_end(&self) -> Option<()>;
    }
}
