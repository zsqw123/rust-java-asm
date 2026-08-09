use java_asm::AsmErr;
use java_asm::dex::{ClassDef, DexFile, DexFileAccessor};
use java_asm::node::element::{ClassNode, FieldNode, MethodNode};
use java_asm::smali::ToSmali;
use serde_json::{Value, json};
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[cfg(not(target_family = "wasm"))]
use zip::ZipArchive;

pub const DEFAULT_OUTPUT_DIR: &str = "asm_cli_output";

#[derive(Debug)]
pub enum CliError {
    Usage(String),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Zip {
        path: PathBuf,
        message: String,
    },
    Parse {
        source: String,
        message: String,
    },
    NotFound(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Zip { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Parse { source, message } => write!(f, "{source}: {message}"),
            Self::NotFound(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for CliError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandKind {
    Decompile,
    FindClass,
    FindMethod,
    FindField,
}

#[derive(Debug)]
struct CommandLine {
    kind: CommandKind,
    input: PathBuf,
    positionals: Vec<String>,
    output: PathBuf,
    class_filter: Option<String>,
    source_filter: Option<String>,
    limit: usize,
}

enum ClassPayload {
    Dex {
        accessor: Arc<DexFileAccessor>,
        class_def: ClassDef,
    },
    Jvm {
        node: Arc<ClassNode>,
    },
}

struct ClassEntry {
    class_name: String,
    descriptor: String,
    source: String,
    payload: ClassPayload,
}

#[derive(Default)]
struct InputIndex {
    classes: Vec<ClassEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemberKind {
    Method,
    Field,
}

#[derive(Debug)]
struct MemberEntry {
    kind: MemberKind,
    owner: String,
    owner_descriptor: String,
    name: String,
    descriptor: String,
    source: String,
    dex: Option<String>,
}

#[derive(Debug, Default)]
struct MemberQuery {
    owner: Option<String>,
    name: Option<String>,
    descriptor: Option<String>,
}

pub fn execute<I, S>(args: I) -> Result<Value, CliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect();
    let command_line = parse_command_line(&args)?;
    let index = InputIndex::load(&command_line.input, command_line.source_filter.as_deref())?;

    match command_line.kind {
        CommandKind::Decompile => execute_decompile(&command_line, &index),
        CommandKind::FindClass => execute_find_class(&command_line, &index),
        CommandKind::FindMethod | CommandKind::FindField => {
            execute_find_member(&command_line, &index)
        }
    }
}

pub fn usage() -> &'static str {
    "asm_cli - inspect and render Java class files, DEX, JAR and APK inputs

USAGE:
  asm_cli decompile <input> [--output <dir>] [--class <name>] [--source <entry>]
  asm_cli find-class <input> <query> [--limit <n>]
  asm_cli find-method <input> <class> <name> [descriptor] [--limit <n>]
  asm_cli find-field  <input> <class> <name> [descriptor] [--limit <n>]

Aliases: findClass, findMethod, findField. If the first argument is an input
file, decompile is assumed. Class names accept dotted, slash-separated and
descriptor forms (for example com.example.Main, com/example/Main or
Lcom/example/Main;). Member references also accept Lpkg/Class;->method:(I)V.

The default decompile directory is ./asm_cli_output. Search results include
the source entry, such as classes14.dex, so it can be passed to --source for a
faster targeted decompile."
}

fn parse_command_line(args: &[String]) -> Result<CommandLine, CliError> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        return Err(CliError::Usage(usage().to_owned()));
    }

    let (kind, start) = match args[0].as_str() {
        "decompile" | "dump" => (CommandKind::Decompile, 1),
        "find-class" | "findClass" => (CommandKind::FindClass, 1),
        "find-method" | "findMethod" => (CommandKind::FindMethod, 1),
        "find-field" | "findField" => (CommandKind::FindField, 1),
        "--version" | "-V" => {
            return Err(CliError::Usage(env!("CARGO_PKG_VERSION").to_owned()));
        }
        _ => (CommandKind::Decompile, 0),
    };
    let mut positionals = Vec::new();
    let mut output = PathBuf::from(DEFAULT_OUTPUT_DIR);
    let mut class_filter = None;
    let mut source_filter = None;
    let mut limit = 30usize;
    let mut index = start;
    while index < args.len() {
        let arg = &args[index];
        match arg.as_str() {
            "--help" | "-h" => return Err(CliError::Usage(usage().to_owned())),
            "--output" | "-o" => {
                output = PathBuf::from(next_value(args, &mut index, arg)?);
            }
            "--class" | "-c" => {
                class_filter = Some(next_value(args, &mut index, arg)?);
            }
            "--source" | "-s" => {
                source_filter = Some(next_value(args, &mut index, arg)?);
            }
            "--limit" | "-n" => {
                let raw = next_value(args, &mut index, arg)?;
                limit = raw
                    .parse()
                    .map_err(|_| CliError::Usage(format!("invalid --limit value: {raw}")))?;
            }
            _ if arg.starts_with('-') => {
                return Err(CliError::Usage(format!(
                    "unknown option: {arg}\n\n{}",
                    usage()
                )));
            }
            _ => positionals.push(arg.clone()),
        }
        index += 1;
    }

    let Some(input) = positionals.first() else {
        return Err(CliError::Usage(usage().to_owned()));
    };
    if matches!(kind, CommandKind::Decompile) && positionals.len() > 1 {
        return Err(CliError::Usage(
            "decompile accepts one input; use --class and --source for filters".to_owned(),
        ));
    }
    if matches!(kind, CommandKind::FindClass) && positionals.len() != 2 {
        return Err(CliError::Usage(
            "find-class requires <input> <query>".to_owned(),
        ));
    }
    if matches!(kind, CommandKind::FindMethod | CommandKind::FindField) && positionals.len() < 2 {
        return Err(CliError::Usage(
            "find-method/find-field requires an input and a member query".to_owned(),
        ));
    }
    if !matches!(kind, CommandKind::Decompile)
        && (output != PathBuf::from(DEFAULT_OUTPUT_DIR)
            || class_filter.is_some()
            || source_filter.is_some())
    {
        return Err(CliError::Usage(
            "--output, --class and --source are only valid for decompile".to_owned(),
        ));
    }

    Ok(CommandLine {
        kind,
        input: PathBuf::from(input),
        positionals: positionals.into_iter().skip(1).collect(),
        output,
        class_filter,
        source_filter,
        limit,
    })
}

fn next_value(args: &[String], index: &mut usize, option: &str) -> Result<String, CliError> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| CliError::Usage(format!("missing value for {option}\n\n{}", usage())))
}

impl InputIndex {
    fn load(path: &Path, source_filter: Option<&str>) -> Result<Self, CliError> {
        let bytes = fs::read(path).map_err(|source| CliError::Io {
            path: path.to_owned(),
            source,
        })?;
        let source = input_source_name(path);
        if is_zip(&bytes) || is_archive_extension(path) {
            Self::load_zip(path, bytes, source_filter)
        } else {
            let mut index = Self::default();
            index.add_embedded(&source, bytes)?;
            if index.classes.is_empty() {
                return Err(CliError::Parse {
                    source,
                    message: "unsupported input; expected DEX, class or ZIP/JAR/APK".to_owned(),
                });
            }
            Ok(index)
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn load_zip(
        path: &Path,
        bytes: Vec<u8>,
        source_filter: Option<&str>,
    ) -> Result<Self, CliError> {
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|error| CliError::Zip {
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        let mut index = Self::default();
        for entry_index in 0..archive.len() {
            let mut entry = archive
                .by_index(entry_index)
                .map_err(|error| CliError::Zip {
                    path: path.to_owned(),
                    message: error.to_string(),
                })?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_owned();
            if source_filter.is_some_and(|filter| filter != name) {
                continue;
            }
            let mut entry_bytes = Vec::with_capacity(entry.size().min(usize::MAX as u64) as usize);
            entry
                .read_to_end(&mut entry_bytes)
                .map_err(|source| CliError::Io {
                    path: PathBuf::from(&name),
                    source,
                })?;
            if is_dex(&entry_bytes) || is_class(&entry_bytes) {
                index.add_embedded(&name, entry_bytes)?;
            }
        }
        if index.classes.is_empty() {
            let suffix = source_filter
                .map(|filter| format!(" matching --source {filter}"))
                .unwrap_or_default();
            return Err(CliError::Parse {
                source: path.display().to_string(),
                message: format!("archive contains no supported classes or DEX files{suffix}"),
            });
        }
        Ok(index)
    }

    #[cfg(target_family = "wasm")]
    fn load_zip(
        _path: &Path,
        _bytes: Vec<u8>,
        _source_filter: Option<&str>,
    ) -> Result<Self, CliError> {
        Err(CliError::Usage(
            "asm_cli is a native-only executable".to_owned(),
        ))
    }

    fn add_embedded(&mut self, source: &str, bytes: Vec<u8>) -> Result<(), CliError> {
        if is_dex(&bytes) {
            self.add_dex(source, bytes)
        } else if is_class(&bytes) {
            self.add_class(source, bytes)
        } else {
            Ok(())
        }
    }

    fn add_dex(&mut self, source: &str, bytes: Vec<u8>) -> Result<(), CliError> {
        let file =
            DexFile::resolve_from_bytes(&bytes).map_err(|error| parse_error(source, error))?;
        let accessor = Arc::new(DexFileAccessor::new(file, bytes, source.into()));
        let class_defs = accessor.file.class_defs.clone();
        for class_def in class_defs {
            let descriptor = accessor
                .get_type(class_def.class_idx)
                .map_err(|error| parse_error(source, error))?;
            let Some(class_name) = descriptor_to_internal(&descriptor) else {
                return Err(CliError::Parse {
                    source: source.to_owned(),
                    message: format!("invalid class descriptor {descriptor}"),
                });
            };
            self.classes.push(ClassEntry {
                class_name,
                descriptor: descriptor.to_string(),
                source: source.to_owned(),
                payload: ClassPayload::Dex {
                    accessor: Arc::clone(&accessor),
                    class_def,
                },
            });
        }
        Ok(())
    }

    fn add_class(&mut self, source: &str, bytes: Vec<u8>) -> Result<(), CliError> {
        let node = ClassNode::from_bytes(&bytes).map_err(|error| parse_error(source, error))?;
        let class_name = node.name.to_string();
        let descriptor = format!("L{class_name};");
        self.classes.push(ClassEntry {
            class_name,
            descriptor,
            source: source.to_owned(),
            payload: ClassPayload::Jvm {
                node: Arc::new(node),
            },
        });
        Ok(())
    }
}

fn parse_error(source: &str, error: AsmErr) -> CliError {
    CliError::Parse {
        source: source.to_owned(),
        message: format!("{error:?}"),
    }
}

fn is_dex(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && &bytes[..4] == b"dex\n"
}

fn is_class(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[..4] == [0xCA, 0xFE, 0xBA, 0xBE]
}

fn is_zip(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && matches!(&bytes[..4], b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08")
}

fn is_archive_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "apk" | "jar" | "zip"
            )
        })
}

fn input_source_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn descriptor_to_internal(descriptor: &str) -> Option<String> {
    descriptor
        .strip_prefix('L')
        .and_then(|name| name.strip_suffix(';'))
        .map(ToOwned::to_owned)
}

fn normalize_class_name(value: &str) -> String {
    let value = value.trim();
    let value = value.strip_prefix('L').unwrap_or(value);
    let value = value.strip_suffix(';').unwrap_or(value);
    value.replace('.', "/")
}

fn class_matches(entry: &ClassEntry, query: &str) -> Option<u32> {
    let query = normalize_class_name(query);
    if query.is_empty() {
        return Some(0);
    }
    let name = entry.class_name.as_str();
    if name == query {
        return Some(10_000);
    }
    let query_lower = query.to_ascii_lowercase();
    let name_lower = name.to_ascii_lowercase();
    if name_lower == query_lower {
        return Some(9_000);
    }
    if name_lower.ends_with(&format!("/{query_lower}")) {
        return Some(8_000);
    }
    if name_lower.contains(&query_lower) {
        return Some(5_000);
    }
    let mut query_chars = query_lower.chars();
    let mut next = query_chars.next();
    let mut matched = 0u32;
    for character in name_lower.chars() {
        if next == Some(character) {
            matched += 1;
            next = query_chars.next();
        }
    }
    (next.is_none() && matched > 0).then_some(1_000 + matched)
}

fn execute_find_class(command: &CommandLine, index: &InputIndex) -> Result<Value, CliError> {
    let query = command.positionals.first().cloned().unwrap_or_default();
    let mut matches: Vec<(&ClassEntry, u32)> = index
        .classes
        .iter()
        .filter_map(|entry| class_matches(entry, &query).map(|score| (entry, score)))
        .collect();
    matches.sort_by(|(left, left_score), (right, right_score)| {
        right_score
            .cmp(left_score)
            .then_with(|| left.class_name.cmp(&right.class_name))
            .then_with(|| left.source.cmp(&right.source))
    });
    let results: Vec<Value> = matches
        .into_iter()
        .take(command.limit)
        .map(|(entry, _)| class_json(entry))
        .collect();
    Ok(json!({
        "ok": true,
        "operation": "findClass",
        "query": query,
        "count": results.len(),
        "results": results,
    }))
}

fn execute_find_member(command: &CommandLine, index: &InputIndex) -> Result<Value, CliError> {
    let kind = match command.kind {
        CommandKind::FindMethod => MemberKind::Method,
        CommandKind::FindField => MemberKind::Field,
        _ => unreachable!(),
    };
    let query = parse_member_query(&command.positionals)?;
    let mut results = Vec::new();
    for class in &index.classes {
        if let Some(owner) = &query.owner {
            if normalize_class_name(&class.class_name) != *owner {
                continue;
            }
        }
        for member in class.members(kind)? {
            if query
                .name
                .as_deref()
                .is_some_and(|name| member.name != name)
            {
                continue;
            }
            if query
                .descriptor
                .as_deref()
                .is_some_and(|descriptor| member.descriptor != descriptor)
            {
                continue;
            }
            results.push(member_json(&member));
        }
    }
    results.truncate(command.limit);
    Ok(json!({
        "ok": true,
        "operation": if kind == MemberKind::Method { "findMethod" } else { "findField" },
        "query": {
            "class": query.owner,
            "name": query.name,
            "descriptor": query.descriptor,
        },
        "count": results.len(),
        "results": results,
    }))
}

fn parse_member_query(positionals: &[String]) -> Result<MemberQuery, CliError> {
    let Some(first) = positionals.first() else {
        return Err(CliError::Usage("member query cannot be empty".to_owned()));
    };
    if positionals.len() == 1 {
        if let Some((owner, name, descriptor)) = parse_member_reference(first) {
            return Ok(MemberQuery {
                owner: Some(normalize_class_name(owner)),
                name: Some(name.to_owned()),
                descriptor: descriptor.map(ToOwned::to_owned),
            });
        }
        return Ok(MemberQuery {
            name: Some(first.clone()),
            ..Default::default()
        });
    }
    let owner = normalize_class_name(first);
    let name = positionals[1].clone();
    let descriptor = positionals.get(2).cloned();
    if positionals.len() > 3 {
        return Err(CliError::Usage(
            "member query accepts <class> <name> [descriptor]".to_owned(),
        ));
    }
    Ok(MemberQuery {
        owner: Some(owner),
        name: Some(name),
        descriptor,
    })
}

fn parse_member_reference(value: &str) -> Option<(&str, &str, Option<&str>)> {
    let (owner, member) = value
        .split_once("->")
        .or_else(|| value.split_once("::"))
        .or_else(|| value.split_once('#'))?;
    let (name, descriptor) = member
        .split_once(':')
        .map_or((member, None), |(name, descriptor)| {
            (name, Some(descriptor))
        });
    (!owner.is_empty() && !name.is_empty()).then_some((owner, name, descriptor))
}

impl ClassEntry {
    fn members(&self, kind: MemberKind) -> Result<Vec<MemberEntry>, CliError> {
        match &self.payload {
            ClassPayload::Jvm { node } => Ok(jvm_members(self, node, kind)),
            ClassPayload::Dex {
                accessor,
                class_def,
            } => {
                let class_data = if class_def.class_data_off == 0 {
                    None
                } else {
                    Some(
                        accessor
                            .get_class_element(class_def.class_data_off)
                            .map_err(|error| parse_error(&self.source, error))?,
                    )
                };
                let mut result = Vec::new();
                let Some(class_data) = class_data else {
                    return Ok(result);
                };
                if kind == MemberKind::Field {
                    result.extend(
                        class_data
                            .static_fields
                            .into_iter()
                            .chain(class_data.instance_fields)
                            .map(|field| MemberEntry {
                                kind,
                                owner: self.class_name.clone(),
                                owner_descriptor: self.descriptor.clone(),
                                name: field.name.to_string(),
                                descriptor: field.descriptor.to_string(),
                                source: self.source.clone(),
                                dex: Some(self.source.clone()),
                            }),
                    );
                } else {
                    result.extend(
                        class_data
                            .direct_methods
                            .into_iter()
                            .chain(class_data.virtual_methods)
                            .map(|method| MemberEntry {
                                kind,
                                owner: self.class_name.clone(),
                                owner_descriptor: self.descriptor.clone(),
                                name: method.name.to_string(),
                                descriptor: format!(
                                    "({}){}",
                                    method
                                        .parameters
                                        .iter()
                                        .map(|parameter| parameter.as_ref())
                                        .collect::<String>(),
                                    method.return_type,
                                ),
                                source: self.source.clone(),
                                dex: Some(self.source.clone()),
                            }),
                    );
                }
                Ok(result)
            }
        }
    }

    fn render(&self) -> Result<String, CliError> {
        match &self.payload {
            ClassPayload::Jvm { node } => Ok(render_jvm_class(node)),
            ClassPayload::Dex {
                accessor,
                class_def,
            } => accessor
                .get_class_smali(*class_def)
                .map(|node| format!("# source: {}\n{}", self.source, node.render(0)))
                .map_err(|error| parse_error(&self.source, error)),
        }
    }
}

fn jvm_members(class: &ClassEntry, node: &ClassNode, kind: MemberKind) -> Vec<MemberEntry> {
    match kind {
        MemberKind::Field => node
            .fields
            .iter()
            .map(|field| member_from_jvm_field(class, field))
            .collect(),
        MemberKind::Method => node
            .methods
            .iter()
            .map(|method| member_from_jvm_method(class, method))
            .collect(),
    }
}

fn member_from_jvm_field(class: &ClassEntry, field: &FieldNode) -> MemberEntry {
    MemberEntry {
        kind: MemberKind::Field,
        owner: class.class_name.clone(),
        owner_descriptor: class.descriptor.clone(),
        name: field.name.to_string(),
        descriptor: field.desc.to_string(),
        source: class.source.clone(),
        dex: None,
    }
}

fn member_from_jvm_method(class: &ClassEntry, method: &MethodNode) -> MemberEntry {
    MemberEntry {
        kind: MemberKind::Method,
        owner: class.class_name.clone(),
        owner_descriptor: class.descriptor.clone(),
        name: method.name.to_string(),
        descriptor: method.desc.to_string(),
        source: class.source.clone(),
        dex: None,
    }
}

fn class_json(entry: &ClassEntry) -> Value {
    json!({
        "class_name": entry.class_name,
        "descriptor": entry.descriptor,
        "source": entry.source,
        "dex": class_dex_source(entry),
        "can_decompile": true,
    })
}

fn member_json(member: &MemberEntry) -> Value {
    json!({
        "kind": if member.kind == MemberKind::Method { "method" } else { "field" },
        "class_name": member.owner,
        "class_descriptor": member.owner_descriptor,
        "name": member.name,
        "descriptor": member.descriptor,
        "source": member.source,
        "dex": member.dex,
        "can_decompile": true,
    })
}

fn execute_decompile(command: &CommandLine, index: &InputIndex) -> Result<Value, CliError> {
    let selected: Vec<&ClassEntry> = if let Some(filter) = &command.class_filter {
        let normalized = normalize_class_name(filter);
        index
            .classes
            .iter()
            .filter(|entry| entry.class_name == normalized)
            .collect()
    } else {
        index.classes.iter().collect()
    };
    if selected.is_empty() {
        let class = command.class_filter.as_deref().unwrap_or("<all>");
        return Err(CliError::NotFound(format!("class not found: {class}")));
    }
    fs::create_dir_all(&command.output).map_err(|source| CliError::Io {
        path: command.output.clone(),
        source,
    })?;
    let mut outputs = Vec::with_capacity(selected.len());
    for entry in selected {
        let content = entry.render()?;
        let output_path = class_output_path(&command.output, entry);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|source| CliError::Io {
                path: parent.to_owned(),
                source,
            })?;
        }
        fs::write(&output_path, content.as_bytes()).map_err(|source| CliError::Io {
            path: output_path.clone(),
            source,
        })?;
        outputs.push(json!({
            "class_name": entry.class_name,
            "descriptor": entry.descriptor,
            "source": entry.source,
            "output": output_path,
        }));
    }
    let manifest = json!({
        "ok": true,
        "operation": "decompile",
        "input": command.input,
        "output_dir": command.output,
        "count": outputs.len(),
        "classes": outputs,
    });
    let manifest_path = command.output.join("manifest.json");
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| CliError::Usage(format!("serialize manifest: {error}")))?;
    fs::write(&manifest_path, manifest_bytes).map_err(|source| CliError::Io {
        path: manifest_path,
        source,
    })?;
    Ok(manifest)
}

fn class_output_path(output: &Path, entry: &ClassEntry) -> PathBuf {
    let mut result = output.to_owned();
    result.push(safe_component(&source_stem(&entry.source)));
    for part in entry.class_name.split('/') {
        result.push(safe_component(part));
    }
    result.set_extension("smali");
    result
}

fn class_dex_source(entry: &ClassEntry) -> Option<&str> {
    matches!(&entry.payload, ClassPayload::Dex { .. }).then_some(entry.source.as_str())
}

fn source_stem(source: &str) -> String {
    Path::new(source)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "input".to_owned())
}

fn safe_component(value: &str) -> String {
    let result: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | '$') {
                character
            } else {
                '_'
            }
        })
        .collect();
    if result.is_empty() {
        "input".to_owned()
    } else {
        result
    }
}

fn render_jvm_class(node: &ClassNode) -> String {
    let mut output = String::new();
    output.push_str(".class");
    append_words(&mut output, java_class_access(node.access));
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
        append_words(&mut output, java_field_access(field.access));
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
        append_words(&mut output, java_method_access(method.access));
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

fn append_words(output: &mut String, words: Vec<&'static str>) {
    for word in words {
        output.push(' ');
        output.push_str(word);
    }
}

fn java_class_access(flags: u16) -> Vec<&'static str> {
    access_words(
        flags,
        &[
            (0x0001, "public"),
            (0x0002, "private"),
            (0x0004, "protected"),
            (0x0010, "final"),
            (0x0200, "interface"),
            (0x0400, "abstract"),
            (0x1000, "synthetic"),
            (0x2000, "annotation"),
            (0x4000, "enum"),
            (0x8000, "module"),
        ],
    )
}

fn java_method_access(flags: u16) -> Vec<&'static str> {
    access_words(
        flags,
        &[
            (0x0001, "public"),
            (0x0002, "private"),
            (0x0004, "protected"),
            (0x0008, "static"),
            (0x0010, "final"),
            (0x0020, "synchronized"),
            (0x0040, "bridge"),
            (0x0080, "varargs"),
            (0x0100, "native"),
            (0x0400, "abstract"),
            (0x0800, "strict"),
            (0x1000, "synthetic"),
        ],
    )
}

fn java_field_access(flags: u16) -> Vec<&'static str> {
    access_words(
        flags,
        &[
            (0x0001, "public"),
            (0x0002, "private"),
            (0x0004, "protected"),
            (0x0008, "static"),
            (0x0010, "final"),
            (0x0040, "volatile"),
            (0x0080, "transient"),
            (0x1000, "synthetic"),
            (0x4000, "enum"),
        ],
    )
}

fn access_words(flags: u16, known: &[(u16, &'static str)]) -> Vec<&'static str> {
    known
        .iter()
        .filter_map(|(flag, name)| (flags & flag != 0).then_some(*name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        InputIndex, MemberKind, normalize_class_name, parse_member_reference, safe_component,
    };

    #[test]
    fn normalizes_java_class_names() {
        assert_eq!(normalize_class_name("com.example.Main"), "com/example/Main");
        assert_eq!(
            normalize_class_name("Lcom/example/Main;"),
            "com/example/Main"
        );
    }

    #[test]
    fn parses_smali_member_reference() {
        assert_eq!(
            parse_member_reference("Lpkg/Main;->run:(I)V"),
            Some(("Lpkg/Main;", "run", Some("(I)V"))),
        );
        assert_eq!(
            parse_member_reference("Lpkg/Main;#count:I"),
            Some(("Lpkg/Main;", "count", Some("I")))
        );
    }

    #[test]
    fn sanitizes_output_components() {
        assert_eq!(safe_component("classes14.dex"), "classes14.dex");
        assert_eq!(safe_component("a:b"), "a_b");
    }

    #[test]
    fn indexes_jvm_fixture_and_resolves_members() {
        let bytes = include_bytes!("../../asm/tests/res/bytecode/CompileTesting.class");
        let mut index = InputIndex::default();
        index
            .add_embedded("CompileTesting.class", bytes.to_vec())
            .unwrap();

        assert_eq!(index.classes.len(), 1);
        assert_eq!(index.classes[0].class_name, "CompileTesting");
        let methods = index.classes[0].members(MemberKind::Method).unwrap();
        assert!(methods.iter().any(|method| {
            method.name == "main" && method.descriptor == "([Ljava/lang/String;)V"
        }));
    }
}
