#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;

use std::env;
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

const APP_NAME: &str = "BrowserRouter";
const APP_DESCRIPTION: &str = "Routes links between browsers";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Запустили без аргументов -> открываем окно настроек (GUI)
        run_gui();
        return;
    }

    match args[1].as_str() {
        "--register" => register(),
        "--unregister" => unregister(),
        url => route_url(url),
    }
}

// Логика маршрутизации — теперь читает правила из конфига, а не из хардкода
fn route_url(url: &str) {
    let cfg = config::load_config();

    println!("Получен URL: {}", url);

    // Ищем первое подходящее правило
    let browser_path = cfg
        .rules
        .iter()
        .find(|rule| url.contains(&rule.pattern))
        .map(|rule| {
            println!("Совпало правило \"{}\" -> {}", rule.label, rule.browser_path);
            rule.browser_path.clone()
        })
        .unwrap_or_else(|| {
            println!("Ни одно правило не подошло -> браузер по умолчанию");
            cfg.default_browser_path.clone()
        });

    match Command::new(&browser_path).arg(url).spawn() {
        Ok(_) => println!("Браузер запущен успешно"),
        Err(e) => eprintln!("Ошибка запуска браузера ({}): {}", browser_path, e),
    }
}

// Заглушка — реальный GUI подключим на следующем шаге через Tauri
fn run_gui() {
    println!("GUI ещё не подключен — на следующем шаге");
}

fn register() {
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Не удалось определить путь к самой программе: {}", e);
            return;
        }
    };
    let exe_path_str = exe_path.to_string_lossy().to_string();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let (prog_key, _) = hkcu
        .create_subkey(format!(r"Software\Classes\{}", APP_NAME))
        .expect("Не удалось создать ключ ProgID");
    prog_key.set_value("", &APP_DESCRIPTION).expect("Не удалось задать описание");

    let (command_key, _) = prog_key
        .create_subkey(r"shell\open\command")
        .expect("Не удалось создать ключ command");
    let command_value = format!("\"{}\" \"%1\"", exe_path_str);
    command_key.set_value("", &command_value).expect("Не удалось задать команду запуска");

    let (registered_apps, _) = hkcu
        .create_subkey(r"Software\RegisteredApplications")
        .expect("Не удалось открыть RegisteredApplications");
    registered_apps
        .set_value(APP_NAME, &format!(r"Software\Classes\{}\Capabilities", APP_NAME))
        .expect("Не удалось зарегистрировать приложение");

    let (capabilities_key, _) = hkcu
        .create_subkey(format!(r"Software\Classes\{}\Capabilities", APP_NAME))
        .expect("Не удалось создать Capabilities");
    capabilities_key.set_value("ApplicationName", &APP_NAME).expect("ApplicationName");
    capabilities_key.set_value("ApplicationDescription", &APP_DESCRIPTION).expect("ApplicationDescription");

    let (url_assoc_key, _) = capabilities_key
        .create_subkey("URLAssociations")
        .expect("Не удалось создать URLAssociations");
    url_assoc_key.set_value("http", &APP_NAME).expect("http");
    url_assoc_key.set_value("https", &APP_NAME).expect("https");

    println!("Регистрация завершена успешно!");
}

fn unregister() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let _ = hkcu.delete_subkey_all(format!(r"Software\Classes\{}", APP_NAME));
    if let Ok(registered_apps) = hkcu.open_subkey_with_flags(r"Software\RegisteredApplications", KEY_SET_VALUE) {
        let _ = registered_apps.delete_value(APP_NAME);
    }
    println!("Регистрация удалена.");
}