use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Rule {
    pub pattern: String,
    pub browser_path: String,
    pub label: String,
    pub vpn_protected: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub default_browser_path: String,
    pub rules: Vec<Rule>,
    pub vpn_check_enabled: bool,
    pub ip_check_url: String,
    pub trusted_ips: Vec<String>, // список IP, при которых считаем что VPN точно выключен
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            default_browser_path: r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
            rules: vec![Rule {
                pattern: "yandex-team.ru".to_string(),
                browser_path: r"C:\Program Files\Yandex\YandexBrowser\Application\browser.exe".to_string(),
                label: "Рабочие ссылки Яндекса".to_string(),
                vpn_protected: true,
            }],
            vpn_check_enabled: true,
            ip_check_url: "https://2ip.ru".to_string(),
            trusted_ips: vec![],
        }
    }
}

fn config_path() -> PathBuf {
    let mut dir = dirs::config_dir().expect("Не удалось найти папку AppData");
    dir.push("BrowserRouter");
    fs::create_dir_all(&dir).expect("Не удалось создать папку конфига");
    dir.push("config.json");
    dir
}

pub fn load_config() -> AppConfig {
    let path = config_path();

    if !path.exists() {
        let default = AppConfig::default();
        save_config(&default);
        return default;
    }

    let content = fs::read_to_string(&path).expect("Не удалось прочитать конфиг");
    serde_json::from_str(&content).unwrap_or_else(|_| {
        eprintln!("Конфиг повреждён, использую настройки по умолчанию");
        AppConfig::default()
    })
}

pub fn save_config(config: &AppConfig) {
    let path = config_path();
    let json = serde_json::to_string_pretty(config).expect("Не удалось сериализовать конфиг");
    fs::write(&path, json).expect("Не удалось сохранить конфиг");
}