#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod registry;
mod commands;

use std::env;
use std::process::Command;
use std::thread;
use std::time::Duration;
use rfd::{MessageDialog, MessageButtons, MessageDialogResult};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        run_gui();
        return;
    }

    match args[1].as_str() {
        "--register" => {
            match registry::register() {
                Ok(_) => println!("Регистрация завершена успешно!"),
                Err(e) => eprintln!("Ошибка регистрации: {}", e),
            }
        }
        "--unregister" => {
            match registry::unregister() {
                Ok(_) => println!("Регистрация удалена."),
                Err(e) => eprintln!("Ошибка: {}", e),
            }
        }
        url => route_url(url),
    }
}

// Спрашивает текущий публичный IP через внешний сервис.
// Возвращает None, если не получилось (нет интернета, сервис недоступен и т.д.)
pub fn get_current_ip() -> Option<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok()?;

    let response = client.get("https://api.ipify.org").send().ok()?;
    let ip = response.text().ok()?;
    let ip = ip.trim().to_string();

    if ip.is_empty() {
        None
    } else {
        Some(ip)
    }
}

fn route_url(url: &str) {
    let cfg = config::load_config();

    let matched = cfg.rules.iter().find(|rule| url.contains(&rule.pattern));

    let (browser_path, needs_vpn_check) = match matched {
        Some(rule) => (rule.browser_path.clone(), rule.vpn_protected),
        None => (cfg.default_browser_path.clone(), false),
    };

    if needs_vpn_check && cfg.vpn_check_enabled {
        let current_ip = get_current_ip();

        let is_trusted = match &current_ip {
            Some(ip) => cfg.trusted_ips.contains(ip),
            None => false, // не смогли проверить -> считаем недоверенным, лучше перебдеть
        };

        if !is_trusted {
            let allowed = confirm_vpn_off(&cfg.ip_check_url, &cfg.default_browser_path);
            if !allowed {
                return;
            }
        }
        // Если IP совпал с доверенным -> открываем сразу, без лишних окон
    }

    let _ = Command::new(&browser_path).arg(url).spawn();
}

fn confirm_vpn_off(ip_check_url: &str, checker_browser: &str) -> bool {
    let step1 = MessageDialog::new()
        .set_title("Обнаружен незнакомый IP-адрес")
        .set_description(
            "Это рабочая ссылка, а твой текущий IP-адрес не входит в список доверенных — возможно, включён VPN.\n\nНажми \"OK\", чтобы открыть страницу с твоим текущим IP-адресом и проверить это.",
        )
        .set_buttons(MessageButtons::OkCancel)
        .show();

    if step1 != MessageDialogResult::Ok {
        return false;
    }

    let _ = Command::new(checker_browser).arg(ip_check_url).spawn();

    thread::sleep(Duration::from_secs(1));

    let step2 = MessageDialog::new()
        .set_title("Подтверждение")
        .set_description(
            "Посмотри на IP-адрес, который открылся в браузере.\n\nЕсли это твой настоящий адрес (VPN точно выключен) — нажми \"Да\".\n\nЕсли что-то не так — нажми \"Нет\", ссылка не откроется.",
        )
        .set_buttons(MessageButtons::YesNo)
        .show();

    step2 == MessageDialogResult::Yes
}

fn run_gui() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::add_rule,
            commands::remove_rule,
            commands::register_browser,
            commands::unregister_browser,
            commands::get_current_ip_command,
            commands::add_trusted_ip,
            commands::remove_trusted_ip,
            commands::get_installed_browsers,
            commands::toggle_vpn_protection,
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка запуска Tauri приложения");
}