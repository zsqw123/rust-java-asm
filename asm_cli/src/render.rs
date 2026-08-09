use java_asm::node::element::ClassNode;
use java_asm::smali::ToSmali;
use java_asm::{ConstContainer, JavaClassAccessFlags, JavaFieldAccessFlags, JavaMethodAccessFlags};

pub(crate) fn render_jvm_class(node: &ClassNode) -> String {
    let mut output = String::new();
    output.push_str(".class");
    append_java_flags(&mut output, node.access, JavaClassAccessFlags::const_name);
    output.push(' ');
    output.push_str(&node.name);
    output.push('\n');
    if let Some(super_name) = &node.super_name {
        output.push_str(".super ");
        output.push_str(super_name);
        output.push('\n');
    }
    for interface in &node.interfaces {
        output.push_str(".implements ");
        output.push_str(interface);
        output.push('\n');
    }
    if let Some(source_file) = &node.source_file {
        output.push_str(".source ");
        output.push_str(source_file);
        output.push('\n');
    }
    for field in &node.fields {
        output.push_str(".field");
        append_java_flags(&mut output, field.access, JavaFieldAccessFlags::const_name);
        output.push(' ');
        output.push_str(&field.name);
        output.push(' ');
        output.push_str(&field.desc);
        if let Some(value) = &field.value {
            output.push_str(" = ");
            output.push_str(&format!("{value:?}"));
        }
        output.push('\n');
    }
    for method in &node.methods {
        output.push_str(".method");
        append_java_flags(
            &mut output,
            method.access,
            JavaMethodAccessFlags::const_name,
        );
        output.push(' ');
        output.push_str(&method.name);
        output.push(' ');
        output.push_str(&method.desc);
        output.push('\n');
        if let Some(code_body) = &method.code_body {
            output.push_str("  .registers ");
            output.push_str(&code_body.max_locals.to_string());
            output.push('\n');
            for instruction in &code_body.instructions {
                for line in instruction.to_smali().render(0).lines() {
                    output.push_str("  ");
                    output.push_str(line);
                    output.push('\n');
                }
            }
        }
        output.push_str(".end method\n");
    }
    output.push_str(".end class\n");
    output
}

fn append_java_flags(output: &mut String, flags: u16, const_name: fn(u16) -> Option<&'static str>) {
    for bit in 0..u16::BITS {
        let flag = 1u16 << bit;
        if flags & flag == 0 {
            continue;
        }
        let Some(name) = const_name(flag) else {
            continue;
        };
        output.push(' ');
        output.push_str(name.strip_prefix("acc_").unwrap_or(name));
    }
}
