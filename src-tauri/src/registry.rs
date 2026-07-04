use winreg::enums::*;
use winreg::RegKey;
use std::env;

const APP_NAME: &str = "BrowserRouter";
const APP_DESCRIPTION: &str = "Routes links between browsers";

pub fn register() -> Result<(), String> {
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let exe_path_str = exe_path.to_string_lossy().to_string();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let (prog_key, _) = hkcu
        .create_subkey(format!(r"Software\Classes\{}", APP_NAME))
        .map_err(|e| e.to_string())?;
    prog_key.set_value("", &APP_DESCRIPTION).map_err(|e| e.to_string())?;

    let (command_key, _) = prog_key
        .create_subkey(r"shell\open\command")
        .map_err(|e| e.to_string())?;
    let command_value = format!("\"{}\" \"%1\"", exe_path_str);
    command_key.set_value("", &command_value).map_err(|e| e.to_string())?;

    let (registered_apps, _) = hkcu
        .create_subkey(r"Software\RegisteredApplications")
        .map_err(|e| e.to_string())?;
    registered_apps
        .set_value(APP_NAME, &format!(r"Software\Classes\{}\Capabilities", APP_NAME))
        .map_err(|e| e.to_string())?;

    let (capabilities_key, _) = hkcu
        .create_subkey(format!(r"Software\Classes\{}\Capabilities", APP_NAME))
        .map_err(|e| e.to_string())?;
    capabilities_key.set_value("ApplicationName", &APP_NAME).map_err(|e| e.to_string())?;
    capabilities_key.set_value("ApplicationDescription", &APP_DESCRIPTION).map_err(|e| e.to_string())?;

    let (url_assoc_key, _) = capabilities_key
        .create_subkey("URLAssociations")
        .map_err(|e| e.to_string())?;
    url_assoc_key.set_value("http", &APP_NAME).map_err(|e| e.to_string())?;
    url_assoc_key.set_value("https", &APP_NAME).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn unregister() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.delete_subkey_all(format!(r"Software\Classes\{}", APP_NAME))
        .map_err(|e| e.to_string())?;

    if let Ok(registered_apps) = hkcu.open_subkey_with_flags(r"Software\RegisteredApplications", KEY_SET_VALUE) {
        let _ = registered_apps.delete_value(APP_NAME);
    }

    Ok(())
}