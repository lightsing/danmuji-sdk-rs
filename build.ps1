param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Release",

    [string]$RustPackage = "danmuji-rust-plugin",

    [string]$RustLibraryName = "danmuji_rust_plugin",

    [string]$OutputDirectory = "dist"
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$RustProfileArgs = @()
$RustProfileDir = "debug"

if ($Configuration -eq "Release") {
    $RustProfileArgs = @("--release")
    $RustProfileDir = "release"
}

Push-Location $Root
try {
    cargo build -p $RustPackage @RustProfileArgs

    dotnet build "bridge\DanmujiRustBridge\DanmujiRustBridge.csproj" -c $Configuration

    if ([System.IO.Path]::IsPathRooted($OutputDirectory)) {
        $Dist = $OutputDirectory
    }
    else {
        $Dist = Join-Path $Root $OutputDirectory
    }

    New-Item -ItemType Directory -Force -Path $Dist | Out-Null

    $BridgeOut = Join-Path $Root "bridge\DanmujiRustBridge\bin\$Configuration\net461"
    $NativeDll = Join-Path $Root "target\$RustProfileDir\$RustLibraryName.dll"

    Copy-Item -Force (Join-Path $BridgeOut "DanmujiRustBridge.dll") $Dist
    Copy-Item -Force (Join-Path $BridgeOut "Newtonsoft.Json.dll") $Dist
    Copy-Item -Force (Join-Path $Root "vendor\BilibiliDM_PluginFramework.dll") $Dist
    Copy-Item -Force $NativeDll (Join-Path $Dist "danmuji_rust_plugin.dll")

    Write-Host "Packaged plugin files in $Dist"
}
finally {
    Pop-Location
}
