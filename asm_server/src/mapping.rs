use java_asm::smali::{MappedName, SmaliNode, SmaliToken};
use java_asm::{DescriptorRef, StrRef};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappingError {
    pub line: usize,
    pub message: String,
}

impl MappingError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl Display for MappingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            formatter.write_str(&self.message)
        } else {
            write!(formatter, "line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for MappingError {}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct MemberKey {
    raw_name: StrRef,
    raw_descriptor: DescriptorRef,
}

#[derive(Clone, Debug)]
struct FieldMapping {
    display_name: StrRef,
}

#[derive(Clone, Debug)]
struct MethodMapping {
    display_owner: DescriptorRef,
    display_name: StrRef,
    raw_line_range: Option<(u32, u32)>,
    display_line_range: Option<(u32, u32)>,
}

#[derive(Clone, Debug)]
struct ClassMapping {
    display_descriptor: DescriptorRef,
    fields: HashMap<MemberKey, FieldMapping>,
    methods: HashMap<MemberKey, Vec<MethodMapping>>,
}

#[derive(Clone, Debug, Default)]
pub struct Mapping {
    classes_by_raw: HashMap<DescriptorRef, ClassMapping>,
    raw_by_display: HashMap<DescriptorRef, DescriptorRef>,
}

#[derive(Clone, Debug)]
struct ParsedClass {
    display_name: String,
    raw_name: String,
    fields: Vec<ParsedField>,
    methods: Vec<ParsedMethod>,
}

#[derive(Clone, Debug)]
struct ParsedField {
    display_type: String,
    display_name: String,
    raw_name: String,
    raw_signature: Option<String>,
}

#[derive(Clone, Debug)]
struct ParsedMethod {
    display_owner: Option<String>,
    display_return_type: String,
    display_name: String,
    display_arguments: Vec<String>,
    raw_name: String,
    raw_line_range: Option<(u32, u32)>,
    display_line_range: Option<(u32, u32)>,
    raw_signature: Option<String>,
}

#[derive(Copy, Clone, Debug)]
enum LastMember {
    Field(usize),
    Method(usize),
}

impl Mapping {
    pub fn parse(bytes: &[u8]) -> Result<Self, MappingError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|error| MappingError::new(0, format!("mapping is not UTF-8: {error}")))?;
        Self::parse_str(text.trim_start_matches('\u{feff}'))
    }

    pub fn parse_str(text: &str) -> Result<Self, MappingError> {
        let mut classes = Vec::<ParsedClass>::new();
        let mut current_class = None;
        let mut last_member = None;

        for (line_index, raw_line) in text.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line.trim_end_matches('\r');
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with('#') {
                apply_json_comment(trimmed, &mut classes, current_class, last_member);
                continue;
            }

            if !line.starts_with(char::is_whitespace) {
                let (display_name, raw_name) = parse_class_line(trimmed)
                    .ok_or_else(|| MappingError::new(line_number, "invalid class mapping"))?;
                classes.push(ParsedClass {
                    display_name,
                    raw_name,
                    fields: Vec::new(),
                    methods: Vec::new(),
                });
                current_class = Some(classes.len() - 1);
                last_member = None;
                continue;
            }

            let Some(class_index) = current_class else {
                return Err(MappingError::new(
                    line_number,
                    "member mapping has no class",
                ));
            };
            let (left, raw_name) = split_arrow(trimmed)
                .ok_or_else(|| MappingError::new(line_number, "invalid member mapping"))?;
            if left.contains('(') {
                let method = parse_method(left, raw_name)
                    .ok_or_else(|| MappingError::new(line_number, "invalid method mapping"))?;
                let class = &mut classes[class_index];
                class.methods.push(method);
                last_member = Some(LastMember::Method(class.methods.len() - 1));
            } else {
                let field = parse_field(left, raw_name)
                    .ok_or_else(|| MappingError::new(line_number, "invalid field mapping"))?;
                let class = &mut classes[class_index];
                class.fields.push(field);
                last_member = Some(LastMember::Field(class.fields.len() - 1));
            }
        }

        if classes.is_empty() {
            return Err(MappingError::new(0, "mapping contains no classes"));
        }
        Ok(Self::from_parsed(classes))
    }

    fn from_parsed(classes: Vec<ParsedClass>) -> Self {
        let mut mapping = Self::default();
        for class in &classes {
            let display_descriptor = java_class_to_descriptor(&class.display_name);
            let raw_descriptor = java_class_to_descriptor(&class.raw_name);
            mapping
                .raw_by_display
                .insert(Arc::clone(&display_descriptor), Arc::clone(&raw_descriptor));
            mapping.classes_by_raw.insert(
                raw_descriptor,
                ClassMapping {
                    display_descriptor,
                    fields: HashMap::new(),
                    methods: HashMap::new(),
                },
            );
        }

        for parsed in classes {
            let raw_owner = java_class_to_descriptor(&parsed.raw_name);
            let display_owner = java_class_to_descriptor(&parsed.display_name);
            let fields: Vec<_> = parsed
                .fields
                .into_iter()
                .map(|field| {
                    let raw_descriptor = field
                        .raw_signature
                        .map(|signature| normalize_raw_signature(&signature, false).into())
                        .unwrap_or_else(|| mapping.raw_type_descriptor(&field.display_type));
                    (
                        MemberKey {
                            raw_name: field.raw_name.into(),
                            raw_descriptor,
                        },
                        FieldMapping {
                            display_name: field.display_name.into(),
                        },
                    )
                })
                .collect();
            let methods: Vec<_> = parsed
                .methods
                .into_iter()
                .map(|method| {
                    let raw_descriptor = method
                        .raw_signature
                        .map(|signature| normalize_raw_signature(&signature, true).into())
                        .unwrap_or_else(|| {
                            mapping.raw_method_descriptor(
                                &method.display_arguments,
                                &method.display_return_type,
                            )
                        });
                    let display_owner = method
                        .display_owner
                        .as_deref()
                        .map(java_class_to_descriptor)
                        .unwrap_or_else(|| Arc::clone(&display_owner));
                    (
                        MemberKey {
                            raw_name: method.raw_name.into(),
                            raw_descriptor,
                        },
                        MethodMapping {
                            display_owner,
                            display_name: method.display_name.into(),
                            raw_line_range: method.raw_line_range,
                            display_line_range: method.display_line_range,
                        },
                    )
                })
                .collect();

            let Some(class) = mapping.classes_by_raw.get_mut(&raw_owner) else {
                continue;
            };
            class.fields.extend(fields);
            for (key, method) in methods {
                class.methods.entry(key).or_default().push(method);
            }
        }
        mapping
    }

    pub fn display_class_name(&self, raw_name: &str) -> Option<DescriptorRef> {
        self.classes_by_raw
            .get(raw_name)
            .map(|class| Arc::clone(&class.display_descriptor))
    }

    pub fn raw_class_name(&self, display_name: &str) -> Option<DescriptorRef> {
        self.raw_by_display.get(display_name).map(Arc::clone)
    }

    pub fn mapped_class_name(&self, raw_name: StrRef) -> MappedName {
        let display_name = self
            .display_class_name(&raw_name)
            .unwrap_or_else(|| Arc::clone(&raw_name));
        MappedName {
            raw_name,
            display_name,
        }
    }

    pub fn apply_to_smali(&self, raw_class_name: &str, smali: &mut SmaliNode) {
        self.apply_node(raw_class_name, None, smali);
    }

    fn apply_node(
        &self,
        raw_class_name: &str,
        method_context: Option<MemberKey>,
        node: &mut SmaliNode,
    ) {
        let signature = node_signature(&node.content, raw_class_name);
        let mut child_method_context = method_context;
        if let Some((owner, member_index, descriptor_index, key)) = signature {
            let is_method = key.raw_descriptor.starts_with('(');
            let display_name = if is_method {
                self.display_method_name(&owner, &key)
            } else {
                self.display_field_name(&owner, &key)
            };
            if let Some(display_name) = display_name
                && let Some(SmaliToken::MemberName(name)) = node.content.get_mut(member_index)
            {
                name.display_name = display_name;
            }
            if is_method && owner == raw_class_name {
                child_method_context = Some(key);
            }
            // Keep the raw descriptor until member lookup has completed.
            self.map_descriptor_token(&mut node.content[descriptor_index]);
        }

        for token in &mut node.content {
            match token {
                SmaliToken::Descriptor(_) => self.map_descriptor_token(token),
                SmaliToken::SourceLine {
                    raw_line,
                    display_line,
                } => {
                    if let Some(key) = child_method_context.as_ref()
                        && let Some(line) = self.display_line(raw_class_name, key, *raw_line)
                    {
                        *display_line = line;
                    }
                }
                _ => {}
            }
        }
        for child in &mut node.children {
            self.apply_node(raw_class_name, child_method_context.clone(), child);
        }
    }

    fn map_descriptor_token(&self, token: &mut SmaliToken) {
        let SmaliToken::Descriptor(name) = token else {
            return;
        };
        if let Some(display) = self.display_descriptor(&name.raw_name) {
            name.display_name = display;
        }
    }

    fn display_field_name(&self, raw_owner: &str, key: &MemberKey) -> Option<StrRef> {
        self.classes_by_raw
            .get(raw_owner)?
            .fields
            .get(key)
            .map(|field| Arc::clone(&field.display_name))
    }

    fn display_method_name(&self, raw_owner: &str, key: &MemberKey) -> Option<StrRef> {
        let class = self.classes_by_raw.get(raw_owner)?;
        class
            .methods
            .get(key)?
            .iter()
            .rev()
            .find(|method| method.display_owner == class.display_descriptor)
            .or_else(|| class.methods.get(key)?.last())
            .map(|method| Arc::clone(&method.display_name))
    }

    fn display_line(&self, raw_owner: &str, key: &MemberKey, raw_line: u32) -> Option<u32> {
        let class = self.classes_by_raw.get(raw_owner)?;
        let method = class.methods.get(key)?.iter().rev().find(|method| {
            method.display_owner == class.display_descriptor
                && contains_line(method.raw_line_range, raw_line)
        })?;
        translate_line(method.raw_line_range?, method.display_line_range?, raw_line)
    }

    fn display_descriptor(&self, raw_descriptor: &str) -> Option<DescriptorRef> {
        let mut matched = false;
        let descriptor = map_descriptor(raw_descriptor, |raw_class| {
            let display = self.display_class_name(raw_class);
            matched |= display.is_some();
            display
        });
        matched.then(|| descriptor.into())
    }

    fn raw_type_descriptor(&self, display_type: &str) -> DescriptorRef {
        let display_descriptor = java_type_to_descriptor(display_type);
        map_descriptor(&display_descriptor, |display_class| {
            self.raw_class_name(display_class)
        })
        .into()
    }

    fn raw_method_descriptor(
        &self,
        display_arguments: &[String],
        display_return_type: &str,
    ) -> DescriptorRef {
        let arguments = display_arguments
            .iter()
            .map(|argument| self.raw_type_descriptor(argument))
            .collect::<Vec<_>>()
            .join("");
        format!(
            "({arguments}){}",
            self.raw_type_descriptor(display_return_type)
        )
        .into()
    }
}

fn node_signature(
    tokens: &[SmaliToken],
    default_owner: &str,
) -> Option<(String, usize, usize, MemberKey)> {
    let member_index = tokens
        .iter()
        .position(|token| matches!(token, SmaliToken::MemberName(_)))?;
    let descriptor_index = tokens
        .iter()
        .enumerate()
        .skip(member_index + 1)
        .find_map(|(index, token)| matches!(token, SmaliToken::Descriptor(_)).then_some(index))?;
    let raw_name = match &tokens[member_index] {
        SmaliToken::MemberName(name) => Arc::clone(&name.raw_name),
        _ => return None,
    };
    let raw_descriptor = match &tokens[descriptor_index] {
        SmaliToken::Descriptor(name) => Arc::clone(&name.raw_name),
        _ => return None,
    };
    let owner = tokens[..member_index]
        .iter()
        .rev()
        .find_map(|token| match token {
            SmaliToken::Descriptor(name) if !name.raw_name.starts_with('(') => {
                Some(name.raw_name.to_string())
            }
            _ => None,
        })
        .unwrap_or_else(|| default_owner.to_owned());
    Some((
        owner,
        member_index,
        descriptor_index,
        MemberKey {
            raw_name,
            raw_descriptor,
        },
    ))
}

fn parse_class_line(line: &str) -> Option<(String, String)> {
    let line = line.strip_suffix(':')?;
    let (display, raw) = split_arrow(line)?;
    Some((display.to_owned(), raw.to_owned()))
}

fn split_arrow(line: &str) -> Option<(&str, &str)> {
    let (left, right) = line.split_once("->")?;
    let left = left.trim();
    let right = right.trim();
    (!left.is_empty() && !right.is_empty()).then_some((left, right))
}

fn parse_field(left: &str, raw_name: &str) -> Option<ParsedField> {
    let split = left.rfind(char::is_whitespace)?;
    let display_type = left[..split].trim();
    let display_name = left[split..].trim();
    if display_type.is_empty() || display_name.is_empty() {
        return None;
    }
    Some(ParsedField {
        display_type: display_type.to_owned(),
        display_name: display_name.to_owned(),
        raw_name: raw_name.to_owned(),
        raw_signature: None,
    })
}

fn parse_method(left: &str, raw_name: &str) -> Option<ParsedMethod> {
    let (raw_line_range, left) = take_leading_range(left);
    let type_split = left.find(char::is_whitespace)?;
    let display_return_type = left[..type_split].trim();
    let signature = left[type_split..].trim();
    let open = signature.find('(')?;
    let close = signature.rfind(')')?;
    if close < open {
        return None;
    }
    let qualified_name = &signature[..open];
    let (display_owner, display_name) = qualified_name
        .rsplit_once('.')
        .map(|(owner, name)| (Some(owner.to_owned()), name.to_owned()))
        .unwrap_or_else(|| (None, qualified_name.to_owned()));
    let display_arguments = signature[open + 1..close]
        .split(',')
        .filter(|argument| !argument.trim().is_empty())
        .map(|argument| argument.trim().to_owned())
        .collect();
    let display_line_range = parse_trailing_range(signature[close + 1..].trim()).or(raw_line_range);
    Some(ParsedMethod {
        display_owner,
        display_return_type: display_return_type.to_owned(),
        display_name,
        display_arguments,
        raw_name: raw_name.to_owned(),
        raw_line_range,
        display_line_range,
        raw_signature: None,
    })
}

fn take_leading_range(value: &str) -> (Option<(u32, u32)>, &str) {
    let Some(first_colon) = value.find(':') else {
        return (None, value);
    };
    let Some(second_relative) = value[first_colon + 1..].find(':') else {
        return (None, value);
    };
    let second_colon = first_colon + 1 + second_relative;
    let Ok(start) = value[..first_colon].parse() else {
        return (None, value);
    };
    let Ok(end) = value[first_colon + 1..second_colon].parse() else {
        return (None, value);
    };
    (Some((start, end)), &value[second_colon + 1..])
}

fn parse_trailing_range(value: &str) -> Option<(u32, u32)> {
    let value = value.strip_prefix(':')?;
    let mut parts = value.split(':');
    let start = parts.next()?.parse().ok()?;
    let end = parts
        .next()
        .and_then(|part| part.parse().ok())
        .unwrap_or(start);
    Some((start, end))
}

fn apply_json_comment(
    line: &str,
    classes: &mut [ParsedClass],
    current_class: Option<usize>,
    last_member: Option<LastMember>,
) {
    let Some(json_start) = line.find('{') else {
        return;
    };
    let Ok(json) = serde_json::from_str::<Value>(&line[json_start..]) else {
        return;
    };
    let Some(id) = json.get("id").and_then(Value::as_str) else {
        return;
    };
    if id != "com.android.tools.r8.residualsignature" {
        return;
    }
    let Some(signature) = json.get("signature").and_then(Value::as_str) else {
        return;
    };
    let Some(class) = current_class.and_then(|index| classes.get_mut(index)) else {
        return;
    };
    match last_member {
        Some(LastMember::Field(index)) => {
            if let Some(field) = class.fields.get_mut(index) {
                field.raw_signature = Some(signature.to_owned());
            }
        }
        Some(LastMember::Method(index)) => {
            if let Some(method) = class.methods.get_mut(index) {
                method.raw_signature = Some(signature.to_owned());
            }
        }
        None => {}
    }
}

fn java_class_to_descriptor(class_name: &str) -> DescriptorRef {
    format!("L{};", class_name.replace('.', "/")).into()
}

fn java_type_to_descriptor(type_name: &str) -> String {
    let mut base = type_name.trim();
    let mut dimensions = 0usize;
    while let Some(stripped) = base.strip_suffix("[]") {
        dimensions += 1;
        base = stripped;
    }
    let descriptor = match base {
        "void" => "V".to_owned(),
        "boolean" => "Z".to_owned(),
        "byte" => "B".to_owned(),
        "char" => "C".to_owned(),
        "short" => "S".to_owned(),
        "int" => "I".to_owned(),
        "long" => "J".to_owned(),
        "float" => "F".to_owned(),
        "double" => "D".to_owned(),
        class_name if class_name.starts_with('L') && class_name.ends_with(';') => {
            class_name.to_owned()
        }
        class_name => format!("L{};", class_name.replace('.', "/")),
    };
    format!("{}{descriptor}", "[".repeat(dimensions))
}

fn normalize_raw_signature(signature: &str, is_method: bool) -> String {
    if !is_method {
        return normalize_raw_type(signature);
    }
    let Some(close) = signature.rfind(')') else {
        return signature.to_owned();
    };
    let arguments = &signature[..=close];
    let returned = &signature[close + 1..];
    format!("{arguments}{}", normalize_raw_type(returned))
}

fn normalize_raw_type(raw_type: &str) -> String {
    if raw_type.starts_with('[')
        || raw_type.starts_with('L') && raw_type.ends_with(';')
        || matches!(
            raw_type,
            "V" | "Z" | "B" | "C" | "S" | "I" | "J" | "F" | "D"
        )
    {
        raw_type.to_owned()
    } else {
        format!("L{};", raw_type.replace('.', "/"))
    }
}

fn map_descriptor(
    descriptor: &str,
    mut map_class: impl FnMut(&str) -> Option<DescriptorRef>,
) -> String {
    let mut mapped = String::with_capacity(descriptor.len());
    let mut cursor = 0usize;
    while let Some(relative_start) = descriptor[cursor..].find('L') {
        let start = cursor + relative_start;
        mapped.push_str(&descriptor[cursor..start]);
        let Some(relative_end) = descriptor[start..].find(';') else {
            mapped.push_str(&descriptor[start..]);
            return mapped;
        };
        let end = start + relative_end + 1;
        let class = &descriptor[start..end];
        if let Some(replacement) = map_class(class) {
            mapped.push_str(&replacement);
        } else {
            mapped.push_str(class);
        }
        cursor = end;
    }
    mapped.push_str(&descriptor[cursor..]);
    mapped
}

fn contains_line(range: Option<(u32, u32)>, line: u32) -> bool {
    range.is_some_and(|(start, end)| start <= line && line <= end)
}

fn translate_line(raw: (u32, u32), display: (u32, u32), line: u32) -> Option<u32> {
    if line < raw.0 || line > raw.1 {
        return None;
    }
    let offset = line - raw.0;
    Some(display.0 + offset.min(display.1.saturating_sub(display.0)))
}

#[cfg(test)]
mod tests {
    use super::Mapping;
    use java_asm::smali::{SmaliToken, stb};

    const SAMPLE: &str = r#"
com.example.User -> a.b:
    java.lang.String name -> a
    1:3:java.lang.String getName(com.example.Input):40:42 -> b
com.example.Input -> c:
"#;

    #[test]
    fn parses_classes_members_and_lines() {
        let mapping = Mapping::parse_str(SAMPLE).unwrap();
        assert_eq!(
            mapping.display_class_name("La/b;").as_deref(),
            Some("Lcom/example/User;")
        );
        assert_eq!(
            mapping.raw_class_name("Lcom/example/Input;").as_deref(),
            Some("Lc;")
        );

        let mut smali = stb()
            .mn("b".into())
            .d("(Lc;)Ljava/lang/String;".into())
            .s_with_children(vec![stb().raw(".source-line").source_line(2).s()]);
        mapping.apply_to_smali("La/b;", &mut smali);
        let SmaliToken::MemberName(name) = &smali.content[0] else {
            panic!()
        };
        assert_eq!(name.display_name.as_ref(), "getName");
        let SmaliToken::Descriptor(descriptor) = &smali.content[1] else {
            panic!()
        };
        assert_eq!(
            descriptor.display_name.as_ref(),
            "(Lcom/example/Input;)Ljava/lang/String;"
        );
        let SmaliToken::SourceLine { display_line, .. } = &smali.children[0].content[1] else {
            panic!()
        };
        assert_eq!(*display_line, 41);
    }

    #[test]
    fn reads_r8_raw_signatures() {
        let mapping = Mapping::parse_str(
            r#"
com.example.User -> a:
    boolean enabled -> a
    # {"id":"com.android.tools.r8.residualsignature","signature":"I"}
    void setEnabled(boolean) -> a
    # {"id":"com.android.tools.r8.residualsignature","signature":"(I)V"}
"#,
        )
        .unwrap();
        assert!(
            mapping
                .display_field_name(
                    "La;",
                    &super::MemberKey {
                        raw_name: "a".into(),
                        raw_descriptor: "I".into(),
                    }
                )
                .is_some()
        );
        assert!(
            mapping
                .display_method_name(
                    "La;",
                    &super::MemberKey {
                        raw_name: "a".into(),
                        raw_descriptor: "(I)V".into(),
                    }
                )
                .is_some()
        );
        assert_eq!(super::normalize_raw_signature("(Z)a", true), "(Z)La;");
    }
}
