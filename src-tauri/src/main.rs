#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env;
use std::process::Command;
use winreg::enums::*;
use winreg::RegKey;

const CHROME_PATH: &str = r"C:\Program Files\Google\Chrome\Application\chrome.exe";
const YANDEX_PATH: &str = r"C:\Program Files\Yandex\YandexBrowser\Application\browser.exe";
const WORK_DOMAIN_MARKER: &str = "yandex-team.ru";

// Имя, под которым программа будет "представляться" системе
const APP_NAME: &str = "BrowserRouter";
const APP_DESCRIPTION: &str = "Routes links between Chrome and Yandex Browser";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Использование:");
        println!("  browser-router.exe --register    — зарегистрировать как браузер по умолчанию");
        println!("  browser-router.exe --unregister  — убрать регистрацию");
        println!("  browser-router.exe <url>         — открыть ссылку в нужном браузере");
        return;
    }

    match args[1].as_str() {
        "--register" => register(),
        "--unregister" => unregister(),
        url => route_url(url),
    }
}

// Логика маршрутизации ссылки — то, что уже проверили и оно работает
fn route_url(url: &str) {
    println!("Получен URL: {}", url);

    let browser_path = if url.contains(WORK_DOMAIN_MARKER) {
        println!("Обнаружен рабочий домен -> открываем в Yandex");
        YANDEX_PATH
    } else {
        println!("Обычная ссылка -> открываем в Chrome");
        CHROME_PATH
    };

    match Command::new(browser_path).arg(url).spawn() {
        Ok(_) => println!("Браузер запущен успешно"),
        Err(e) => eprintln!("Ошибка запуска браузера: {}", e),
    }
}

// Регистрация в реестре Windows как кандидата на "браузер по умолчанию"
fn register() {
    // Путь к самому себе — нужно записать в реестр АБСОЛЮТНЫЙ путь до exe
    let exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Не удалось определить путь к самой программе: {}", e);
            return;
        }
    };
    let exe_path_str = exe_path.to_string_lossy().to_string();

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // 1. Описываем саму программу как обработчик (ProgID)
    let (prog_key, _) = hkcu
        .create_subkey(format!(r"Software\Classes\{}", APP_NAME))
        .expect("Не удалось создать ключ ProgID");
    prog_key
        .set_value("", &APP_DESCRIPTION)
        .expect("Не удалось задать описание");

    // 2. Команда запуска: путь_к_exe "%1" (Windows подставит вместо %1 саму ссылку)
    let (command_key, _) = prog_key
        .create_subkey(r"shell\open\command")
        .expect("Не удалось создать ключ command");
    let command_value = format!("\"{}\" \"%1\"", exe_path_str);
    command_key
        .set_value("", &command_value)
        .expect("Не удалось задать команду запуска");

    // 3. Регистрируем приложение, чтобы оно появилось в списке кандидатов
    let (registered_apps, _) = hkcu
        .create_subkey(r"Software\RegisteredApplications")
        .expect("Не удалось открыть RegisteredApplications");
    registered_apps
        .set_value(
            APP_NAME,
            &format!(r"Software\Classes\{}\Capabilities", APP_NAME),
        )
        .expect("Не удалось зарегистрировать приложение");

    // 4. Capabilities — описываем, что умеем открывать http/https
    let (capabilities_key, _) = hkcu
        .create_subkey(format!(r"Software\Classes\{}\Capabilities", APP_NAME))
        .expect("Не удалось создать Capabilities");
    capabilities_key
        .set_value("ApplicationName", &APP_NAME)
        .expect("Не удалось задать ApplicationName");
    capabilities_key
        .set_value("ApplicationDescription", &APP_DESCRIPTION)
        .expect("Не удалось задать ApplicationDescription");

    let (url_assoc_key, _) = capabilities_key
        .create_subkey("URLAssociations")
        .expect("Не удалось создать URLAssociations");
    url_assoc_key
        .set_value("http", &APP_NAME)
        .expect("Не удалось привязать http");
    url_assoc_key
        .set_value("https", &APP_NAME)
        .expect("Не удалось привязать https");

    println!("Регистрация завершена успешно!");
    println!("Теперь зайди в Параметры Windows -> Приложения -> Приложения по умолчанию -> Веб-браузер");
    println!("и выбери \"{}\" из списка.", APP_NAME);
}

// Полная очистка всех ключей, которые мы создали
fn unregister() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let _ = hkcu.delete_subkey_all(format!(r"Software\Classes\{}", APP_NAME));

    if let Ok(registered_apps) = hkcu.open_subkey_with_flags(
        r"Software\RegisteredApplications",
        KEY_SET_VALUE,
    ) {
        let _ = registered_apps.delete_value(APP_NAME);
    }

    println!("Регистрация удалена. Программа больше не будет предложена как браузер по умолчанию.");
}