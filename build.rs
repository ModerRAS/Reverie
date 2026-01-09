use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn copy_dir_recursive(from: &Path, to: &Path) -> io::Result<()> {
    if !to.exists() {
        fs::create_dir_all(to)?;
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src = entry.path();
        let dst = to.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src, &dst)?;
        } else if file_type.is_file() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dst)?;
        }
    }

    Ok(())
}

fn main() {
    // Allow opting out (e.g. minimal CI builds)
    if env::var("REVERIE_SKIP_UI_BUILD").ok().as_deref() == Some("1") {
        return;
    }

    let workspace_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));

    // Re-run if UI sources or assets change
    println!("cargo:rerun-if-changed=reverie-ui/src");
    println!("cargo:rerun-if-changed=reverie-ui/assets");
    println!("cargo:rerun-if-changed=reverie-ui/Dioxus.toml");
    println!("cargo:rerun-if-changed=reverie-ui/Cargo.toml");

    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let dx_profile = if profile == "release" {
        "release"
    } else {
        "debug"
    };

    // Build the UI with dioxus CLI
    let mut cmd = Command::new("dx");
    cmd.current_dir(&workspace_dir)
        .arg("build")
        .arg("--package")
        .arg("reverie-ui");

    if profile == "release" {
        cmd.arg("--release");
    }

    let status = cmd.status();
    let Ok(status) = status else {
        // Don't hard-fail compilation if dx is missing; server can still run without UI.
        println!("cargo:warning=dx not found; skipping UI build. Install dioxus-cli or set REVERIE_SKIP_UI_BUILD=1.");
        return;
    };

    if !status.success() {
        println!(
            "cargo:warning=dx build failed; UI will not be bundled into target/{}/ui.",
            profile
        );
        return;
    }

    let ui_src = workspace_dir
        .join("target")
        .join("dx")
        .join("reverie-ui")
        .join(dx_profile)
        .join("web")
        .join("public");

    if !ui_src.join("index.html").exists() {
        println!("cargo:warning=dx build output not found at {:?}", ui_src);
        return;
    }

    let ui_out = workspace_dir.join("target").join(&profile).join("ui");

    // Replace output dir
    let _ = fs::remove_dir_all(&ui_out);
    if let Err(e) = fs::create_dir_all(&ui_out) {
        println!(
            "cargo:warning=failed to create ui output dir {:?}: {}",
            ui_out, e
        );
        return;
    }

    if let Err(e) = copy_dir_recursive(&ui_src, &ui_out) {
        println!(
            "cargo:warning=failed to copy ui from {:?} to {:?}: {}",
            ui_src, ui_out, e
        );
        return;
    }
}
