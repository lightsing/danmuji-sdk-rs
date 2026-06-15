use std::cmp::Ordering;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use eyre::{eyre, Result, WrapErr};
use sha2::{Digest, Sha256};

pub const DEFAULT_SDK_REPO: &str = "https://github.com/copyliu/bililive_dm.git";

const VERSION_FILE: &str = ".danmuji-version";
const BRIDGE_DLL: &str = "DanmujiRustBridge.dll";
const BRIDGE_TEMPLATE_VERSION: &str = "5";
const SDK_SOURCE_DIR: &str = "BilibiliDM_PluginFramework";
const RUST_PLUGIN_CS: &str = include_str!("../bridge/RustPlugin.cs");
const RUST_NATIVE_CS: &str = include_str!("../bridge/RustNative.cs");
const FFI_SCOPE_CS: &str = include_str!("../bridge/FfiScope.cs");

pub fn read_bridge_template(
    explicit_template: Option<&Path>,
    project_dir: &Path,
    sdk_repo: &str,
    explicit_sdk_version: Option<&str>,
    refresh_sdk: bool,
) -> Result<Vec<u8>> {
    if let Some(path) = explicit_template {
        return Ok(fs::read(path).wrap_err_with(|| format!("failed to read {}", path.display()))?);
    }

    let sdk_version = resolve_sdk_version(explicit_sdk_version, project_dir, sdk_repo)?;
    let bridge = ensure_bridge_template(sdk_repo, &sdk_version, refresh_sdk)?;
    fs::read(&bridge).wrap_err_with(|| format!("failed to read {}", bridge.display()))
}

pub fn write_version_file(root: &Path, sdk_version: &str) -> Result<()> {
    let path = root.join(VERSION_FILE);
    write_version_path(&path, sdk_version)
}

pub struct VersionUpgrade {
    pub path: PathBuf,
    pub old_version: Option<String>,
    pub new_version: String,
}

pub fn upgrade_version_file(project_dir: &Path, sdk_repo: &str) -> Result<VersionUpgrade> {
    let path = find_version_file(project_dir).unwrap_or_else(|| project_dir.join(VERSION_FILE));
    let old_version = if path.exists() {
        Some(read_version_file(&path)?)
    } else {
        None
    };
    let new_version = latest_sdk_version(sdk_repo)?;

    write_version_path(&path, &new_version)?;

    Ok(VersionUpgrade {
        path,
        old_version,
        new_version,
    })
}

fn resolve_sdk_version(
    explicit: Option<&str>,
    project_dir: &Path,
    sdk_repo: &str,
) -> Result<String> {
    if let Some(version) = explicit {
        return non_empty_version(version, "--sdk-version");
    }

    if let Some(path) = find_version_file(project_dir) {
        return read_version_file(&path);
    }

    let latest = latest_sdk_version(sdk_repo)?;
    let path = project_dir.join(VERSION_FILE);
    write_version_path(&path, &latest)?;
    eprintln!("Created {} with latest SDK {}", path.display(), latest);
    Ok(latest)
}

fn read_version_file(path: &Path) -> Result<String> {
    let text =
        fs::read_to_string(path).wrap_err_with(|| format!("failed to read {}", path.display()))?;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if !line.is_empty() {
            return Ok(line.to_string());
        }
    }

    Err(eyre!("{} is empty", path.display()))
}

fn write_version_path(path: &Path, sdk_version: &str) -> Result<()> {
    let sdk_version = non_empty_version(sdk_version, "sdk version")?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("failed to create {}", parent.display()))?;
        }
    }

    fs::write(path, format!("{sdk_version}\n"))
        .wrap_err_with(|| format!("failed to write {}", path.display()))
}

fn latest_sdk_version(sdk_repo: &str) -> Result<String> {
    let output = Command::new("git")
        .arg("ls-remote")
        .arg("--tags")
        .arg("--refs")
        .arg(sdk_repo)
        .output()
        .wrap_err("failed to run git ls-remote")?;

    if !output.status.success() {
        return Err(eyre!(
            "git ls-remote --tags --refs {} failed with status {}",
            sdk_repo,
            output.status
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut latest: Option<(Vec<u64>, String)> = None;

    for line in stdout.lines() {
        let Some(reference) = line.split_whitespace().nth(1) else {
            continue;
        };
        let Some(tag) = reference.strip_prefix("refs/tags/") else {
            continue;
        };
        let Some(key) = numeric_tag_key(tag) else {
            continue;
        };

        if latest
            .as_ref()
            .map(|(latest_key, _)| compare_version_key(&key, latest_key).is_gt())
            .unwrap_or(true)
        {
            latest = Some((key, tag.to_string()));
        }
    }

    latest
        .map(|(_, tag)| tag)
        .ok_or_else(|| eyre!("no numeric SDK tags found in {}", sdk_repo))
}

fn numeric_tag_key(tag: &str) -> Option<Vec<u64>> {
    let tag = tag.strip_prefix('v').unwrap_or(tag);
    let mut key = Vec::new();

    for part in tag.split('.') {
        if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }

        key.push(part.parse().ok()?);
    }

    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn compare_version_key(left: &[u64], right: &[u64]) -> Ordering {
    let len = left.len().max(right.len());
    for index in 0..len {
        let left = left.get(index).copied().unwrap_or(0);
        let right = right.get(index).copied().unwrap_or(0);

        match left.cmp(&right) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }

    Ordering::Equal
}

fn non_empty_version(version: &str, source: &str) -> Result<String> {
    let version = version.trim();
    if version.is_empty() {
        Err(eyre!("{source} cannot be empty"))
    } else {
        Ok(version.to_string())
    }
}

fn find_version_file(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let path = dir.join(VERSION_FILE);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn ensure_bridge_template(sdk_repo: &str, sdk_version: &str, refresh_sdk: bool) -> Result<PathBuf> {
    let checkout = ensure_sdk_checkout(sdk_repo, sdk_version, refresh_sdk)?;
    let sdk_source_dir = checkout.path.join(SDK_SOURCE_DIR);
    if !sdk_source_dir.exists() {
        return Err(eyre!(
            "B站弹幕姬 SDK source directory not found at {}",
            sdk_source_dir.display()
        ));
    }

    let key = bridge_cache_key(sdk_repo, sdk_version, &checkout.revision);
    let bridge_dir = cache_root().join("bridge").join(key);
    let output = bridge_dir
        .join("project")
        .join("bin")
        .join("Release")
        .join("net461")
        .join(BRIDGE_DLL);

    if output.exists() {
        eprintln!(
            "Using cached B站弹幕姬 bridge for SDK {sdk_version} ({})",
            short_revision(&checkout.revision)
        );
        return Ok(output);
    }

    let project_dir = bridge_dir.join("project");
    fs::create_dir_all(&project_dir)
        .wrap_err_with(|| format!("failed to create {}", project_dir.display()))?;

    write_bridge_project(&project_dir, &sdk_source_dir)?;

    eprintln!(
        "Building B站弹幕姬 bridge for SDK {sdk_version} ({})",
        short_revision(&checkout.revision)
    );
    let mut command = Command::new("dotnet");
    command
        .arg("build")
        .arg(project_dir.join("DanmujiRustBridge.csproj"))
        .arg("-c")
        .arg("Release")
        .arg("--nologo");
    run_process(command, "dotnet build DanmujiRustBridge")?;

    if !output.exists() {
        return Err(eyre!("bridge DLL was not produced at {}", output.display()));
    }

    Ok(output)
}

struct SdkCheckout {
    path: PathBuf,
    revision: String,
}

fn ensure_sdk_checkout(
    sdk_repo: &str,
    sdk_version: &str,
    refresh_sdk: bool,
) -> Result<SdkCheckout> {
    let source_dir = cache_root().join("upstream").join(hash_hex(&[
        sdk_repo.as_bytes(),
        b"\0",
        sdk_version.as_bytes(),
    ]));

    if refresh_sdk && source_dir.exists() {
        fs::remove_dir_all(&source_dir)
            .wrap_err_with(|| format!("failed to remove {}", source_dir.display()))?;
    }

    if !source_dir.join(".git").exists() {
        if source_dir.exists() {
            fs::remove_dir_all(&source_dir)
                .wrap_err_with(|| format!("failed to remove {}", source_dir.display()))?;
        }

        fs::create_dir_all(&source_dir)
            .wrap_err_with(|| format!("failed to create {}", source_dir.display()))?;

        let mut init = Command::new("git");
        init.arg("init").arg(&source_dir);
        run_process(init, "git init")?;

        let mut remote = Command::new("git");
        remote
            .arg("-C")
            .arg(&source_dir)
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg(sdk_repo);
        run_process(remote, "git remote add")?;

        fetch_checkout(&source_dir, sdk_version)?;
    }

    let revision = match git_output(&source_dir, &["rev-parse", "HEAD"]) {
        Ok(revision) => revision,
        Err(_) => {
            fetch_checkout(&source_dir, sdk_version)?;
            git_output(&source_dir, &["rev-parse", "HEAD"])?
        }
    };

    Ok(SdkCheckout {
        path: source_dir,
        revision,
    })
}

fn fetch_checkout(source_dir: &Path, sdk_version: &str) -> Result<()> {
    let mut fetch = Command::new("git");
    fetch
        .arg("-C")
        .arg(source_dir)
        .arg("fetch")
        .arg("--depth")
        .arg("1")
        .arg("origin")
        .arg(sdk_version);
    run_process(fetch, "git fetch B站弹幕姬 SDK")?;

    let mut checkout = Command::new("git");
    checkout
        .arg("-C")
        .arg(source_dir)
        .arg("checkout")
        .arg("--detach")
        .arg("FETCH_HEAD");
    run_process(checkout, "git checkout B站弹幕姬 SDK")?;

    Ok(())
}

fn write_bridge_project(project_dir: &Path, sdk_source_dir: &Path) -> Result<()> {
    let sdk_project_dir = project_dir.join("sdk");
    fs::create_dir_all(&sdk_project_dir)
        .wrap_err_with(|| format!("failed to create {}", sdk_project_dir.display()))?;

    write_file(
        sdk_project_dir.join("BilibiliDM_PluginFramework.csproj"),
        &sdk_csproj(sdk_source_dir),
    )?;
    write_file(project_dir.join("RustPlugin.cs"), RUST_PLUGIN_CS)?;
    write_file(project_dir.join("RustNative.cs"), RUST_NATIVE_CS)?;
    write_file(project_dir.join("FfiScope.cs"), FFI_SCOPE_CS)?;
    write_file(
        project_dir.join("DanmujiRustBridge.csproj"),
        &bridge_csproj(&sdk_project_dir.join("BilibiliDM_PluginFramework.csproj")),
    )?;
    Ok(())
}

fn write_file(path: PathBuf, contents: &str) -> Result<()> {
    fs::write(&path, contents).wrap_err_with(|| format!("failed to write {}", path.display()))
}

fn sdk_csproj(sdk_source_dir: &Path) -> String {
    let source = |path: &str| escape_xml_attr(&sdk_source_dir.join(path).display().to_string());
    format!(
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net461</TargetFramework>
    <AssemblyName>BilibiliDM_PluginFramework</AssemblyName>
    <RootNamespace>BilibiliDM_PluginFramework</RootNamespace>
    <LangVersion>7.3</LangVersion>
    <EnableDefaultCompileItems>false</EnableDefaultCompileItems>
    <GenerateAssemblyInfo>false</GenerateAssemblyInfo>
    <NoWarn>$(NoWarn);0618</NoWarn>
  </PropertyGroup>

  <ItemGroup>
    <Compile Include="{}" Link="DMPlugin.cs" />
    <Compile Include="{}" Link="DanmakuModel.cs" />
    <Compile Include="{}" Link="Events.cs" />
    <Compile Include="{}" Link="GiftRank.cs" />
    <Compile Include="{}" Link="Properties\Annotations.cs" />
    <Compile Include="{}" Link="Properties\AssemblyInfo.cs" />
  </ItemGroup>

  <ItemGroup>
    <Reference Include="Microsoft.CSharp" />
    <Reference Include="PresentationCore" />
    <Reference Include="PresentationFramework" />
    <Reference Include="System.Xaml" />
    <Reference Include="WindowsBase" />
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.NETFramework.ReferenceAssemblies.net461" Version="1.0.3" PrivateAssets="all" />
    <PackageReference Include="Newtonsoft.Json" Version="13.0.1" />
  </ItemGroup>
</Project>
"#,
        source("DMPlugin.cs"),
        source("DanmakuModel.cs"),
        source("Events.cs"),
        source("GiftRank.cs"),
        source("Properties/Annotations.cs"),
        source("Properties/AssemblyInfo.cs"),
    )
}

fn bridge_csproj(sdk_project: &Path) -> String {
    let sdk_project = escape_xml_attr(&sdk_project.display().to_string());
    format!(
        r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net461</TargetFramework>
    <AssemblyName>DanmujiRustBridge</AssemblyName>
    <RootNamespace>DanmujiSdkRust.Bridge</RootNamespace>
    <LangVersion>7.3</LangVersion>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include="{sdk_project}" Private="false" />
    <Reference Include="WindowsBase" />
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.NETFramework.ReferenceAssemblies.net461" Version="1.0.3" PrivateAssets="all" />
    <PackageReference Include="Newtonsoft.Json" Version="13.0.1" />
  </ItemGroup>
</Project>
"#
    )
}

fn bridge_cache_key(sdk_repo: &str, sdk_version: &str, revision: &str) -> String {
    hash_hex(&[
        sdk_repo.as_bytes(),
        b"\0",
        sdk_version.as_bytes(),
        b"\0",
        revision.as_bytes(),
        b"\0",
        BRIDGE_TEMPLATE_VERSION.as_bytes(),
        b"\0",
        RUST_PLUGIN_CS.as_bytes(),
        b"\0",
        RUST_NATIVE_CS.as_bytes(),
        b"\0",
        FFI_SCOPE_CS.as_bytes(),
    ])
}

fn cache_root() -> PathBuf {
    env::var_os("DANMUJI_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("danmuji-sdk-rs").join("cargo-danmuji"))
}

fn run_process(mut command: Command, label: &str) -> Result<()> {
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .wrap_err_with(|| format!("failed to run {label}"))?;

    if !status.success() {
        return Err(eyre!("{label} failed with status {status}"));
    }

    Ok(())
}

fn git_output(source_dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(source_dir)
        .args(args)
        .output()
        .wrap_err("failed to run git")?;

    if !output.status.success() {
        return Err(eyre!(
            "git {} failed with status {}",
            args.join(" "),
            output.status
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn hash_hex(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }

    to_hex(&hasher.finalize())
}

fn to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(hex_char(byte >> 4));
        output.push(hex_char(byte & 0x0f));
    }

    output
}

fn hex_char(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => unreachable!(),
    }
}

fn short_revision(revision: &str) -> &str {
    revision.get(..12).unwrap_or(revision)
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
