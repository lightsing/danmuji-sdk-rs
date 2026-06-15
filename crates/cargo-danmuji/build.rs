use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=DANMUJI_BRIDGE_TEMPLATE");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let output = out_dir.join("DanmujiRustBridge.dll");

    if let Some(template) = env::var_os("DANMUJI_BRIDGE_TEMPLATE") {
        let template = PathBuf::from(template);
        println!("cargo:rerun-if-changed={}", template.display());
        copy_template(&template, &output);
        return;
    }

    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("cargo-danmuji lives under crates/cargo-danmuji");
    let bridge_project = repo_root
        .join("bridge")
        .join("DanmujiRustBridge")
        .join("DanmujiRustBridge.csproj");

    if !bridge_project.exists() {
        let packaged_template = manifest_dir.join("assets").join("DanmujiRustBridge.dll");
        println!("cargo:rerun-if-changed={}", packaged_template.display());
        copy_template(&packaged_template, &output);
        return;
    }

    let bridge_dir = bridge_project
        .parent()
        .expect("bridge project has a parent");

    println!("cargo:rerun-if-changed={}", bridge_project.display());
    println!(
        "cargo:rerun-if-changed={}",
        bridge_dir.join("RustNative.cs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        bridge_dir.join("RustPlugin.cs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        bridge_dir.join("FfiScope.cs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        repo_root
            .join("vendor")
            .join("BilibiliDM_PluginFramework.dll")
            .display()
    );

    let status = Command::new("dotnet")
        .arg("build")
        .arg(&bridge_project)
        .arg("-c")
        .arg("Release")
        .status()
        .expect("failed to run dotnet build for DanmujiRustBridge");

    if !status.success() {
        panic!("dotnet build failed for {}", bridge_project.display());
    }

    let built_template = bridge_dir
        .join("bin")
        .join("Release")
        .join("net461")
        .join("DanmujiRustBridge.dll");
    copy_template(&built_template, &output);
    sync_template_asset(&manifest_dir, &built_template);
}

fn copy_template(source: &Path, output: &Path) {
    if !source.exists() {
        panic!("bridge template not found: {}", source.display());
    }

    fs::create_dir_all(output.parent().expect("output has a parent"))
        .expect("failed to create OUT_DIR");
    copy_if_changed(source, output);
}

fn sync_template_asset(manifest_dir: &Path, source: &Path) {
    let output = manifest_dir.join("assets").join("DanmujiRustBridge.dll");
    fs::create_dir_all(output.parent().expect("asset output has a parent"))
        .expect("failed to create cargo-danmuji assets directory");
    copy_if_changed(source, &output);
}

fn copy_if_changed(source: &Path, output: &Path) {
    let bytes = fs::read(source).expect("failed to read bridge template");
    if fs::read(output)
        .map(|existing| existing == bytes)
        .unwrap_or(false)
    {
        return;
    }

    fs::write(output, bytes).expect("failed to write bridge template");
}
