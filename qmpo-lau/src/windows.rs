#![cfg(target_os = "windows")]

use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;
use winreg::RegKey;
use winreg::enums::*;

use crate::{LauError, Result, find_qmpo_executable};

const PROTOCOL_NAME: &str = "directory";

pub fn register(path: Option<PathBuf>) -> Result<()> {
    let base_dirs = BaseDirs::new().ok_or(LauError::NoUserDirectories)?;

    let qmpo_path = path.map_or_else(find_qmpo_executable, Ok)?;

    if !qmpo_path.exists() {
        return Err(LauError::ExecutableNotFound(
            qmpo_path.display().to_string(),
        ));
    }

    // Install qmpo to %LOCALAPPDATA%\qmpo\
    let install_dir = base_dirs.data_local_dir().join("qmpo");
    fs::create_dir_all(&install_dir)?;

    let installed_path = install_dir.join("qmpo.exe");
    if qmpo_path != installed_path {
        fs::copy(&qmpo_path, &installed_path)?;
        println!("Installed qmpo to: {}", installed_path.display());
    }

    // Create registry keys
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        .map_err(|e| LauError::Registry(e.to_string()))?;

    // Create directory protocol key
    let (protocol_key, _) = classes
        .create_subkey(PROTOCOL_NAME)
        .map_err(|e| LauError::Registry(e.to_string()))?;
    protocol_key
        .set_value("", &"URL:Directory Protocol")
        .map_err(|e| LauError::Registry(e.to_string()))?;
    protocol_key
        .set_value("URL Protocol", &"")
        .map_err(|e| LauError::Registry(e.to_string()))?;

    // Create shell\open\command key
    let (shell_key, _) = protocol_key
        .create_subkey("shell")
        .map_err(|e| LauError::Registry(e.to_string()))?;
    let (open_key, _) = shell_key
        .create_subkey("open")
        .map_err(|e| LauError::Registry(e.to_string()))?;
    let (command_key, _) = open_key
        .create_subkey("command")
        .map_err(|e| LauError::Registry(e.to_string()))?;

    // Validate path doesn't contain characters that could break the command
    let path_str = installed_path
        .to_str()
        .ok_or_else(|| LauError::InvalidPath("contains invalid Unicode characters".into()))?;
    if path_str.contains('"') {
        return Err(LauError::InvalidPath("contains double quote".into()));
    }

    let command = format!("\"{path_str}\" \"%1\"");
    command_key
        .set_value("", &command)
        .map_err(|e| LauError::Registry(e.to_string()))?;

    println!("Registered qmpo as handler for directory:// URIs");
    Ok(())
}

pub fn unregister() -> Result<()> {
    let base_dirs = BaseDirs::new().ok_or(LauError::NoUserDirectories)?;

    // Remove registry keys
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(classes) = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE) {
        let _ = classes.delete_subkey_all(PROTOCOL_NAME);
        println!("Removed registry entries");
    }

    // Remove installed binary
    let install_dir = base_dirs.data_local_dir().join("qmpo");
    if install_dir.exists() {
        let _ = fs::remove_dir_all(&install_dir);
        println!("Removed: {}", install_dir.display());
    }

    println!("Unregistered qmpo");
    Ok(())
}

pub fn status() -> Result<()> {
    let base_dirs = BaseDirs::new().ok_or(LauError::NoUserDirectories)?;

    // Check installed binary
    let installed_path = base_dirs.data_local_dir().join("qmpo").join("qmpo.exe");
    if installed_path.exists() {
        println!("qmpo binary: {} (installed)", installed_path.display());
    } else {
        println!("qmpo binary: not installed");
    }

    // Check registry
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let registry_path = format!("Software\\Classes\\{PROTOCOL_NAME}\\shell\\open\\command");

    match hkcu.open_subkey(&registry_path) {
        Ok(key) => {
            let command: std::result::Result<String, _> = key.get_value("");
            if let Ok(cmd) = command {
                println!("Registry: registered");
                println!("Command: {cmd}");
            } else {
                println!("Registry: registered (no command)");
            }
        }
        Err(_) => {
            println!("Registry: not registered");
        }
    }

    Ok(())
}
