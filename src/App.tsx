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
  trusted_ips: string[];
}

interface BrowserInfo {
  name: string;
  exe_path: string;
}

function deepClone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}

function App() {
  const [revealedIps, setRevealedIps] = useState<Set<number>>(new Set());

  function toggleIpReveal(index: number) {
    setRevealedIps((prev) => {
      const updated = new Set(prev);
      if (updated.has(index)) {
        updated.delete(index);
      } else {
        updated.add(index);
      }
      return updated;
    });
  }
  
  const [saved, setSaved] = useState<AppConfig | null>(null);
  const [draft, setDraft] = useState<AppConfig | null>(null);
  const [browsers, setBrowsers] = useState<BrowserInfo[]>([]);
  const [statusMessage, setStatusMessage] = useState("");
  const [currentIp, setCurrentIp] = useState<string | null>(null);
  const [checkingIp, setCheckingIp] = useState(false);

  const [newPattern, setNewPattern] = useState("");
  const [newBrowserPath, setNewBrowserPath] = useState("");
  const [newLabel, setNewLabel] = useState("");
  const [newVpnProtected, setNewVpnProtected] = useState(false);
  const [useCustomPath, setUseCustomPath] = useState(false);

  const hasUnsavedChanges =
    draft && saved && JSON.stringify(draft) !== JSON.stringify(saved);

  useEffect(() => {
    loadConfig();
    loadBrowsers();
  }, []);

  async function loadConfig() {
    const cfg = await invoke<AppConfig>("get_config");
    setSaved(deepClone(cfg));
    setDraft(deepClone(cfg));
  }

  async function loadBrowsers() {
    const list = await invoke<BrowserInfo[]>("get_installed_browsers");
    setBrowsers(list);
    if (list.length > 0) {
      setNewBrowserPath(list[0].exe_path);
    } else {
      setUseCustomPath(true); // нет данных из реестра -> сразу даём ввести путь руками
    }
  }

  async function handleCheckMyIp() {
    setCheckingIp(true);
    try {
      const ip = await invoke<string>("get_current_ip_command");
      setCurrentIp(ip);
    } catch (e) {
      setStatusMessage(String(e));
    } finally {
      setCheckingIp(false);
    }
  }

  function handleAddTrustedIp() {
    if (!currentIp || !draft) return;
    if (draft.trusted_ips.includes(currentIp)) {
      setStatusMessage("Этот IP уже в списке");
      return;
    }
    setDraft({ ...draft, trusted_ips: [...draft.trusted_ips, currentIp] });
  }

  function handleRemoveTrustedIp(index: number) {
    if (!draft) return;
    const updated = draft.trusted_ips.filter((_, i) => i !== index);
    setDraft({ ...draft, trusted_ips: updated });
  }

  function handleToggleVpn(index: number) {
    if (!draft) return;
    const updatedRules = draft.rules.map((rule, i) =>
      i === index ? { ...rule, vpn_protected: !rule.vpn_protected } : rule
    );
    setDraft({ ...draft, rules: updatedRules });
  }

  function handleAddRule() {
    if (!draft) return;
    if (!newPattern || !newBrowserPath || !newLabel) {
      setStatusMessage("Заполни все поля перед добавлением правила");
      return;
    }
    const newRule: Rule = {
      pattern: newPattern,
      browser_path: newBrowserPath,
      label: newLabel,
      vpn_protected: newVpnProtected,
    };
    setDraft({ ...draft, rules: [...draft.rules, newRule] });
    setNewPattern("");
    setNewLabel("");
    setNewVpnProtected(false);
  }

  function handleRemoveRule(index: number) {
    if (!draft) return;
    setDraft({ ...draft, rules: draft.rules.filter((_, i) => i !== index) });
  }

  async function handleSave() {
    if (!draft) return;
    try {
      await invoke("save_config", { newConfig: draft });
      setSaved(deepClone(draft));
      setStatusMessage("Изменения сохранены");
    } catch (e) {
      setStatusMessage("Ошибка сохранения: " + String(e));
    }
  }

  function handleCancel() {
    if (!saved) return;
    setDraft(deepClone(saved));
    setStatusMessage("Изменения отменены");
  }

  async function handleRegister() {
    const message = await invoke<string>("register_browser");
    setStatusMessage(message);
  }

  async function handleUnregister() {
    const message = await invoke<string>("unregister_browser");
    setStatusMessage(message);
  }

  if (!draft) {
    return <div className="container">Загрузка настроек...</div>;
  }

  return (
    <div className="container">
      <h1>Browser Router — Настройки</h1>

      {hasUnsavedChanges && (
        <div className="unsaved-bar">
          У тебя есть несохранённые изменения
          <button onClick={handleSave}>Сохранить</button>
          <button onClick={handleCancel}>Отмена</button>
        </div>
      )}

      <section>
        <h2>Регистрация в системе</h2>
        <button onClick={handleRegister}>Зарегистрировать</button>
        <button onClick={handleUnregister}>Отменить регистрацию</button>
      </section>

      <section>
        <h2>Доверенные IP (для VPN-проверки)</h2>
        <button onClick={handleCheckMyIp} disabled={checkingIp}>
          {checkingIp ? "Проверяю..." : "Узнать мой текущий IP"}
        </button>
        {currentIp && <p>Твой текущий IP: <strong>{currentIp}</strong></p>}
        <button onClick={handleAddTrustedIp} disabled={!currentIp}>
          Добавить текущий IP в доверенные
        </button>
        <ul>
  {draft.trusted_ips.map((ip, index) => (
    <li key={index}>
      <span
        className={revealedIps.has(index) ? "ip-text" : "ip-text blurred"}
        onClick={() => toggleIpReveal(index)}
        title="Нажми, чтобы показать/скрыть"
      >
        {ip}
      </span>
      <button className="danger" onClick={() => handleRemoveTrustedIp(index)}>
        Удалить
      </button>
    </li>
  ))}
</ul>
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
            {draft.rules.map((rule, index) => (
              <tr key={index}>
                <td>{rule.label}</td>
                <td>{rule.pattern}</td>
                <td>{rule.browser_path}</td>
                <td>
                  <input
                    type="checkbox"
                    checked={rule.vpn_protected}
                    onChange={() => handleToggleVpn(index)}
                  />
                </td>
                <td>
                  <button onClick={() => handleRemoveRule(index)}>Удалить</button>
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

        {!useCustomPath ? (
          <select
            value={newBrowserPath}
            onChange={(e) => setNewBrowserPath(e.target.value)}
          >
            {browsers.map((b) => (
              <option key={b.exe_path} value={b.exe_path}>
                {b.name}
              </option>
            ))}
          </select>
        ) : (
          <input
            placeholder="Путь к браузеру (.exe)"
            value={newBrowserPath}
            onChange={(e) => setNewBrowserPath(e.target.value)}
          />
        )}
        <label>
          <input
            type="checkbox"
            checked={useCustomPath}
            onChange={(e) => setUseCustomPath(e.target.checked)}
          />
          Указать путь вручную (браузера нет в списке)
        </label>

        <label>
          <input
            type="checkbox"
            checked={newVpnProtected}
            onChange={(e) => setNewVpnProtected(e.target.checked)}
          />
          Требовать проверку VPN перед открытием
        </label>
        <button onClick={handleAddRule}>Добавить правило (в черновик)</button>
      </section>

      {statusMessage && <p className="status">{statusMessage}</p>}
    </div>
  );
}

export default App;