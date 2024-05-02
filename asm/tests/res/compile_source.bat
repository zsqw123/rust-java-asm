@echo off
set "sourceDir=.\source"
set "bytecodeDir=.\bytecode"

for %%f in ("%sourceDir%\*.java") do (
    javac -d "%bytecodeDir%" "%%f"
)

echo Compilation complete.