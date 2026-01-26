#![cfg(target_os = "macos")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use directories::BaseDirs;
use plist::Value;

use crate::{BoxError, find_qmpo_executable};

const APP_NAME: &str = "qmpo.app";
const BUNDLE_ID: &str = "com.github.qmpo";
const LSREGISTER_PATH: &str = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";

pub fn register(path: Option<PathBuf>) -> Result<(), BoxError> {
    let base_dirs = BaseDirs::new().ok_or("Could not determine user directories")?;
    let home_dir = base_dirs.home_dir();

    let qmpo_path = path.map_or_else(find_qmpo_executable, Ok)?;

    if !qmpo_path.exists() {
        return Err(format!("qmpo executable not found at: {}", qmpo_path.display()).into());
    }

    // Create app bundle at ~/Applications/qmpo.app
    let applications_dir = home_dir.join("Applications");
    fs::create_dir_all(&applications_dir)?;

    let app_bundle = applications_dir.join(APP_NAME);
    let contents_dir = app_bundle.join("Contents");
    let macos_dir = contents_dir.join("MacOS");

    fs::create_dir_all(&macos_dir)?;

    // Copy executable
    let installed_path = macos_dir.join("qmpo");
    if qmpo_path != installed_path {
        fs::copy(&qmpo_path, &installed_path)?;
        set_executable_permissions(&installed_path)?;
        println!("Installed qmpo to: {}", installed_path.display());
    }

    // Create Info.plist
    let info_plist_path = contents_dir.join("Info.plist");
    let info_plist = create_info_plist();
    let mut file = fs::File::create(&info_plist_path)?;
    info_plist.to_writer_xml(&mut file)?;
    println!("Created Info.plist: {}", info_plist_path.display());

    // Register with Launch Services
    let app_bundle_str = app_bundle.to_str().ok_or("Invalid app bundle path")?;

    let status = Command::new(LSREGISTER_PATH)
        .args(["-register", app_bundle_str])
        .status()?;

    if status.success() {
        println!("Registered qmpo as handler for directory:// URIs");
        Ok(())
    } else {
        Err("Failed to register with Launch Services".into())
    }
}

pub fn unregister() -> Result<(), BoxError> {
    let base_dirs = BaseDirs::new().ok_or("Could not determine user directories")?;
    let app_bundle = base_dirs.home_dir().join("Applications").join(APP_NAME);

    if app_bundle.exists() {
        // Unregister from Launch Services (ignore errors)
        if let Some(path_str) = app_bundle.to_str() {
            let _ = Command::new(LSREGISTER_PATH)
                .args(["-unregister", path_str])
                .status();
        }

        fs::remove_dir_all(&app_bundle)?;
        println!("Removed: {}", app_bundle.display());
    }

    println!("Unregistered qmpo");
    Ok(())
}

pub fn status() -> Result<(), BoxError> {
    let base_dirs = BaseDirs::new().ok_or("Could not determine user directories")?;

    let app_bundle = base_dirs.home_dir().join("Applications").join(APP_NAME);
    let executable = app_bundle.join("Contents/MacOS/qmpo");

    if executable.exists() {
        println!("qmpo app bundle: {} (installed)", app_bundle.display());
    } else {
        println!("qmpo app bundle: not installed");
    }

    // Check Launch Services registration
    let output = Command::new(LSREGISTER_PATH).args(["-dump"]).output()?;

    let dump = String::from_utf8_lossy(&output.stdout);
    if dump.contains(BUNDLE_ID) {
        println!("Launch Services: registered");
    } else {
        println!("Launch Services: not registered");
    }

    Ok(())
}

fn create_info_plist() -> Value {
    let mut dict = plist::Dictionary::new();

    dict.insert(
        "CFBundleIdentifier".to_string(),
        Value::String(BUNDLE_ID.to_string()),
    );
    dict.insert(
        "CFBundleName".to_string(),
        Value::String("qmpo".to_string()),
    );
    dict.insert(
        "CFBundleDisplayName".to_string(),
        Value::String("qmpo".to_string()),
    );
    dict.insert(
        "CFBundleExecutable".to_string(),
        Value::String("qmpo".to_string()),
    );
    dict.insert(
        "CFBundlePackageType".to_string(),
        Value::String("APPL".to_string()),
    );
    dict.insert(
        "CFBundleVersion".to_string(),
        Value::String("1.0".to_string()),
    );
    dict.insert(
        "CFBundleShortVersionString".to_string(),
        Value::String("1.0".to_string()),
    );
    dict.insert("LSBackgroundOnly".to_string(), Value::Boolean(true));

    // URL Types for directory:// scheme
    let mut url_type = plist::Dictionary::new();
    url_type.insert(
        "CFBundleURLName".to_string(),
        Value::String("Directory URL".to_string()),
    );
    url_type.insert(
        "CFBundleURLSchemes".to_string(),
        Value::Array(vec![Value::String("directory".to_string())]),
    );

    dict.insert(
        "CFBundleURLTypes".to_string(),
        Value::Array(vec![Value::Dictionary(url_type)]),
    );

    Value::Dictionary(dict)
}

fn set_executable_permissions(path: &std::path::Path) -> Result<(), BoxError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}
