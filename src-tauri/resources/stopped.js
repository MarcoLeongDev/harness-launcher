(() => {
  function invoke(cmd, args) {
    const core = window.__TAURI__?.core;
    if (core && typeof core.invoke === "function") return core.invoke(cmd, args || {});
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === "function")
      return window.__TAURI_INTERNALS__.invoke(cmd, args || {}, undefined);
    return Promise.reject(new Error("Tauri IPC unavailable"));
  }
  const LOCALES = {
    en: {
      title: "Engine Stopped - Harness Launcher",
      stoppedSub: "The harness engine is stopped",
      stoppedDesc:
        "The Harness Launcher engine is not running. Start it to open the harness UI, or open the Control Panel to manage versions, updates, port and logs.",
      startEngine: "Start Engine",
      openPanel: "Open Control Panel",
      brandGithub: "Open Harness Launcher on GitHub",
      logoAlt: "Harness Launcher logo",
    },
    "zh-Hant": {
      title: "引擎已停止 - Harness Launcher",
      stoppedSub: "Harness 引擎已停止",
      stoppedDesc:
        "Harness Launcher 引擎尚未執行。啟動以開啟 Harness 介面，或開啟控制面板以管理版本、更新、連接埠和日誌。",
      startEngine: "啟動引擎",
      openPanel: "開啟控制面板",
      brandGithub: "在 GitHub 上開啟 Harness Launcher",
      logoAlt: "Harness Launcher 標誌",
    },
    "zh-Hans": {
      title: "引擎已停止 - Harness Launcher",
      stoppedSub: "Harness 引擎已停止",
      stoppedDesc:
        "Harness Launcher 引擎尚未运行。启动以打开 Harness 界面，或打开控制面板以管理版本、更新、端口和日志。",
      startEngine: "启动引擎",
      openPanel: "打开控制面板",
      brandGithub: "在 GitHub 上打开 Harness Launcher",
      logoAlt: "Harness Launcher 标志",
    },
    ja: {
      title: "エンジン停止中 - Harness Launcher",
      stoppedSub: "Harness エンジンは停止しています",
      stoppedDesc:
        "Harness Launcher エンジンは実行されていません。起動して Harness UI を開くか、コントロールパネルでバージョン・更新・ポート・ログを管理してください。",
      startEngine: "エンジンを起動",
      openPanel: "コントロールパネルを開く",
      brandGithub: "GitHub で Harness Launcher を開く",
      logoAlt: "Harness Launcher のロゴ",
    },
    es: {
      title: "Motor detenido - Harness Launcher",
      stoppedSub: "El motor Harness está detenido",
      stoppedDesc:
        "El motor de Harness Launcher no está en ejecución. Inícialo para abrir la interfaz de Harness o abre el panel de control para gestionar versiones, actualizaciones, puerto y registros.",
      startEngine: "Iniciar motor",
      openPanel: "Abrir panel de control",
      brandGithub: "Abrir Harness Launcher en GitHub",
      logoAlt: "Logotipo de Harness Launcher",
    },
  };
  let lang = "en";
  function t(key) {
    return LOCALES[lang]?.[key] || LOCALES.en[key] || key;
  }
  function applyLanguage(l) {
    if (!LOCALES[l]) l = "en";
    lang = l;
    try {
      document.documentElement.lang = l;
    } catch (_e) {}
    try {
      document.title = t("title");
    } catch (_e) {}
    let i, els;
    els = document.querySelectorAll("[data-i18n]");
    for (i = 0; i < els.length; i++) els[i].textContent = t(els[i].getAttribute("data-i18n"));
    els = document.querySelectorAll("[data-i18n-title]");
    for (i = 0; i < els.length; i++) els[i].title = t(els[i].getAttribute("data-i18n-title"));
    els = document.querySelectorAll("[data-i18n-aria]");
    for (i = 0; i < els.length; i++)
      els[i].setAttribute("aria-label", t(els[i].getAttribute("data-i18n-aria")));
    els = document.querySelectorAll("[data-i18n-alt]");
    for (i = 0; i < els.length; i++) els[i].setAttribute("alt", t(els[i].getAttribute("data-i18n-alt")));
  }
  function refresh() {
    invoke("get_status", {})
      .then((s) => {
        if (s?.language) applyLanguage(s.language);
      })
      .catch(() => {
        applyLanguage("en");
      });
  }
  const hint = document.getElementById("hint");
  function run(fn) {
    hint.textContent = "...";
    fn()
      .then((r) => {
        hint.textContent = r?.message || "done";
        setTimeout(() => {
          hint.textContent = "";
        }, 4000);
      })
      .catch((e) => {
        hint.textContent = String(e?.message || e);
      });
  }
  document.getElementById("btn-start").addEventListener("click", () => {
    run(() => invoke("engine_start", {}));
  });
  document.getElementById("btn-settings").addEventListener("click", () => {
    run(() => invoke("open_settings", {}));
  });
  document.getElementById("brand-link").addEventListener("click", (e) => {
    e.preventDefault();
    invoke("open_repo_page", {}).catch((err) => {
      hint.textContent = String(err?.message || err);
    });
  });
  // Instant repaint on language switch; polling stays as a reconnect fallback.
  try {
    const ev = window.__TAURI__?.event;
    if (ev && typeof ev.listen === "function") {
      ev.listen("launcher://language", (e) => {
        if (e?.payload?.language) applyLanguage(e.payload.language);
      });
    }
  } catch (_e) {}
  refresh();
  try {
    setInterval(refresh, 3000);
  } catch (_e) {}
  try {
    document.addEventListener("visibilitychange", () => {
      if (!document.hidden) refresh();
    });
  } catch (_e) {}
  try {
    window.addEventListener("focus", refresh);
  } catch (_e) {}
})();
