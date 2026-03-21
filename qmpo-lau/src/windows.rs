#![cfg(target_os = "windows")]

use std::fs;
use std::io;
use std::path::PathBuf;

use winreg::RegKey;
use winreg::enums::*;

use crate::{LauError, Result, check_install_permissions, find_qmpo_executable};

const PROTOCOL_NAME: &str = "directory";
const ADMIN_HINT: &str = "run as Administrator, or use the install script (scripts\\install.ps1)";

/// Convert a winreg `io::Error` into the appropriate `LauError`.
/// Returns `PermissionDenied` with an admin hint for access-denied errors,
/// and a generic `Registry` error for everything else.
fn registry_error(e: io::Error, operation: &str) -> LauError {
    if e.kind() == io::ErrorKind::PermissionDenied {
        LauError::PermissionDenied {
            operation: operation.to_string(),
            hint: ADMIN_HINT.to_string(),
        }
    } else {
        LauError::Registry(format!("{operation}: {e}"))
    }
}

/// Returns the machine-wide install directory (`%PROGRAMFILES%\qmpo`).
fn install_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(
        std::env::var("PROGRAMFILES").map_err(|_| LauError::EnvVarNotSet("PROGRAMFILES".into()))?,
    )
    .join("qmpo"))
}

/// Find an existing policy value whose JSON contains `"protocol":"directory"`.
/// Returns the value name (e.g. "1", "2") if found.
#[allow(clippy::collapsible_if)]
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

/// Browser policy registry paths for AutoLaunchProtocolsFromOrigins.
/// Each entry is `(display_name, registry_path)`.
const BROWSER_POLICY_PATHS: [(&str, &str); 2] = [
    (
        "Edge",
        r"Software\Policies\Microsoft\Edge\AutoLaunchProtocolsFromOrigins",
    ),
    (
        "Chrome",
        r"Software\Policies\Google\Chrome\AutoLaunchProtocolsFromOrigins",
    ),
];

pub fn register(path: Option<PathBuf>) -> Result<()> {
    let qmpo_path = path.map_or_else(find_qmpo_executable, Ok)?;

    if !qmpo_path.exists() {
        return Err(LauError::ExecutableNotFound(
            qmpo_path.display().to_string(),
        ));
    }

    // Install qmpo to %PROGRAMFILES%\qmpo\
    let install_dir = install_dir()?;
    check_install_permissions(&install_dir, ADMIN_HINT)?;

    let installed_path = install_dir.join("qmpo.exe");
    if qmpo_path != installed_path {
        fs::copy(&qmpo_path, &installed_path)?;
        println!("Installed qmpo to: {}", installed_path.display());
    }

    // Register protocol handler in HKLM (machine-wide).
    // HKLM\Software\Classes is the machine-wide equivalent of per-user
    // HKCU\Software\Classes. All users on this machine will get the handler.
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let classes = hklm
        .open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        .map_err(|e| registry_error(e, "opening HKLM\\Software\\Classes"))?;

    // Create directory protocol key
    let (protocol_key, _) = classes
        .create_subkey(PROTOCOL_NAME)
        .map_err(|e| registry_error(e, "creating protocol key in HKLM"))?;
    protocol_key
        .set_value("", &"URL:Directory Protocol")
        .map_err(|e| registry_error(e, "writing protocol description to HKLM"))?;
    protocol_key
        .set_value("URL Protocol", &"")
        .map_err(|e| registry_error(e, "writing URL Protocol marker to HKLM"))?;

    // Create shell\open\command key
    let (shell_key, _) = protocol_key
        .create_subkey("shell")
        .map_err(|e| registry_error(e, "creating shell key in HKLM"))?;
    let (open_key, _) = shell_key
        .create_subkey("open")
        .map_err(|e| registry_error(e, "creating open key in HKLM"))?;
    let (command_key, _) = open_key
        .create_subkey("command")
        .map_err(|e| registry_error(e, "creating command key in HKLM"))?;

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
        .map_err(|e| registry_error(e, "writing command to HKLM"))?;

    // Set browser policies in HKLM to suppress protocol launch confirmation dialog.
    // HKLM policies are treated as managed/enforced by Chrome and Edge, meaning they
    // cannot be overridden by user-level settings.
    let policy_value = r#"{"protocol":"directory","allowed_origins":["*"]}"#;

    for (_, browser_path) in BROWSER_POLICY_PATHS {
        let (policy_key, _) = hklm
            .create_subkey(browser_path)
            .map_err(|e| registry_error(e, &format!("creating HKLM\\{browser_path}")))?;

        // Find a free slot or reuse an existing "directory" entry to avoid
        // overwriting values set by other applications.
        let existing_name = find_directory_policy_entry(&policy_key);
        let value_name = existing_name.unwrap_or_else(|| next_policy_slot(&policy_key));

        policy_key
            .set_value(&value_name, &policy_value)
            .map_err(|e| registry_error(e, &format!("writing policy to HKLM\\{browser_path}")))?;
    }

    // Clean up legacy HKCU entries from previous versions
    clean_legacy_hkcu();

    println!("Registered qmpo as handler for directory:// URIs");
    Ok(())
}

/// Remove legacy HKCU protocol handler and browser policy entries left by
/// previous versions. Errors are silently ignored since these are best-effort.
fn clean_legacy_hkcu() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(classes) = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE)
        && classes.open_subkey(PROTOCOL_NAME).is_ok()
    {
        let _ = classes.delete_subkey_all(PROTOCOL_NAME);
    }
    for (_, browser_path) in BROWSER_POLICY_PATHS {
        if let Ok(policy_key) = hkcu.open_subkey_with_flags(browser_path, KEY_READ | KEY_WRITE)
            && let Some(name) = find_directory_policy_entry(&policy_key)
        {
            let _ = policy_key.delete_value(&name);
        }
    }
}

#[allow(clippy::collapsible_if)]
pub fn unregister() -> Result<()> {
    // Remove HKLM registry keys (current location)
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey_with_flags("Software\\Classes", KEY_WRITE) {
        Ok(classes) => match classes.delete_subkey_all(PROTOCOL_NAME) {
            Ok(()) => println!("Removed HKLM registry entries"),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                return Err(registry_error(e, "removing HKLM registry entries"));
            }
            Err(e) => eprintln!("Warning: failed to remove HKLM registry entries: {e}"),
        },
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            return Err(registry_error(e, "opening HKLM\\Software\\Classes"));
        }
        Err(_) => {}
    }

    // Remove only our "directory" protocol entry from HKLM browser policy keys,
    // leaving entries set by other applications intact.
    for (_, browser_path) in BROWSER_POLICY_PATHS {
        match hklm.open_subkey_with_flags(browser_path, KEY_READ | KEY_WRITE) {
            Ok(policy_key) => {
                if let Some(name) = find_directory_policy_entry(&policy_key) {
                    if let Err(e) = policy_key.delete_value(&name) {
                        if e.kind() == io::ErrorKind::PermissionDenied {
                            return Err(registry_error(
                                e,
                                &format!("removing policy from HKLM\\{browser_path}"),
                            ));
                        }
                        eprintln!(
                            "Warning: failed to remove policy from HKLM\\{browser_path}: {e}"
                        );
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                return Err(registry_error(e, &format!("opening HKLM\\{browser_path}")));
            }
            Err(_) => {}
        }
    }

    // Clean up legacy HKCU entries from previous versions
    clean_legacy_hkcu();

    // Remove installed binary (current location)
    if let Ok(install_dir) = install_dir() {
        if install_dir.exists() {
            let _ = fs::remove_dir_all(&install_dir);
            println!("Removed: {}", install_dir.display());
        }
    }

    // Clean up legacy install location (%LOCALAPPDATA%\qmpo)
    if let Some(base_dirs) = directories::BaseDirs::new() {
        let legacy_dir = base_dirs.data_local_dir().join("qmpo");
        let legacy_exe = legacy_dir.join("qmpo.exe");
        if legacy_exe.exists() {
            let _ = fs::remove_dir_all(&legacy_dir);
            println!("Removed legacy install: {}", legacy_dir.display());
        }
    }

    println!("Unregistered qmpo");
    Ok(())
}

pub fn status() -> Result<()> {
    // Check installed binary
    match install_dir() {
        Ok(dir) => {
            let installed_path = dir.join("qmpo.exe");
            if installed_path.exists() {
                println!("qmpo binary: {} (installed)", installed_path.display());
            } else {
                println!("qmpo binary: not installed");
            }
        }
        Err(_) => {
            println!("qmpo binary: not installed (PROGRAMFILES not set)");
        }
    }

    // Check registry — protocol handler (HKLM)
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let protocol_path = format!("Software\\Classes\\{PROTOCOL_NAME}");

    match hklm.open_subkey(&protocol_path) {
        Ok(protocol_key) => {
            let description: std::result::Result<String, _> = protocol_key.get_value("");
            println!(
                "Protocol key (HKLM): {}",
                description.as_deref().unwrap_or("(no description)")
            );

            let has_url_protocol = protocol_key.get_value::<String, _>("URL Protocol").is_ok();
            println!(
                "URL Protocol marker: {}",
                if has_url_protocol { "set" } else { "missing" }
            );

            let command_path = format!("{protocol_path}\\shell\\open\\command");
            match hklm.open_subkey(&command_path) {
                Ok(cmd_key) => {
                    let command: std::result::Result<String, _> = cmd_key.get_value("");
                    if let Ok(cmd) = command {
                        println!("Command: {cmd}");
                    } else {
                        println!("Command: (not set)");
                    }
                }
                Err(_) => {
                    println!("Command: (not set)");
                }
            }
        }
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!("Warning: cannot read HKLM registry (access denied)");
        }
        Err(_) => {
            // Fall back to checking HKCU for legacy registrations
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            match hkcu.open_subkey(&protocol_path) {
                Ok(_) => {
                    println!(
                        "Protocol key: registered in HKCU (legacy — run `qmpo-lau register` to migrate to HKLM)"
                    );
                }
                Err(_) => {
                    println!("Protocol key: not registered");
                }
            }
        }
    }

    // Check browser policies (HKLM)
    for (name, browser_path) in BROWSER_POLICY_PATHS {
        match hklm.open_subkey(browser_path) {
            Ok(key) => {
                if let Some(entry_name) = find_directory_policy_entry(&key) {
                    let value: std::result::Result<String, _> = key.get_value(&entry_name);
                    if let Ok(v) = value {
                        println!("{name} auto-launch policy (HKLM): {v}");
                    } else {
                        println!("{name} auto-launch policy (HKLM): entry exists (no value)");
                    }
                } else {
                    println!("{name} auto-launch policy: not set");
                }
            }
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                eprintln!("Warning: cannot read HKLM {name} policy (access denied)");
            }
            Err(_) => {
                // Fall back to checking HKCU for legacy policy entries
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                match hkcu.open_subkey(browser_path) {
                    Ok(key) => {
                        if find_directory_policy_entry(&key).is_some() {
                            println!(
                                "{name} auto-launch policy: set in HKCU (legacy — run `qmpo-lau register` to migrate to HKLM)"
                            );
                        } else {
                            println!("{name} auto-launch policy: not set");
                        }
                    }
                    Err(_) => {
                        println!("{name} auto-launch policy: not set");
                    }
                }
            }
        }
    }

    Ok(())
}
