import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Rule {
  pattern: string;
  browser_path: string;
  label: string;
  vpn_protected: boolean;
}

interface AppConfig {
  default_browser_path: string;
  rules: Rule[];
  vpn_check_enabled: boolean;
  ip_check_url: string;
}

function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [statusMessage, setStatusMessage] = useState("");

  const [newPattern, setNewPattern] = useState("");
  const [newBrowserPath, setNewBrowserPath] = useState("");
  const [newLabel, setNewLabel] = useState("");
  const [newVpnProtected, setNewVpnProtected] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  async function loadConfig() {
    const cfg = await invoke<AppConfig>("get_config");
    setConfig(cfg);
  }

  async function handleAddRule() {
    if (!newPattern || !newBrowserPath || !newLabel) {
      setStatusMessage("Заполни все поля перед добавлением правила");
      return;
    }
    const updated = await invoke<AppConfig>("add_rule", {
      pattern: newPattern,
      browserPath: newBrowserPath,
      label: newLabel,
      vpnProtected: newVpnProtected,
    });
    setConfig(updated);
    setNewPattern("");
    setNewBrowserPath("");
    setNewLabel("");
    setNewVpnProtected(false);
    setStatusMessage("Правило добавлено");
  }

  async function handleRemoveRule(index: number) {
    const updated = await invoke<AppConfig>("remove_rule", { index });
    setConfig(updated);
    setStatusMessage("Правило удалено");
  }

  async function handleRegister() {
    const message = await invoke<string>("register_browser");
    setStatusMessage(message);
  }

  async function handleUnregister() {
    const message = await invoke<string>("unregister_browser");
    setStatusMessage(message);
  }

  if (!config) {
    return <div className="container">Загрузка настроек...</div>;
  }

  return (
    <div className="container">
      <h1>Browser Router — Настройки</h1>

      <section>
        <h2>Регистрация в системе</h2>
        <p>
          Чтобы приложение начало ловить ссылки, нужно один раз
          зарегистрироваться, а потом выбрать его как браузер по умолчанию в
          Параметрах Windows.
        </p>
        <button onClick={handleRegister}>Зарегистрировать</button>
        <button onClick={handleUnregister}>Отменить регистрацию</button>
      </section>

      <section>
        <h2>Правила маршрутизации</h2>
        <table>
          <thead>
            <tr>
              <th>Название</th>
              <th>Если ссылка содержит</th>
              <th>Открывать в</th>
              <th>VPN-проверка</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {config.rules.map((rule, index) => (
              <tr key={index}>
                <td>{rule.label}</td>
                <td>{rule.pattern}</td>
                <td>{rule.browser_path}</td>
                <td>{rule.vpn_protected ? "Да" : "Нет"}</td>
                <td>
                  <button onClick={() => handleRemoveRule(index)}>
                    Удалить
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        <h3>Добавить новое правило</h3>
        <input
          placeholder="Название (например: Работа)"
          value={newLabel}
          onChange={(e) => setNewLabel(e.target.value)}
        />
        <input
          placeholder="Часть ссылки (например: yandex-team.ru)"
          value={newPattern}
          onChange={(e) => setNewPattern(e.target.value)}
        />
        <input
          placeholder="Путь к браузеру (.exe)"
          value={newBrowserPath}
          onChange={(e) => setNewBrowserPath(e.target.value)}
        />
        <label>
          <input
            type="checkbox"
            checked={newVpnProtected}
            onChange={(e) => setNewVpnProtected(e.target.checked)}
          />
          Требовать проверку VPN перед открытием
        </label>
        <button onClick={handleAddRule}>Добавить правило</button>
      </section>

      {statusMessage && <p className="status">{statusMessage}</p>}
    </div>
  );
}

export default App;