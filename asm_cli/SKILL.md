---
name: asm-cli
description: Find and export classes from APK, APKS, DEX, JAR, ZIP, and JVM class inputs with the native java_asm_cli binary. Use when an Agent needs class names, method signatures, field types, nested APK/DEX locations, targeted Smali output, or filtered bulk export.
---

# ASM CLI

Use `java_asm_cli` to inspect bytecode without starting an app container or MCP server.

## Installation

Prefer downloading the archive for your platform from [GitHub Releases](https://github.com/zsqw123/rust-java-asm/releases/latest), then extract and run `java_asm_cli` (`java_asm_cli.exe` on Windows).

Alternatively, install it from crates.io with Cargo:

```text
cargo install java_asm_cli
```

## Find classes

Run `find-classes` with a dotted name, internal name, or descriptor. Matching and ordering use the same nucleo path matcher as the server: matching is case-insensitive, dots are treated as slash separators, and shorter paths rank first. Omit the query to return every class.

```text
java_asm_cli find-classes app.apks com.example.Main
java_asm_cli findClasses classes.dex Lcom/example/Main;
```

Read the JSON `classes` array. Each class includes `class_name`, `descriptor`, `methods` with names and signatures, and `fields` with names and types.

For archive inputs, preserve the returned `internal_path` exactly. It identifies the DEX or class entry and includes nested package segments, such as `base.apk!classes2.dex`. Standalone DEX and class inputs omit `internal_path`.

## Export one class

Pass the same input, exact class name, and the `internal_path` returned by `find-classes`. Omit `--internal-path` for standalone DEX or class files.

```text
java_asm_cli export-class app.apks com.example.Main --internal-path 'base.apk!classes2.dex'
java_asm_cli exportClass classes.dex com.example.Main
```

Without `--output`, read raw Smali from stdout. To save it directly:

```text
java_asm_cli export-class app.apk com.example.Main --internal-path classes2.dex --output Main.smali
```

If a class occurs in multiple archive entries and no `internal_path` is supplied, rerun with one of the paths reported in the ambiguity error.

## Export many classes

Export all classes to `./asm_cli_output`, or use a fuzzy class filter and another directory.

```text
java_asm_cli export-all app.apk
java_asm_cli exportAll app.apk --class-filter com.example.feature --output exported
```

Read `manifest.json` for exact output paths. Pass `--format smali` explicitly when a workflow should pin the representation; future versions may add other formats.

## Failure handling

Use `java_asm_cli --help` or `java_asm_cli <command> --help` for the current interface. Treat an empty `classes` array as no match. Argument failures use clap diagnostics; parse, lookup, ambiguity, and I/O failures are JSON on stderr with a non-zero exit code.
