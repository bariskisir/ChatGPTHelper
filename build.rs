// Builds frontend assets before Tauri packages the desktop application.
use std::path::Path;
use std::process::Command;
use std::{fs, io, time::SystemTime};

/// Ensures `cargo run` can compile even when the Tauri CLI did not run
/// `beforeBuildCommand` first.
fn main() {
    ensure_frontend_dist();
    tauri_build::build();
}

/// Ensures packaged frontend assets exist before Rust compilation continues.
fn ensure_frontend_dist() {
    println!("cargo:rerun-if-changed=frontend/index.html");
    println!("cargo:rerun-if-changed=frontend/styles.css");
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=icons/icon.png");

    let dist = Path::new("frontend").join("dist");
    let dist_app = dist.join("app.js");
    let dist_index = dist.join("index.html");
    let dist_styles = dist.join("styles.css");
    let dist_icon = dist.join("icon.png");

    if dist_app.exists()
        && dist_index.exists()
        && dist_styles.exists()
        && dist_icon.exists()
        && !frontend_dist_is_stale(&dist)
    {
        return;
    }

    if Path::new("frontend").join("node_modules").exists() {
        build_frontend();
        if dist_app.exists() && dist_index.exists() && dist_styles.exists() && dist_icon.exists() {
            return;
        }
    }

    if dist_app.exists() && dist_index.exists() && dist_styles.exists() && dist_icon.exists() {
        return;
    }

    fs::create_dir_all(&dist).expect("Could not create frontend/dist");
    copy_asset("frontend/index.html", &dist_index);
    copy_asset("frontend/styles.css", &dist_styles);
    copy_asset("icons/icon.png", &dist_icon);

    if !dist_app.exists() {
        fs::write(
            &dist_app,
            "console.warn('Run npm install && npm run build in frontend.');",
        )
        .expect("Could not create placeholder frontend/dist/app.js");
    }
}

/// Checks whether generated frontend assets are older than their sources.
fn frontend_dist_is_stale(dist: &Path) -> bool {
    let dist_app = dist.join("app.js");
    let Ok(dist_modified) = modified_at(&dist_app) else {
        return true;
    };

    let sources = [
        Path::new("frontend").join("index.html"),
        Path::new("frontend").join("styles.css"),
        Path::new("frontend").join("tsconfig.json"),
        Path::new("frontend").join("package.json"),
        Path::new("icons").join("icon.png"),
    ];

    sources
        .iter()
        .any(|path| modified_at(path).is_ok_and(|modified| modified > dist_modified))
        || directory_newer_than(Path::new("frontend").join("src").as_path(), dist_modified)
            .unwrap_or(true)
}

/// Walks a directory tree to find files newer than the supplied timestamp.
fn directory_newer_than(path: &Path, timestamp: SystemTime) -> io::Result<bool> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            if directory_newer_than(&entry.path(), timestamp)? {
                return Ok(true);
            }
        } else if metadata.modified()? > timestamp {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Reads the last modification time for a filesystem path.
fn modified_at(path: &Path) -> io::Result<SystemTime> {
    fs::metadata(path)?.modified()
}

/// Runs the frontend build script with the local npm executable.
fn build_frontend() {
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let status = Command::new(npm)
        .args(["run", "build"])
        .current_dir("frontend")
        .status()
        .expect("Could not start frontend build. Run `npm install` in frontend.");

    if !status.success() {
        panic!("Frontend build failed. Run `npm install` and `npm run build` in frontend.");
    }
}

/// Copies a required static asset into the frontend distribution directory.
fn copy_asset(source: &str, destination: &Path) {
    fs::copy(source, destination).unwrap_or_else(|error| {
        panic!(
            "Could not copy {} to {}: {}. Run `npm install` and `npm run build` in frontend.",
            source,
            destination.display(),
            error
        )
    });
}
