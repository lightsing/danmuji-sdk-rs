param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Release",

    [Parameter(Mandatory = $true)]
    [string]$RustPackage,

    [Parameter(Mandatory = $true)]
    [string]$RustLibraryName,

    [string]$OutputDirectory = "dist",

    [switch]$RebuildBridge
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

    if ($RebuildBridge) {
        dotnet build "bridge\DanmujiRustBridge\DanmujiRustBridge.csproj" -c $Configuration
        $BridgeOut = Join-Path $Root "bridge\DanmujiRustBridge\bin\$Configuration\net461"
    }
    else {
        $BridgeOut = Join-Path $Root "bridge\prebuilt"
    }

    if ([System.IO.Path]::IsPathRooted($OutputDirectory)) {
        $Dist = $OutputDirectory
    }
    else {
        $Dist = Join-Path $Root $OutputDirectory
    }

    New-Item -ItemType Directory -Force -Path $Dist | Out-Null

    $NativeDll = Join-Path $Root "target\$RustProfileDir\$RustLibraryName.dll"
    $BridgeDll = Join-Path $BridgeOut "DanmujiRustBridge.dll"
    $NewtonsoftDll = Join-Path $BridgeOut "Newtonsoft.Json.dll"

    if (!(Test-Path $BridgeDll) -or !(Test-Path $NewtonsoftDll)) {
        throw "Missing bridge runtime files in $BridgeOut. Run .\build.ps1 -RebuildBridge once or restore bridge\prebuilt."
    }

    Copy-Item -Force $BridgeDll $Dist
    Copy-Item -Force $NewtonsoftDll $Dist
    Copy-Item -Force (Join-Path $Root "vendor\BilibiliDM_PluginFramework.dll") $Dist
    Copy-Item -Force $NativeDll (Join-Path $Dist "danmuji_rust_plugin.dll")

    Write-Host "Packaged plugin files in $Dist"
}
finally {
    Pop-Location
}
