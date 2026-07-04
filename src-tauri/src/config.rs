use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// Одно правило: если ссылка содержит `pattern` — открываем в `browser_path`
#[derive(Serialize, Deserialize, Clone)]
pub struct Rule {
    pub pattern: String,
    pub browser_path: String,
    pub label: String, // человекочитаемое имя для GUI, типа "Рабочие ссылки Яндекса"
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub default_browser_path: String, // куда падает всё, что не подошло под правила
    pub rules: Vec<Rule>,
    pub vpn_check_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            default_browser_path: r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
            rules: vec![Rule {
                pattern: "yandex-team.ru".to_string(),
                browser_path: r"C:\Program Files\Yandex\YandexBrowser\Application\browser.exe".to_string(),
                label: "Рабочие ссылки Яндекса".to_string(),
            }],
            vpn_check_enabled: true,
        }
    }
}

// Путь к конфигу: C:\Users\Kirill\AppData\Roaming\BrowserRouter\config.json
fn config_path() -> PathBuf {
    let mut dir = dirs::config_dir().expect("Не удалось найти папку AppData");
    dir.push("BrowserRouter");
    fs::create_dir_all(&dir).expect("Не удалось создать папку конфига");
    dir.push("config.json");
    dir
}

// Загружает конфиг с диска. Если файла ещё нет (первый запуск) — создаёт с настройками по умолчанию
pub fn load_config() -> AppConfig {
    let path = config_path();

    if !path.exists() {
        let default = AppConfig::default();
        save_config(&default);
        return default;
    }

    let content = fs::read_to_string(&path).expect("Не удалось прочитать конфиг");
    serde_json::from_str(&content).unwrap_or_else(|_| {
        // Если файл повреждён/битый JSON — не падаем, а откатываемся на дефолт
        eprintln!("Конфиг повреждён, использую настройки по умолчанию");
        AppConfig::default()
    })
}

pub fn save_config(config: &AppConfig) {
    let path = config_path();
    let json = serde_json::to_string_pretty(config).expect("Не удалось сериализовать конфиг");
    fs::write(&path, json).expect("Не удалось сохранить конфиг");
}