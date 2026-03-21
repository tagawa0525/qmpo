#![cfg(target_os = "windows")]

use std::fs;
use std::path::PathBuf;

use directories::BaseDirs;
use winreg::RegKey;
use winreg::enums::*;

use crate::{LauError, Result, find_qmpo_executable};

const PROTOCOL_NAME: &str = "directory";

/// Find an existing policy value whose JSON contains `"protocol":"directory"`.
/// Returns the value name (e.g. "1", "2") if found.
fn find_directory_policy_entry(key: &RegKey) -> Option<String> {
    for entry in key.enum_values().flatten() {
        let value_name = entry.0;
        if let Ok(s) = key.get_value::<String, _>(&value_name) {
            if s.contains(r#""protocol":"directory""#) {
                return Some(value_name);
            }
        }
    }
    None
}

/// Find the next available numbered slot (e.g. "1", "2", "3") in a policy key.
fn next_policy_slot(key: &RegKey) -> String {
    let mut max: u32 = 0;
    for name in key.enum_values().flatten() {
        if let Ok(n) = name.0.parse::<u32>() {
            max = max.max(n);
        }
    }
    (max + 1).to_string()
}

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

    // Set browser policies to suppress protocol launch confirmation dialog.
    // AutoLaunchProtocolsFromOrigins allows the directory:// protocol to launch
    // without the "{server} wants to open this application" prompt.
    let policy_value =
        r#"{"protocol":"directory","allowed_origins":["*"]}"#;

    for browser_path in [
        r"Software\Policies\Microsoft\Edge\AutoLaunchProtocolsFromOrigins",
        r"Software\Policies\Google\Chrome\AutoLaunchProtocolsFromOrigins",
    ] {
        let (policy_key, _) = hkcu
            .create_subkey(browser_path)
            .map_err(|e| LauError::Registry(e.to_string()))?;

        // Find a free slot or reuse an existing "directory" entry to avoid
        // overwriting values set by other applications.
        let existing_name = find_directory_policy_entry(&policy_key);
        let value_name = existing_name.unwrap_or_else(|| next_policy_slot(&policy_key));

        policy_key
            .set_value(&value_name, &policy_value)
            .map_err(|e| LauError::Registry(e.to_string()))?;
    }

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

    // Remove only our "directory" protocol entry from browser policy keys,
    // leaving entries set by other applications intact.
    for browser_path in [
        r"Software\Policies\Microsoft\Edge\AutoLaunchProtocolsFromOrigins",
        r"Software\Policies\Google\Chrome\AutoLaunchProtocolsFromOrigins",
    ] {
        if let Ok(policy_key) =
            hkcu.open_subkey_with_flags(browser_path, KEY_READ | KEY_WRITE)
        {
            if let Some(name) = find_directory_policy_entry(&policy_key) {
                let _ = policy_key.delete_value(&name);
            }
        }
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

    // Check browser policies
    for (name, browser_path) in [
        ("Edge", r"Software\Policies\Microsoft\Edge\AutoLaunchProtocolsFromOrigins"),
        ("Chrome", r"Software\Policies\Google\Chrome\AutoLaunchProtocolsFromOrigins"),
    ] {
        match hkcu.open_subkey(browser_path) {
            Ok(key) => {
                if let Some(entry_name) = find_directory_policy_entry(&key) {
                    let value: std::result::Result<String, _> =
                        key.get_value(&entry_name);
                    if let Ok(v) = value {
                        println!("{name} auto-launch policy: {v}");
                    } else {
                        println!("{name} auto-launch policy: entry exists (no value)");
                    }
                } else {
                    println!("{name} auto-launch policy: not set");
                }
            }
            Err(_) => {
                println!("{name} auto-launch policy: not set");
            }
        }
    }

    Ok(())
}
