// Отключаем консольное окно на Windows в релизной сборке (в dev-режиме консоль всё равно будет видна)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::process::Command;

// Пути к браузерам — потом вынесем в конфиг-файл, пока хардкодим для теста
const CHROME_PATH: &str = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
const YANDEX_PATH: &str = r"C:\Program Files\Yandex\YandexBrowser\Application\browser.exe";

// Правило: если домен содержит эту подстроку — открываем в рабочем браузере
const WORK_DOMAIN_MARKER: &str = "yandex-team.ru";

fn main() {
    // Забираем аргументы командной строки. args[0] — это путь к самой программе,
    // а args[1] (если есть) — это и есть ссылка, которую нам передала Windows
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Программу запустили без ссылки (например, просто дважды кликнули по .exe)
        // Пока просто выходим — окно показывать не будем
        println!("Нет URL в аргументах. Пример использования: browser-router.exe https://example.com");
        return;
    }

    let url = &args[1];
    println!("Получен URL: {}", url);

    // Решаем, какой браузер использовать
    let browser_path = if url.contains(WORK_DOMAIN_MARKER) {
        println!("Обнаружен рабочий домен -> открываем в Yandex");
        YANDEX_PATH
    } else {
        println!("Обычная ссылка -> открываем в Chrome");
        CHROME_PATH
    };

    // Запускаем нужный браузер с этим URL
    match Command::new(browser_path).arg(url).spawn() {
        Ok(_) => println!("Браузер запущен успешно"),
        Err(e) => eprintln!("Ошибка запуска браузера: {}", e),
    }
}