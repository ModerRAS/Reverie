use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_in_path(exe_name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(exe_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn cargo_bin_dir() -> Option<PathBuf> {
    let home = env::var("USERPROFILE").or_else(|_| env::var("HOME")).ok()?;
    Some(PathBuf::from(home).join(".cargo").join("bin"))
}

fn find_dx() -> Option<PathBuf> {
    #[cfg(windows)]
    const DX: &str = "dx.exe";
    #[cfg(not(windows))]
    const DX: &str = "dx";

    // First try PATH
    if let Some(p) = find_in_path(DX) {
        return Some(p);
    }

    // Fallback to cargo bin dir (common on Windows when PATH isn't refreshed)
    let candidate = cargo_bin_dir()?.join(DX);
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

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
    // Always bundle the release UI output, even for debug builds, to avoid dev hot-reload artifacts.
    let dx_profile = "release";

    // Build the UI with dioxus CLI
    let Some(dx) = find_dx() else {
        // Don't hard-fail compilation if dx is missing; server can still run without UI.
        println!(
            "cargo:warning=dx not found; skipping UI build. Install dioxus-cli or set REVERIE_SKIP_UI_BUILD=1."
        );
        return;
    };

    let mut cmd = Command::new(dx);
    cmd.current_dir(&workspace_dir)
        .arg("build")
        .arg("--package")
        .arg("reverie-ui");

    cmd.arg("--release");

    let status = cmd.status();
    let Ok(status) = status else {
        println!("cargo:warning=failed to execute dx; skipping UI build.");
        return;
    };

    let ui_src = workspace_dir
        .join("target")
        .join("dx")
        .join("reverie-ui")
        .join(dx_profile)
        .join("web")
        .join("public");

    // Some Windows setups may report a non-zero exit code from wasm-opt even when the bundle exists.
    // Prefer checking the actual output over the dx process exit code.
    if !status.success() && !ui_src.join("index.html").exists() {
        println!(
            "cargo:warning=dx build failed; UI will not be bundled into target/{}/ui.",
            profile
        );
    }

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
    }
}
