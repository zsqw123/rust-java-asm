mod render;

#[cfg(target_family = "wasm")]
compile_error!("java_asm_cli is a native-only executable");

use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use java_asm::AsmErr;
use java_asm::StrRef;
use java_asm::dex::{ClassDef, DexFile, DexFileAccessor};
use java_asm::node::element::ClassNode;
use java_asm_server::fuzzy::FuzzyMatchModel;
use serde_json::{Map, Value, json};
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use zip::ZipArchive;

use crate::render::render_jvm_class;

pub const DEFAULT_OUTPUT_DIR: &str = "asm_cli_output";
const MAX_ARCHIVE_DEPTH: usize = 8;

#[derive(Debug)]
pub enum CliError {
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
    Ambiguous(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Zip { path, message } => write!(f, "{}: {message}", path.display()),
            Self::Parse { source, message } => write!(f, "{source}: {message}"),
            Self::NotFound(message) | Self::Ambiguous(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for CliError {}

#[derive(Debug, Parser)]
#[command(
    name = "java_asm_cli",
    version,
    about = "Find and export classes from Java and Android bytecode",
    after_help = "Find commands emit JSON. export-class writes Smali to stdout unless --output is provided.\n\nExamples:\n  java_asm_cli find-classes app.apks com.example.Main\n  java_asm_cli export-class app.apks com.example.Main --internal-path base.apk!classes2.dex\n  java_asm_cli export-all app.apk --class-filter com.example --output exported",
    arg_required_else_help = true,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(
        visible_alias = "findClasses",
        about = "Find all matching classes and return their basic structure"
    )]
    FindClasses(FindClassesArgs),
    #[command(
        visible_alias = "exportClass",
        about = "Export one class to stdout or a file"
    )]
    ExportClass(ExportClassArgs),
    #[command(
        visible_alias = "exportAll",
        about = "Export all classes matching an optional filter"
    )]
    ExportAll(ExportAllArgs),
}

#[derive(Debug, Args)]
struct FindClassesArgs {
    /// APK, APKS, DEX, JAR, ZIP, class file, or another supported input.
    #[arg(value_name = "INPUT", value_hint = ValueHint::FilePath)]
    input: PathBuf,
    /// Dotted name, slash-separated name, or descriptor. Omit to list every class.
    #[arg(value_name = "QUERY")]
    query: Option<String>,
}

#[derive(Debug, Args)]
struct ExportClassArgs {
    /// The same input file passed to find-classes.
    #[arg(value_name = "INPUT", value_hint = ValueHint::FilePath)]
    input: PathBuf,
    /// Exact dotted name, slash-separated name, or descriptor.
    #[arg(value_name = "CLASS")]
    class_name: String,
    /// Archive path returned by find-classes, for example base.apk!classes2.dex.
    #[arg(long, value_name = "PATH")]
    internal_path: Option<String>,
    /// Write to this file instead of stdout.
    #[arg(short, long, value_name = "FILE", value_hint = ValueHint::FilePath)]
    output: Option<PathBuf>,
    /// Export representation. Additional formats may be added in the future.
    #[arg(long, value_enum, default_value_t = ExportFormat::Smali)]
    format: ExportFormat,
}

#[derive(Debug, Args)]
struct ExportAllArgs {
    /// APK, APKS, DEX, JAR, ZIP, class file, or another supported input.
    #[arg(value_name = "INPUT", value_hint = ValueHint::FilePath)]
    input: PathBuf,
    /// Fuzzy class-name filter. Omit to export every class.
    #[arg(long, alias = "filter", value_name = "QUERY")]
    class_filter: Option<String>,
    /// Directory for exported classes and manifest.json.
    #[arg(
        short,
        long,
        default_value = "asm_cli_output",
        value_name = "DIR",
        value_hint = ValueHint::DirPath
    )]
    output: PathBuf,
    /// Export representation. Additional formats may be added in the future.
    #[arg(long, value_enum, default_value_t = ExportFormat::Smali)]
    format: ExportFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ExportFormat {
    Smali,
}

impl ExportFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Smali => "smali",
        }
    }

    fn extension(self) -> &'static str {
        self.as_str()
    }
}

#[derive(Debug)]
pub enum CliOutput {
    Json(Value),
    Text(String),
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
    internal_name: String,
    descriptor: String,
    internal_path: Option<String>,
    payload: ClassPayload,
}

#[derive(Default)]
struct InputIndex {
    classes: Vec<ClassEntry>,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MethodInfo {
    name: String,
    signature: String,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FieldInfo {
    name: String,
    field_type: String,
}

pub fn execute(cli: Cli) -> Result<CliOutput, CliError> {
    match cli.command {
        Commands::FindClasses(args) => execute_find_classes(args),
        Commands::ExportClass(args) => execute_export_class(args),
        Commands::ExportAll(args) => execute_export_all(args),
    }
}

fn execute_find_classes(args: FindClassesArgs) -> Result<CliOutput, CliError> {
    let index = InputIndex::load(&args.input, None)?;
    let query = args.query.unwrap_or_default();
    let classes = find_matching_classes(&index, &query)
        .into_iter()
        .map(|entry| entry.to_json())
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CliOutput::Json(json!({
        "ok": true,
        "operation": "findClasses",
        "input": args.input,
        "query": query,
        "count": classes.len(),
        "classes": classes,
    })))
}

fn execute_export_class(args: ExportClassArgs) -> Result<CliOutput, CliError> {
    let index = InputIndex::load(&args.input, args.internal_path.as_deref())?;
    let expected = normalize_class_name(&args.class_name);
    let matches: Vec<&ClassEntry> = index
        .classes
        .iter()
        .filter(|entry| entry.internal_name == expected)
        .collect();
    let entry = match matches.as_slice() {
        [] => {
            return Err(CliError::NotFound(format!(
                "class not found: {}",
                args.class_name
            )));
        }
        [entry] => *entry,
        entries => {
            let paths = entries
                .iter()
                .filter_map(|entry| entry.internal_path.as_deref())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(CliError::Ambiguous(format!(
                "class {} exists in multiple locations; pass --internal-path with one of: {paths}",
                args.class_name
            )));
        }
    };
    let content = entry.render(args.format)?;
    let Some(output) = args.output else {
        return Ok(CliOutput::Text(content));
    };
    write_file(&output, content.as_bytes())?;
    let mut result = Map::from_iter([
        ("ok".to_owned(), Value::Bool(true)),
        (
            "operation".to_owned(),
            Value::String("exportClass".to_owned()),
        ),
        ("input".to_owned(), json!(args.input)),
        (
            "class_name".to_owned(),
            Value::String(entry.qualified_name()),
        ),
        (
            "descriptor".to_owned(),
            Value::String(entry.descriptor.clone()),
        ),
        (
            "format".to_owned(),
            Value::String(args.format.as_str().to_owned()),
        ),
        ("output".to_owned(), json!(output)),
    ]);
    insert_internal_path(&mut result, entry.internal_path.as_deref());
    Ok(CliOutput::Json(Value::Object(result)))
}

fn execute_export_all(args: ExportAllArgs) -> Result<CliOutput, CliError> {
    let index = InputIndex::load(&args.input, None)?;
    let filter = args.class_filter.as_deref().unwrap_or_default();
    let selected = find_matching_classes(&index, filter);
    if selected.is_empty() {
        return Err(CliError::NotFound(format!(
            "no classes matched filter: {filter}"
        )));
    }
    fs::create_dir_all(&args.output).map_err(|source| CliError::Io {
        path: args.output.clone(),
        source,
    })?;
    let mut classes = Vec::with_capacity(selected.len());
    for entry in selected {
        let output = class_output_path(&args.output, entry, args.format);
        let content = entry.render(args.format)?;
        write_file(&output, content.as_bytes())?;
        let mut class = Map::from_iter([
            (
                "class_name".to_owned(),
                Value::String(entry.qualified_name()),
            ),
            (
                "descriptor".to_owned(),
                Value::String(entry.descriptor.clone()),
            ),
            ("output".to_owned(), json!(output)),
        ]);
        insert_internal_path(&mut class, entry.internal_path.as_deref());
        classes.push(Value::Object(class));
    }
    let manifest = json!({
        "ok": true,
        "operation": "exportAll",
        "input": args.input,
        "class_filter": args.class_filter,
        "format": args.format.as_str(),
        "output_dir": args.output,
        "count": classes.len(),
        "classes": classes,
    });
    let manifest_path = args.output.join("manifest.json");
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| CliError::Parse {
        source: "manifest.json".to_owned(),
        message: format!("serialize manifest: {error}"),
    })?;
    write_file(&manifest_path, &bytes)?;
    Ok(CliOutput::Json(manifest))
}

impl InputIndex {
    fn load(path: &Path, internal_path: Option<&str>) -> Result<Self, CliError> {
        let bytes = fs::read(path).map_err(|source| CliError::Io {
            path: path.to_owned(),
            source,
        })?;
        let mut index = Self::default();
        if let Some(internal_path) = internal_path {
            let bytes = read_internal_entry(path, bytes, internal_path)?;
            index.collect_embedded(bytes, Some(internal_path.to_owned()), 0)?;
        } else {
            index.collect_embedded(bytes, None, 0)?;
        }
        if index.classes.is_empty() {
            return Err(CliError::Parse {
                source: input_label(path, internal_path),
                message: "no supported DEX or class files found".to_owned(),
            });
        }
        Ok(index)
    }

    fn collect_embedded(
        &mut self,
        bytes: Vec<u8>,
        internal_path: Option<String>,
        depth: usize,
    ) -> Result<(), CliError> {
        if is_dex(&bytes) {
            return self.add_dex(internal_path, bytes);
        }
        if is_class(&bytes) {
            return self.add_class(internal_path, bytes);
        }
        if !is_zip(&bytes) {
            return Ok(());
        }
        self.collect_archive(bytes, internal_path.as_deref(), depth)
    }

    fn collect_archive(
        &mut self,
        bytes: Vec<u8>,
        prefix: Option<&str>,
        depth: usize,
    ) -> Result<(), CliError> {
        if depth >= MAX_ARCHIVE_DEPTH {
            return Err(CliError::Parse {
                source: prefix.unwrap_or("input").to_owned(),
                message: "archive nesting is too deep".to_owned(),
            });
        }
        let archive_label = prefix.unwrap_or("input");
        let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|error| CliError::Zip {
            path: PathBuf::from(archive_label),
            message: error.to_string(),
        })?;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).map_err(|error| CliError::Zip {
                path: PathBuf::from(archive_label),
                message: error.to_string(),
            })?;
            if entry.is_dir() || !is_possible_input_entry(entry.name()) || entry.size() < 4 {
                continue;
            }
            let name = entry.name().to_owned();
            let mut header = [0; 4];
            entry
                .read_exact(&mut header)
                .map_err(|source| CliError::Io {
                    path: PathBuf::from(&name),
                    source,
                })?;
            if !is_supported_magic(&header) {
                continue;
            }
            let mut entry_bytes = Vec::with_capacity(entry.size().min(usize::MAX as u64) as usize);
            entry_bytes.extend_from_slice(&header);
            entry
                .read_to_end(&mut entry_bytes)
                .map_err(|source| CliError::Io {
                    path: PathBuf::from(&name),
                    source,
                })?;
            let entry_path = join_internal_path(prefix, &name);
            drop(entry);
            self.collect_embedded(entry_bytes, Some(entry_path), depth + 1)?;
        }
        Ok(())
    }

    fn add_dex(&mut self, internal_path: Option<String>, bytes: Vec<u8>) -> Result<(), CliError> {
        let source = internal_path.as_deref().unwrap_or("input.dex");
        let file =
            DexFile::resolve_from_bytes(&bytes).map_err(|error| parse_error(source, error))?;
        let accessor = Arc::new(DexFileAccessor::new(file, bytes, source.into()));
        let class_defs = accessor.file.class_defs.clone();
        for class_def in class_defs {
            let descriptor = accessor
                .get_type(class_def.class_idx)
                .map_err(|error| parse_error(source, error))?;
            let Some(internal_name) = descriptor_to_internal(&descriptor) else {
                return Err(CliError::Parse {
                    source: source.to_owned(),
                    message: format!("invalid class descriptor {descriptor}"),
                });
            };
            self.classes.push(ClassEntry {
                internal_name,
                descriptor: descriptor.to_string(),
                internal_path: internal_path.clone(),
                payload: ClassPayload::Dex {
                    accessor: Arc::clone(&accessor),
                    class_def,
                },
            });
        }
        Ok(())
    }

    fn add_class(&mut self, internal_path: Option<String>, bytes: Vec<u8>) -> Result<(), CliError> {
        let source = internal_path.as_deref().unwrap_or("input.class");
        let node = ClassNode::from_bytes(&bytes).map_err(|error| parse_error(source, error))?;
        let internal_name = node.name.to_string();
        self.classes.push(ClassEntry {
            descriptor: format!("L{internal_name};"),
            internal_name,
            internal_path,
            payload: ClassPayload::Jvm {
                node: Arc::new(node),
            },
        });
        Ok(())
    }
}

impl ClassEntry {
    fn qualified_name(&self) -> String {
        self.internal_name.replace('/', ".")
    }

    fn members(&self) -> Result<(Vec<MethodInfo>, Vec<FieldInfo>), CliError> {
        let (mut methods, mut fields) = match &self.payload {
            ClassPayload::Jvm { node } => (
                node.methods
                    .iter()
                    .map(|method| MethodInfo {
                        name: method.name.to_string(),
                        signature: method.desc.to_string(),
                    })
                    .collect(),
                node.fields
                    .iter()
                    .map(|field| FieldInfo {
                        name: field.name.to_string(),
                        field_type: field.desc.to_string(),
                    })
                    .collect(),
            ),
            ClassPayload::Dex {
                accessor,
                class_def,
            } => {
                if class_def.class_data_off == 0 {
                    (Vec::new(), Vec::new())
                } else {
                    let source = self.internal_path.as_deref().unwrap_or("input.dex");
                    let data = accessor
                        .get_class_element(class_def.class_data_off)
                        .map_err(|error| parse_error(source, error))?;
                    let methods = data
                        .direct_methods
                        .into_iter()
                        .chain(data.virtual_methods)
                        .map(|method| MethodInfo {
                            name: method.name.to_string(),
                            signature: format!(
                                "({}){}",
                                method
                                    .parameters
                                    .iter()
                                    .map(|parameter| parameter.as_ref())
                                    .collect::<String>(),
                                method.return_type,
                            ),
                        })
                        .collect();
                    let fields = data
                        .static_fields
                        .into_iter()
                        .chain(data.instance_fields)
                        .map(|field| FieldInfo {
                            name: field.name.to_string(),
                            field_type: field.descriptor.to_string(),
                        })
                        .collect();
                    (methods, fields)
                }
            }
        };
        methods.sort();
        fields.sort();
        Ok((methods, fields))
    }

    fn to_json(&self) -> Result<Value, CliError> {
        let (methods, fields) = self.members()?;
        let mut class = Map::from_iter([
            (
                "class_name".to_owned(),
                Value::String(self.qualified_name()),
            ),
            (
                "descriptor".to_owned(),
                Value::String(self.descriptor.clone()),
            ),
            (
                "methods".to_owned(),
                Value::Array(
                    methods
                        .into_iter()
                        .map(|method| {
                            json!({
                                "name": method.name,
                                "signature": method.signature,
                            })
                        })
                        .collect(),
                ),
            ),
            (
                "fields".to_owned(),
                Value::Array(
                    fields
                        .into_iter()
                        .map(|field| {
                            json!({
                                "name": field.name,
                                "type": field.field_type,
                            })
                        })
                        .collect(),
                ),
            ),
        ]);
        insert_internal_path(&mut class, self.internal_path.as_deref());
        Ok(Value::Object(class))
    }

    fn render(&self, format: ExportFormat) -> Result<String, CliError> {
        match format {
            ExportFormat::Smali => match &self.payload {
                ClassPayload::Jvm { node } => Ok(render_jvm_class(node)),
                ClassPayload::Dex {
                    accessor,
                    class_def,
                } => accessor
                    .get_class_smali(*class_def)
                    .map(|node| node.render(0))
                    .map_err(|error| {
                        parse_error(self.internal_path.as_deref().unwrap_or("input.dex"), error)
                    }),
            },
        }
    }
}

fn find_matching_classes<'a>(index: &'a InputIndex, query: &str) -> Vec<&'a ClassEntry> {
    let keys: Vec<StrRef> = index
        .classes
        .iter()
        .map(|entry| entry.descriptor.clone().into())
        .collect();
    let mut model = FuzzyMatchModel::new(query.into(), &keys, keys.len());
    let result = model.full_search();

    result
        .items
        .into_iter()
        .filter_map(|item| index.classes.get(item.index))
        .collect()
}

fn read_internal_entry(
    input: &Path,
    bytes: Vec<u8>,
    internal_path: &str,
) -> Result<Vec<u8>, CliError> {
    if internal_path.is_empty() {
        return Err(CliError::Parse {
            source: input.display().to_string(),
            message: "internal_path cannot be empty".to_owned(),
        });
    }
    let mut current = bytes;
    let mut consumed = Vec::new();
    for segment in internal_path.split('!') {
        if segment.is_empty() {
            return Err(CliError::Parse {
                source: internal_path.to_owned(),
                message: "internal_path contains an empty segment".to_owned(),
            });
        }
        if !is_zip(&current) {
            return Err(CliError::Parse {
                source: consumed.join("!"),
                message: format!("cannot descend into non-archive entry {segment}"),
            });
        }
        let mut archive = ZipArchive::new(Cursor::new(current)).map_err(|error| CliError::Zip {
            path: input.to_owned(),
            message: error.to_string(),
        })?;
        let mut entry = archive.by_name(segment).map_err(|error| CliError::Zip {
            path: PathBuf::from(internal_path),
            message: error.to_string(),
        })?;
        let mut next = Vec::with_capacity(entry.size().min(usize::MAX as u64) as usize);
        entry
            .read_to_end(&mut next)
            .map_err(|source| CliError::Io {
                path: PathBuf::from(internal_path),
                source,
            })?;
        consumed.push(segment);
        current = next;
    }
    Ok(current)
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| CliError::Io {
            path: parent.to_owned(),
            source,
        })?;
    }
    fs::write(path, bytes).map_err(|source| CliError::Io {
        path: path.to_owned(),
        source,
    })
}

fn class_output_path(output: &Path, entry: &ClassEntry, format: ExportFormat) -> PathBuf {
    let mut result = output.to_owned();
    if matches!(&entry.payload, ClassPayload::Dex { .. }) {
        if let Some(internal_path) = &entry.internal_path {
            result.push(safe_component(internal_path));
        }
    } else if let Some(internal_path) = &entry.internal_path
        && let Some((archive_path, _)) = internal_path.rsplit_once('!')
    {
        result.push(safe_component(archive_path));
    }
    for part in entry.internal_name.split('/') {
        result.push(safe_component(part));
    }
    result.set_extension(format.extension());
    result
}

fn insert_internal_path(result: &mut Map<String, Value>, internal_path: Option<&str>) {
    if let Some(internal_path) = internal_path {
        result.insert(
            "internal_path".to_owned(),
            Value::String(internal_path.to_owned()),
        );
    }
}

fn parse_error(source: &str, error: AsmErr) -> CliError {
    CliError::Parse {
        source: source.to_owned(),
        message: format!("{error:?}"),
    }
}

fn input_label(path: &Path, internal_path: Option<&str>) -> String {
    internal_path.map_or_else(
        || path.display().to_string(),
        |internal_path| format!("{}!{internal_path}", path.display()),
    )
}

fn join_internal_path(prefix: Option<&str>, entry: &str) -> String {
    prefix.map_or_else(|| entry.to_owned(), |prefix| format!("{prefix}!{entry}"))
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

fn is_dex(bytes: &[u8]) -> bool {
    bytes.get(..4) == Some(b"dex\n")
}

fn is_class(bytes: &[u8]) -> bool {
    bytes.get(..4) == Some(&[0xCA, 0xFE, 0xBA, 0xBE])
}

fn is_zip(bytes: &[u8]) -> bool {
    matches!(
        bytes.get(..4),
        Some(b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08")
    )
}

fn is_supported_magic(bytes: &[u8]) -> bool {
    is_dex(bytes) || is_class(bytes) || is_zip(bytes)
}

fn is_possible_input_entry(name: &str) -> bool {
    let Some((_, extension)) = name.rsplit_once('.') else {
        return true;
    };
    ["dex", "class", "apk", "apks", "xapk", "aab", "zip", "jar"]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
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

#[cfg(test)]
mod tests {
    use super::{
        Cli, Commands, ExportFormat, InputIndex, class_output_path, normalize_class_name,
        read_internal_entry,
    };
    use clap::Parser;
    use std::io::{Cursor, Write};
    use std::path::PathBuf;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    fn zip_file(name: &str, bytes: &[u8]) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        writer.write_all(bytes).unwrap();
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn parses_new_command_aliases() {
        let cli = Cli::try_parse_from([
            "java_asm_cli",
            "exportClass",
            "app.apks",
            "com.example.Main",
            "--internal-path",
            "base.apk!classes2.dex",
        ])
        .unwrap();
        let Commands::ExportClass(args) = cli.command else {
            panic!("expected export-class command");
        };
        assert_eq!(args.input, PathBuf::from("app.apks"));
        assert_eq!(args.class_name, "com.example.Main");
        assert_eq!(args.internal_path.as_deref(), Some("base.apk!classes2.dex"));
    }

    #[test]
    fn normalizes_java_class_names() {
        assert_eq!(normalize_class_name("com.example.Main"), "com/example/Main");
        assert_eq!(
            normalize_class_name("Lcom/example/Main;"),
            "com/example/Main"
        );
    }

    #[test]
    fn standalone_class_has_structure_without_internal_path() {
        let bytes = include_bytes!("../../asm/tests/res/bytecode/CompileTesting.class");
        let mut index = InputIndex::default();
        index.collect_embedded(bytes.to_vec(), None, 0).unwrap();

        let class = index.classes[0].to_json().unwrap();
        assert_eq!(class["class_name"], "CompileTesting");
        assert!(class.get("internal_path").is_none());
        assert!(
            class["methods"]
                .as_array()
                .unwrap()
                .iter()
                .any(|method| method["name"] == "main"
                    && method["signature"] == "([Ljava/lang/String;)V")
        );
    }

    #[test]
    fn standalone_dex_omits_internal_path() {
        let dex = include_bytes!("../../asm/tests/res/dex/classes14.dex");
        let mut index = InputIndex::default();
        index.collect_embedded(dex.to_vec(), None, 0).unwrap();

        assert!(!index.classes.is_empty());
        assert!(
            index
                .classes
                .iter()
                .all(|class| class.internal_path.is_none())
        );
        assert!(
            index.classes[0]
                .to_json()
                .unwrap()
                .get("internal_path")
                .is_none()
        );
    }

    #[test]
    fn nested_apks_reports_exportable_internal_path() {
        let dex = include_bytes!("../../asm/tests/res/dex/classes14.dex");
        let apk = zip_file("classes14.dex", dex);
        let apks = zip_file("base.apk", &apk);
        let mut index = InputIndex::default();
        index.collect_embedded(apks, None, 0).unwrap();

        assert!(!index.classes.is_empty());
        assert!(
            index
                .classes
                .iter()
                .all(|class| class.internal_path.as_deref() == Some("base.apk!classes14.dex"))
        );
        let output = class_output_path(
            &PathBuf::from("out"),
            &index.classes[0],
            ExportFormat::Smali,
        );
        assert!(output.starts_with("out/base.apk_classes14.dex"));
        assert_eq!(
            output.extension().and_then(|value| value.to_str()),
            Some("smali")
        );
    }

    #[test]
    fn internal_path_reads_only_the_requested_nested_dex() {
        let dex = include_bytes!("../../asm/tests/res/dex/classes14.dex");
        let apk = zip_file("classes14.dex", dex);
        let apks = zip_file("base.apk", &apk);
        let input = PathBuf::from("sample.apks");

        let selected = read_internal_entry(&input, apks, "base.apk!classes14.dex").unwrap();
        assert_eq!(selected, dex);
    }
}
