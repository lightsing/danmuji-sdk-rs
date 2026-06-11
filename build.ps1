param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Release"
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
    cargo build -p danmuji-rust-plugin @RustProfileArgs

    dotnet build "bridge\DanmujiRustBridge\DanmujiRustBridge.csproj" -c $Configuration

    $Dist = Join-Path $Root "dist"
    New-Item -ItemType Directory -Force -Path $Dist | Out-Null

    $BridgeOut = Join-Path $Root "bridge\DanmujiRustBridge\bin\$Configuration\net461"
    Copy-Item -Force (Join-Path $BridgeOut "DanmujiRustBridge.dll") $Dist
    Copy-Item -Force (Join-Path $BridgeOut "Newtonsoft.Json.dll") $Dist
    Copy-Item -Force (Join-Path $Root "vendor\BilibiliDM_PluginFramework.dll") $Dist
    Copy-Item -Force (Join-Path $Root "target\$RustProfileDir\danmuji_rust_plugin.dll") $Dist

    Write-Host "Packaged plugin files in $Dist"
}
finally {
    Pop-Location
}

