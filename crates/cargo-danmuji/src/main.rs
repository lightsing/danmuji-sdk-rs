mod bridge;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cargo_metadata::{Metadata, MetadataCommand, Package};
use clap::{Args, Parser, Subcommand};
use clap_cargo::{Features, Manifest, Workspace};
use eyre::{eyre, Result, WrapErr};
use sha2::{Digest, Sha256};

const OVERLAY_MAGIC: &[u8; 16] = b"DMJRSOVL00000001";
const FOOTER_VERSION: u32 = 1;

fn main() {
    if let Err(error) = run() {
        eprintln!("cargo-danmuji: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let CargoCli::Danmuji(cli) = CargoCli::parse();

    match cli.command {
        Commands::Build(options) => build(*options),
        Commands::Package(options) => package(options),
        Commands::New(options) => create_new(options),
        Commands::Upgrade(options) => upgrade(options),
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "cargo",
    bin_name = "cargo",
    styles = clap_cargo::style::CLAP_STYLING
)]
enum CargoCli {
    #[command(name = "danmuji", version, about, long_about = None)]
    Danmuji(DanmujiArgs),
}

#[derive(Args, Debug)]
#[command(about = "Build and package Rust plugins for B站弹幕姬")]
struct DanmujiArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Build a Rust cdylib and package a single B站弹幕姬 plugin DLL")]
    Build(Box<BuildOptions>),
    #[command(about = "Package an existing native DLL into a single B站弹幕姬 plugin DLL")]
    Package(PackageOptions),
    #[command(about = "Create a new Rust B站弹幕姬 plugin project")]
    New(NewOptions),
    #[command(about = "Update .danmuji-version to the latest B站弹幕姬 SDK tag")]
    Upgrade(UpgradeOptions),
}

#[derive(Args, Debug)]
struct BuildOptions {
    #[command(flatten)]
    manifest: Manifest,

    #[command(flatten)]
    workspace: Workspace,

    #[command(flatten)]
    features: Features,

    #[arg(long)]
    lib_name: Option<String>,

    #[arg(long)]
    release: bool,

    #[arg(long)]
    target: Option<String>,

    #[arg(long)]
    target_dir: Option<PathBuf>,

    #[arg(long, default_value = "dist")]
    out_dir: PathBuf,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[command(flatten)]
    bridge: BridgeOptions,

    #[arg(last = true)]
    cargo_args: Vec<String>,
}

#[derive(Args, Clone, Debug)]
struct BridgeOptions {
    #[arg(long)]
    sdk_version: Option<String>,

    #[arg(long, default_value_t = bridge::DEFAULT_SDK_REPO.to_string())]
    sdk_repo: String,

    #[arg(long)]
    refresh_sdk: bool,

    #[arg(long)]
    template: Option<PathBuf>,
}

fn build(options: BuildOptions) -> Result<()> {
    if options.workspace.workspace || options.workspace.all || !options.workspace.exclude.is_empty()
    {
        return Err(eyre!(
            "cargo danmuji build packages one plugin DLL; use --package to select one package"
        ));
    }

    if options.workspace.package.len() > 1 {
        return Err(eyre!(
            "cargo danmuji build packages one plugin DLL; pass only one --package"
        ));
    }

    let manifest_path = options
        .manifest
        .manifest_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("Cargo.toml"));
    let manifest_path = absolute_path(&manifest_path)?;
    let manifest_dir = manifest_path
        .parent()
        .ok_or_else(|| eyre!("manifest path has no parent directory"))?;

    let metadata = load_metadata(&manifest_path)?;
    let package = select_package(
        &metadata,
        &manifest_path,
        options.workspace.package.first().map(String::as_str),
    )?;
    let package_name = package.name.clone();
    let lib_name = match options.lib_name.clone() {
        Some(lib_name) => lib_name,
        None => cdylib_target_name(package)?,
    };

    let target_dir = options
        .target_dir
        .clone()
        .unwrap_or_else(|| manifest_dir.join("target"));
    let profile_dir = if options.release { "release" } else { "debug" };

    let mut command = Command::new(cargo_binary());
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--target-dir")
        .arg(&target_dir);

    if !options.workspace.package.is_empty() {
        command.arg("--package").arg(&package_name);
    }

    if options.features.all_features {
        command.arg("--all-features");
    }

    if options.features.no_default_features {
        command.arg("--no-default-features");
    }

    for feature in &options.features.features {
        command.arg("--features").arg(feature);
    }

    if options.release {
        command.arg("--release");
    }

    if let Some(target) = &options.target {
        command.arg("--target").arg(target);
    }

    command.args(&options.cargo_args);

    run_command(command)?;

    let native_dll = native_dll_path(
        &target_dir,
        options.target.as_deref(),
        profile_dir,
        &lib_name,
    );
    if !native_dll.exists() {
        return Err(eyre!("native DLL not found: {}", native_dll.display()));
    }

    let output = options.output.unwrap_or_else(|| {
        options
            .out_dir
            .join(format!("{}.dll", package_name.replace('-', "_")))
    });
    let template = bridge::read_bridge_template(
        options.bridge.template.as_deref(),
        manifest_dir,
        &options.bridge.sdk_repo,
        options.bridge.sdk_version.as_deref(),
        options.bridge.refresh_sdk,
    )?;

    write_single_file_plugin(&template, &native_dll, &output)?;
    println!("Packaged B站弹幕姬 plugin in {}", output.display());

    Ok(())
}

#[derive(Args, Debug)]
struct PackageOptions {
    #[arg(long)]
    native: PathBuf,

    #[arg(short = 'o', long)]
    output: PathBuf,

    #[command(flatten)]
    bridge: BridgeOptions,
}

fn package(options: PackageOptions) -> Result<()> {
    let current_dir = env::current_dir().wrap_err("failed to read current directory")?;
    let template = bridge::read_bridge_template(
        options.bridge.template.as_deref(),
        &current_dir,
        &options.bridge.sdk_repo,
        options.bridge.sdk_version.as_deref(),
        options.bridge.refresh_sdk,
    )?;
    write_single_file_plugin(&template, &options.native, &options.output)?;
    println!("Packaged B站弹幕姬 plugin in {}", options.output.display());
    Ok(())
}

#[derive(Args, Debug)]
struct NewOptions {
    name: String,

    #[arg(long)]
    sdk_path: Option<PathBuf>,

    #[arg(long)]
    sdk_version: Option<String>,
}

fn create_new(options: NewOptions) -> Result<()> {
    let root = PathBuf::from(&options.name);
    if root.exists() {
        return Err(eyre!("path already exists: {}", root.display()));
    }

    let crate_name = options.name.replace('_', "-");
    let lib_name = crate_name.replace('-', "_");
    let struct_name = to_pascal_case(&crate_name);

    fs::create_dir_all(root.join("src"))
        .wrap_err_with(|| format!("failed to create {}", root.join("src").display()))?;

    let dependency = if let Some(path) = options.sdk_path {
        let path = path.display().to_string().replace('\\', "/");
        format!("danmuji-sdk = {{ path = \"{path}\" }}")
    } else {
        "danmuji-sdk = \"0.1\"".to_string()
    };

    fs::write(
        root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{crate_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\nname = \"{lib_name}\"\ncrate-type = [\"cdylib\"]\n\n[dependencies]\n{dependency}\n"
        ),
    )
    .wrap_err_with(|| format!("failed to write {}", root.join("Cargo.toml").display()))?;

    let lib_rs = root.join("src").join("lib.rs");
    fs::write(
        &lib_rs,
        format!(
            "use danmuji_sdk::{{Danmaku, DanmujiPlugin, Host, MsgType, PluginContext, PluginMetadata}};\n\n\
#[derive(Default)]\n\
struct {struct_name};\n\n\
impl DanmujiPlugin for {struct_name} {{\n    \
fn metadata(&self) -> PluginMetadata {{\n        \
PluginMetadata {{\n            \
name: \"{crate_name}\",\n            \
author: \"\",\n            \
contact: \"\",\n            \
version: \"v0.1.0\",\n            \
description: \"Rust plugin for B站弹幕姬\",\n        \
}}\n    \
}}\n\n    \
fn start(&mut self, host: Host, _ctx: PluginContext) {{\n        \
host.log(\"{crate_name} started\");\n    \
}}\n\n    \
fn danmaku(&mut self, host: Host, danmaku: Danmaku) {{\n        \
if danmaku.msg_type == MsgType::Comment {{\n            \
let user = danmaku.user_name.as_deref().unwrap_or(\"unknown\");\n            \
let text = danmaku.comment_text.as_deref().unwrap_or(\"\");\n            \
host.log(format!(\"{{user}}: {{text}}\"));\n        \
}}\n    \
}}\n\
}}\n\n\
danmuji_sdk::export_plugin!({struct_name}::default());\n"
        ),
    )
    .wrap_err_with(|| format!("failed to write {}", lib_rs.display()))?;

    if let Some(sdk_version) = options.sdk_version {
        bridge::write_version_file(&root, &sdk_version)?;
    }

    println!("Created B站弹幕姬 plugin project in {}", root.display());
    Ok(())
}

#[derive(Args, Debug)]
struct UpgradeOptions {
    #[command(flatten)]
    manifest: Manifest,

    #[arg(long, default_value_t = bridge::DEFAULT_SDK_REPO.to_string())]
    sdk_repo: String,
}

fn upgrade(options: UpgradeOptions) -> Result<()> {
    let project_dir = project_dir_from_manifest(options.manifest.manifest_path.as_deref())?;
    let upgrade = bridge::upgrade_version_file(&project_dir, &options.sdk_repo)?;

    match upgrade.old_version {
        Some(old) if old == upgrade.new_version => {
            println!(
                "{} is already at latest SDK {}",
                upgrade.path.display(),
                upgrade.new_version
            );
        }
        Some(old) => {
            println!(
                "Updated {} from {} to {}",
                upgrade.path.display(),
                old,
                upgrade.new_version
            );
        }
        None => {
            println!(
                "Created {} with latest SDK {}",
                upgrade.path.display(),
                upgrade.new_version
            );
        }
    }

    Ok(())
}

fn native_dll_path(
    target_dir: &Path,
    target: Option<&str>,
    profile: &str,
    lib_name: &str,
) -> PathBuf {
    let mut path = target_dir.to_path_buf();
    if let Some(target) = target {
        path.push(target);
    }

    path.push(profile);
    path.push(format!("{lib_name}.dll"));
    path
}

fn load_metadata(manifest_path: &Path) -> Result<Metadata> {
    let mut command = MetadataCommand::new();
    command.manifest_path(manifest_path).no_deps();
    command
        .exec()
        .wrap_err_with(|| format!("failed to inspect {}", manifest_path.display()))
}

fn select_package<'a>(
    metadata: &'a Metadata,
    manifest_path: &Path,
    package_name: Option<&str>,
) -> Result<&'a Package> {
    if let Some(package_name) = package_name {
        return metadata
            .packages
            .iter()
            .find(|package| package.name == package_name)
            .ok_or_else(|| eyre!("package `{package_name}` not found in workspace metadata"));
    }

    if let Some(package) = metadata
        .packages
        .iter()
        .find(|package| package.manifest_path.as_std_path() == manifest_path)
    {
        return Ok(package);
    }

    metadata
        .root_package()
        .ok_or_else(|| eyre!("package name is required; pass --package"))
}

fn cdylib_target_name(package: &Package) -> Result<String> {
    package
        .targets
        .iter()
        .find(|target| target.crate_types.iter().any(|kind| kind == "cdylib"))
        .map(|target| target.name.clone())
        .ok_or_else(|| {
            eyre!(
                "package `{}` has no cdylib target; add [lib] crate-type = [\"cdylib\"]",
                package.name
            )
        })
}

fn write_single_file_plugin(template: &[u8], native_dll: &Path, output: &Path) -> Result<()> {
    let native = fs::read(native_dll)
        .wrap_err_with(|| format!("failed to read native DLL {}", native_dll.display()))?;
    let hash = Sha256::digest(&native);

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("failed to create {}", parent.display()))?;
        }
    }

    let mut file = fs::File::create(output)
        .wrap_err_with(|| format!("failed to create {}", output.display()))?;
    file.write_all(template)
        .wrap_err_with(|| format!("failed to write bridge template to {}", output.display()))?;
    file.write_all(&native)
        .wrap_err_with(|| format!("failed to write native DLL to {}", output.display()))?;
    file.write_all(&hash)
        .wrap_err_with(|| format!("failed to write overlay hash to {}", output.display()))?;
    file.write_all(&(native.len() as u64).to_le_bytes())
        .wrap_err_with(|| format!("failed to write overlay length to {}", output.display()))?;
    file.write_all(&FOOTER_VERSION.to_le_bytes())
        .wrap_err_with(|| format!("failed to write overlay version to {}", output.display()))?;
    file.write_all(OVERLAY_MAGIC)
        .wrap_err_with(|| format!("failed to write overlay magic to {}", output.display()))?;
    file.flush()
        .wrap_err_with(|| format!("failed to flush {}", output.display()))?;

    Ok(())
}

fn run_command(mut command: Command) -> Result<()> {
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .wrap_err("failed to run cargo build")?;

    if !status.success() {
        return Err(eyre!("command failed with status {status}"));
    }

    Ok(())
}

fn cargo_binary() -> OsString {
    env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn project_dir_from_manifest(manifest_path: Option<&Path>) -> Result<PathBuf> {
    let manifest_path = manifest_path.unwrap_or_else(|| Path::new("Cargo.toml"));
    let manifest_path = absolute_path(manifest_path)?;
    manifest_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| eyre!("manifest path has no parent directory"))
}

fn to_pascal_case(name: &str) -> String {
    let mut output = String::new();
    let mut upper_next = true;

    for ch in name.chars() {
        if ch == '-' || ch == '_' || ch == ' ' {
            upper_next = true;
            continue;
        }

        if upper_next {
            output.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            output.push(ch);
        }
    }

    if output.is_empty() {
        "Plugin".to_string()
    } else {
        output
    }
}
