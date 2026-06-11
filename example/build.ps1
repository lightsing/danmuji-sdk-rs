param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Release"
)

$ErrorActionPreference = "Stop"

$ExampleRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Split-Path -Parent $ExampleRoot

& (Join-Path $Root "build.ps1") `
    -Configuration $Configuration `
    -RustPackage "danmuji-rust-example-plugin" `
    -RustLibraryName "danmuji_rust_example_plugin" `
    -OutputDirectory "example\dist"

