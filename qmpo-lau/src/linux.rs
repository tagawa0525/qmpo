#![cfg(target_os = "linux")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use directories::BaseDirs;

use crate::{BoxError, find_qmpo_executable};

const DESKTOP_FILE_NAME: &str = "qmpo.desktop";
const MIME_TYPE: &str = "x-scheme-handler/directory";

pub fn register(path: Option<PathBuf>) -> Result<(), BoxError> {
    let base_dirs = BaseDirs::new().ok_or("Could not determine user directories")?;
    let home_dir = base_dirs.home_dir();

    let qmpo_path = path.map_or_else(find_qmpo_executable, Ok)?;

    if !qmpo_path.exists() {
        return Err(format!("qmpo executable not found at: {}", qmpo_path.display()).into());
    }

    // Install qmpo to ~/.local/bin/
    let local_bin = home_dir.join(".local/bin");
    fs::create_dir_all(&local_bin)?;

    let installed_path = local_bin.join("qmpo");
    if qmpo_path != installed_path {
        fs::copy(&qmpo_path, &installed_path)?;
        set_executable_permissions(&installed_path)?;
        println!("Installed qmpo to: {}", installed_path.display());
    }

    // Create desktop file
    let applications_dir = home_dir.join(".local/share/applications");
    fs::create_dir_all(&applications_dir)?;

    let desktop_file_path = applications_dir.join(DESKTOP_FILE_NAME);
    let desktop_content = format!(
        r#"[Desktop Entry]
Type=Application
Name=qmpo
Comment=Directory URI Handler
Exec="{}" %u
Terminal=false
NoDisplay=true
MimeType={MIME_TYPE};
"#,
        installed_path.display()
    );

    fs::write(&desktop_file_path, desktop_content)?;
    println!("Created desktop file: {}", desktop_file_path.display());

    // Update desktop database (ignore errors)
    let _ = Command::new("update-desktop-database")
        .arg(&applications_dir)
        .status();

    // Set as default handler
    let status = Command::new("xdg-mime")
        .args(["default", DESKTOP_FILE_NAME, MIME_TYPE])
        .status()?;

    if status.success() {
        println!("Registered qmpo as handler for directory:// URIs");
        Ok(())
    } else {
        Err("Failed to set default MIME handler".into())
    }
}

pub fn unregister() -> Result<(), BoxError> {
    let base_dirs = BaseDirs::new().ok_or("Could not determine user directories")?;
    let home_dir = base_dirs.home_dir();

    // Remove desktop file
    let desktop_file_path = home_dir
        .join(".local/share/applications")
        .join(DESKTOP_FILE_NAME);

    if desktop_file_path.exists() {
        fs::remove_file(&desktop_file_path)?;
        println!("Removed desktop file: {}", desktop_file_path.display());
    }

    // Update desktop database (ignore errors)
    let applications_dir = home_dir.join(".local/share/applications");
    let _ = Command::new("update-desktop-database")
        .arg(&applications_dir)
        .status();

    // Remove installed binary
    let installed_path = home_dir.join(".local/bin/qmpo");
    if installed_path.exists() {
        fs::remove_file(&installed_path)?;
        println!("Removed: {}", installed_path.display());
    }

    println!("Unregistered qmpo");
    Ok(())
}

pub fn status() -> Result<(), BoxError> {
    let base_dirs = BaseDirs::new().ok_or("Could not determine user directories")?;
    let home_dir = base_dirs.home_dir();

    // Check installed binary
    let installed_path = home_dir.join(".local/bin/qmpo");
    if installed_path.exists() {
        println!("qmpo binary: {} (installed)", installed_path.display());
    } else {
        println!("qmpo binary: not installed");
    }

    // Check desktop file
    let desktop_file_path = home_dir
        .join(".local/share/applications")
        .join(DESKTOP_FILE_NAME);

    if desktop_file_path.exists() {
        println!("Desktop file: {} (exists)", desktop_file_path.display());
    } else {
        println!("Desktop file: not found");
    }

    // Check MIME handler
    let output = Command::new("xdg-mime")
        .args(["query", "default", MIME_TYPE])
        .output()?;

    let handler = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if handler == DESKTOP_FILE_NAME {
        println!("MIME handler: registered");
    } else if handler.is_empty() {
        println!("MIME handler: not set");
    } else {
        println!("MIME handler: {handler} (different handler)");
    }

    Ok(())
}

fn set_executable_permissions(path: &std::path::Path) -> Result<(), BoxError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}
