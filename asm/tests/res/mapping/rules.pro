-dontshrink
-dontoptimize
-dontwarn
-keepattributes SourceFile,LineNumberTable

# Keep an executable entry point while still allowing its class and members to be renamed.
-keep,allowobfuscation,allowoptimization class mapping.fixture.MappingEntry {
    public static void main(java.lang.String[]);
}
