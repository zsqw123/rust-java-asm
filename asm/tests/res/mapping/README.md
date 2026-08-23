# R8 mapping integration fixture

This fixture provides a small Java program with class, field, overloaded method,
array descriptor, and source-line mappings. Generated files can be opened directly
in the desktop or WASM UI:

- `generated/mapping-fixture.dex`: standalone DEX input.
- `generated/mapping-fixture.apk`: APK-shaped ZIP containing `classes.dex`.
- `generated/mapping-fixture.apks`: nested archive containing the APK.
- `generated/mapping-fixture.mapping.txt`: mapping imported through **Open mapping...**.

The `generated/` directory is intentionally ignored; rebuild it before manual testing.

## Rebuild

Use JDK 17 or newer and an R8 jar from Android command-line tools:

```powershell
./build.ps1 -R8Jar D:\path\to\android-sdk\cmdline-tools\lib\r8.jar
```

If `ANDROID_HOME` or `ANDROID_SDK_ROOT` points to an SDK with
`cmdline-tools/lib/r8.jar`, the parameter can be omitted.

The fixture deliberately disables shrinking and optimization so every declaration
remains available and line mappings are direct rather than inline-frame mappings.
R8 minification remains enabled.

## Manual checks

1. Open any generated package. The tree and Smali initially use short raw names.
2. Import `mapping-fixture.mapping.txt`. The tree, tabs, Smali, export, and search
   should switch to `mapping/fixture/...` names.
3. Search for `MappingEntry`, `UserService`, or `overloaded`.
4. Hover or click a mapped member or descriptor to see its raw name.
5. Click a displayed class descriptor and verify that navigation opens the raw class.
6. Inspect `.source-line` values in `UserService.findUser` and hover a changed line
   to see its raw line number.
