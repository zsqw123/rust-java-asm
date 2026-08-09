---
name: asm-cli
description: Inspect APK, DEX, JAR, ZIP and JVM class files with the repository's native asm_cli binary. Use when an Agent needs to find a class, method or field, identify its containing DEX/archive entry, or decompile all or a targeted class into predictable Smali-like files.
---

# ASM CLI

## Overview

Use the native `asm_cli` executable as a fast, machine-readable bridge to the bytecode parser. Search commands print JSON; decompilation writes Smali-like output and a `manifest.json`.

Build it from the repository root with `cargo build --release -p asm_cli`; use `target/release/asm_cli.exe` on Windows or `target/release/asm_cli` on other native targets.

## Workflow

1. Find the class or member in the original input.
2. Read the JSON `source` field. For APK/JAR inputs this is normally an entry such as `classes14.dex`.
3. Decompile only that class and source entry with `--class` and `--source`.
4. Read the output path from the JSON response or inspect the generated `manifest.json`.

## Search

Run a fuzzy class search. Class queries accept dotted names, slash-separated internal names, and descriptors.

```text
asm_cli find-class app.apk com.example.Main
asm_cli findClass app.apk Lcom/example/Main;
```

Find members by class/name/descriptor. Omit the descriptor to return overloads.

```text
asm_cli find-method app.apk com.example.Main onCreate '(Landroid/os/Bundle;)V'
asm_cli find-field app.apk com.example.Main count I
asm_cli findMethod app.apk 'Lcom/example/Main;->onCreate:(Landroid/os/Bundle;)V'
asm_cli findField app.apk 'Lcom/example/Main;#count:I'
```

Member results include `class_name`, `class_descriptor`, `name`, `descriptor`, `source`, and `dex` (for DEX-backed inputs); class results use `descriptor` for the class descriptor. Use `--limit N` to bound results.

## Decompile

Decompile every supported class to `./asm_cli_output`:

```text
asm_cli decompile app.apk
```

Target one class and one APK DEX entry after a search:

```text
asm_cli decompile app.apk --class com.example.Main --source classes14.dex
```

Pass `--output DIR` when a separate run directory is needed. The response and `manifest.json` list exact output paths.

## Input and failure handling

Accept standalone `.class`, `.dex`, `.jar`, `.apk`, and ZIP files. The loader also sniffs magic bytes and does not create an app container or start the GUI.

Treat an empty `results` array as “not found”. Non-usage failures are emitted as JSON on stderr and return a non-zero exit code. `--help` and malformed arguments return exit code 2.
