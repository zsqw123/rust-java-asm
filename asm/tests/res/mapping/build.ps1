param(
    [string]$R8Jar
)

$ErrorActionPreference = 'Stop'
$fixtureRoot = $PSScriptRoot
$buildRoot = Join-Path $fixtureRoot 'build'
$classesRoot = Join-Path $buildRoot 'classes'
$generatedRoot = Join-Path $fixtureRoot 'generated'
$programJar = Join-Path $buildRoot 'mapping-fixture.jar'
$dexZip = Join-Path $buildRoot 'mapping-fixture.zip'
$mappingFile = Join-Path $generatedRoot 'mapping-fixture.mapping.txt'
$apkFile = Join-Path $generatedRoot 'mapping-fixture.apk'
$dexFile = Join-Path $generatedRoot 'mapping-fixture.dex'
$apksFile = Join-Path $generatedRoot 'mapping-fixture.apks'

if (-not $R8Jar) {
    $sdkRoots = @($env:ANDROID_HOME, $env:ANDROID_SDK_ROOT) | Where-Object { $_ }
    foreach ($sdkRoot in $sdkRoots) {
        $candidate = Join-Path $sdkRoot 'cmdline-tools\lib\r8.jar'
        if (Test-Path -LiteralPath $candidate) {
            $R8Jar = $candidate
            break
        }
    }
}
if (-not $R8Jar -or -not (Test-Path -LiteralPath $R8Jar)) {
    throw 'R8 was not found. Pass its jar with -R8Jar <path-to-r8.jar>.'
}
if (-not $env:JAVA_HOME) {
    throw 'JAVA_HOME must point to a JDK.'
}

New-Item -ItemType Directory -Force -Path $classesRoot, $generatedRoot | Out-Null
$sources = Get-ChildItem -LiteralPath (Join-Path $fixtureRoot 'src') -Recurse -File -Filter '*.java'
if (-not $sources) {
    throw 'No Java sources were found.'
}

& javac -g -encoding UTF-8 -d $classesRoot $sources.FullName
if ($LASTEXITCODE -ne 0) { throw "javac failed with exit code $LASTEXITCODE" }

& jar --create --file $programJar -C $classesRoot .
if ($LASTEXITCODE -ne 0) { throw "jar failed with exit code $LASTEXITCODE" }

Remove-Item -LiteralPath $dexZip, $mappingFile, $apkFile, $dexFile, $apksFile -Force -ErrorAction SilentlyContinue
& java -cp $R8Jar com.android.tools.r8.R8 `
    --release `
    --min-api 21 `
    --lib $env:JAVA_HOME `
    --pg-conf (Join-Path $fixtureRoot 'rules.pro') `
    --pg-map-output $mappingFile `
    --output $dexZip `
    $programJar
if ($LASTEXITCODE -ne 0) { throw "R8 failed with exit code $LASTEXITCODE" }

Copy-Item -LiteralPath $dexZip -Destination $apkFile
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::OpenRead($dexZip)
try {
    $entry = $archive.GetEntry('classes.dex')
    if (-not $entry) { throw 'R8 output does not contain classes.dex.' }
    [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $dexFile, $true)
} finally {
    $archive.Dispose()
}

$apksStaging = Join-Path $buildRoot 'apks'
New-Item -ItemType Directory -Force -Path $apksStaging | Out-Null
$nestedApk = Join-Path $apksStaging 'base-master.apk'
Copy-Item -LiteralPath $apkFile -Destination $nestedApk -Force
Compress-Archive -LiteralPath $nestedApk -DestinationPath $apksFile -CompressionLevel Optimal

Write-Output "Generated:"
Write-Output "  $dexFile"
Write-Output "  $apkFile"
Write-Output "  $apksFile"
Write-Output "  $mappingFile"
