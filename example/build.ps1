param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Release",

    [switch]$RebuildBridge
)

$ErrorActionPreference = "Stop"

$ExampleRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Split-Path -Parent $ExampleRoot

$BuildArgs = @{
    Configuration = $Configuration
    RustPackage = "danmuji-rust-example-plugin"
    RustLibraryName = "danmuji_rust_example_plugin"
    OutputDirectory = "example\dist"
}

if ($RebuildBridge) {
    $BuildArgs.RebuildBridge = $true
}

& (Join-Path $Root "build.ps1") @BuildArgs
