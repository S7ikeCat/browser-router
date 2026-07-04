use crate::config::{self, AppConfig, Rule};
use crate::registry;
use crate::get_current_ip;

#[tauri::command]
pub fn get_config() -> AppConfig {
    config::load_config()
}

#[tauri::command]
pub fn save_config(new_config: AppConfig) -> Result<(), String> {
    config::save_config(&new_config);
    Ok(())
}

#[tauri::command]
pub fn add_rule(
    pattern: String,
    browser_path: String,
    label: String,
    vpn_protected: bool,
) -> Result<AppConfig, String> {
    let mut cfg = config::load_config();
    cfg.rules.push(Rule {
        pattern,
        browser_path,
        label,
        vpn_protected,
    });
    config::save_config(&cfg);
    Ok(cfg)
}

#[tauri::command]
pub fn remove_rule(index: usize) -> Result<AppConfig, String> {
    let mut cfg = config::load_config();
    if index < cfg.rules.len() {
        cfg.rules.remove(index);
        config::save_config(&cfg);
    }
    Ok(cfg)
}

#[tauri::command]
pub fn register_browser() -> Result<String, String> {
    registry::register().map_err(|e| e.to_string())?;
    Ok("Успешно зарегистрировано!".to_string())
}

#[tauri::command]
pub fn unregister_browser() -> Result<String, String> {
    registry::unregister().map_err(|e| e.to_string())?;
    Ok("Регистрация удалена".to_string())
}

// Узнать свой текущий IP по кнопке в GUI (просто показать пользователю)
#[tauri::command]
pub fn get_current_ip_command() -> Result<String, String> {
    get_current_ip().ok_or_else(|| "Не удалось определить IP (проверь интернет-соединение)".to_string())
}

// Добавить текущий IP в список доверенных одним кликом
#[tauri::command]
pub fn add_trusted_ip() -> Result<AppConfig, String> {
    let ip = get_current_ip().ok_or_else(|| "Не удалось определить IP".to_string())?;
    let mut cfg = config::load_config();
    if !cfg.trusted_ips.contains(&ip) {
        cfg.trusted_ips.push(ip);
        config::save_config(&cfg);
    }
    Ok(cfg)
}

#[tauri::command]
pub fn remove_trusted_ip(index: usize) -> Result<AppConfig, String> {
    let mut cfg = config::load_config();
    if index < cfg.trusted_ips.len() {
        cfg.trusted_ips.remove(index);
        config::save_config(&cfg);
    }
    Ok(cfg)
}

#[tauri::command]
pub fn get_installed_browsers() -> Vec<registry::BrowserInfo> {
    registry::get_installed_browsers()
}

#[tauri::command]
pub fn toggle_vpn_protection(index: usize) -> Result<AppConfig, String> {
    let mut cfg = config::load_config();
    if index < cfg.rules.len() {
        cfg.rules[index].vpn_protected = !cfg.rules[index].vpn_protected;
        config::save_config(&cfg);
    }
    Ok(cfg)
}