(() => {
  if (window.__DSH_CP__) return;
  window.__DSH_CP__ = true;

  function invoke(cmd, args) {
    const core = window.__TAURI__?.core;
    if (core && typeof core.invoke === "function") return core.invoke(cmd, args || {});
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === "function")
      return window.__TAURI_INTERNALS__.invoke(cmd, args || {}, undefined);
    return Promise.reject(new Error("Tauri IPC unavailable in this context"));
  }
  function listen(evt, cb) {
    const ev = window.__TAURI__?.event;
    if (ev && typeof ev.listen === "function")
      return ev.listen(evt, (e) => {
        cb(e.payload);
      });
    return Promise.resolve();
  }

  const $ = (id) => document.getElementById(id);

  function loadIconsIn(root) {
    const icons = root.querySelectorAll("i.ico.bi[data-ico]");
    for (let i = 0; i < icons.length; i++) {
      ((el) => {
        if (el.getAttribute("data-loaded")) return;
        const name = el.getAttribute("data-ico");
        if (!name) return;
        // The fetched SVG is injected as markup, so the icon name is
        // constrained to the bundled set (lowercase letters, digits, dashes).
        if (!/^[a-z0-9-]+$/.test(name)) return;
        const url = `dsh-ui://localhost/bootstrap-icons/${name}.svg`;
        fetch(url)
          .then((r) => {
            if (!r.ok) return;
            return r.text();
          })
          .then((svg) => {
            if (!svg) return;
            const wrapper = document.createElement("span");
            wrapper.innerHTML = svg;
            const svgEl = wrapper.querySelector("svg");
            if (!svgEl) return;
            svgEl.classList.add("ico");
            if (el.classList.contains("sm")) svgEl.classList.add("sm");
            svgEl.removeAttribute("width");
            svgEl.removeAttribute("height");
            el.setAttribute("data-loaded", "1");
            el.parentNode.replaceChild(svgEl, el);
          })
          .catch(() => {});
      })(icons[i]);
    }
  }

  // ---------- UI language (En | 繁 | 簡 | 日 | Es) ----------
  // Locale table for the Control Panel chrome. Backend progress/notice
  // lines and npm output stay English (operational output, like a terminal).
  // Missing keys fall back to English; unknown codes fall back to En.
  const LOCALES = {
    en: {
      brandGithub: "Open Harness Launcher on GitHub",
      languageName: "Language",
      quit: "Quit",
      engineStatus: "Engine status",
      engineVersion: "Engine version",
      port: "Port",
      openBrowser: "Open in browser",
      start: "Start",
      stop: "Stop",
      restart: "Restart",
      updateBtn: "Update",
      updateAvail: "Update available: v{active} → v{latest}",
      active: "Active",
      activeTitle: "Active version",
      versionCol: "Version",
      directory: "Directory",
      directoryTitle: "Open the version directory",
      actions: "Actions",
      installAria: "Version to install",
      refreshList: "Refresh version list",
      downloadVer: "Download selected version",
      noOutput: "No output yet",
      noLogs: "(no logs yet)",
      goLast: "Go to last line",
      showLog: "Show log",
      hideLog: "Hide log",
      noVersions: "No versions installed — pick one below to install.",
      activeIs: "This is the active version",
      makeActive: "Make v{ver} the active version",
      openDir: "Open the directory for v{ver}",
      delVer: "Delete this version",
      delAria: "Delete version v{ver}?",
      delLead: "Delete ",
      delTail: "? This cannot be undone.",
      cancel: "Cancel",
      delConfirm: "Delete",
      validPort: "Enter a valid port (1–65535)",
      refreshed: "Version list refreshed",
      stuckHint: "no progress for 60s — still working…",
      completed: "Completed",
      doneSwitched: "Switched",
      doneVersionSet: "Version set",
      doneDownloaded: "Downloaded",
      doneUpdated: "Updated",
      doneStarted: "Started",
      doneStopped: "Stopped",
      doneRestarted: "Restarted",
      donePortSet: "Port set",
    },
    "zh-Hant": {
      brandGithub: "在 GitHub 上開啟 Harness Launcher",
      languageName: "語言",
      quit: "結束",
      engineStatus: "引擎狀態",
      engineVersion: "引擎版本",
      port: "連接埠",
      openBrowser: "在瀏覽器中開啟",
      start: "啟動",
      stop: "停止",
      restart: "重新啟動",
      updateBtn: "更新",
      updateAvail: "有可用更新：v{active} → v{latest}",
      active: "使用中",
      activeTitle: "使用中的版本",
      versionCol: "版本",
      directory: "目錄",
      directoryTitle: "開啟版本目錄",
      actions: "操作",
      installAria: "要安裝的版本",
      refreshList: "重新整理版本清單",
      downloadVer: "下載所選版本",
      noOutput: "尚無輸出",
      noLogs: "（尚無日誌）",
      goLast: "跳到最後一行",
      showLog: "顯示日誌",
      hideLog: "隱藏日誌",
      noVersions: "尚未安裝任何版本 — 請在下方挑選版本安裝。",
      activeIs: "這是使用中的版本",
      makeActive: "將 v{ver} 設為使用中版本",
      openDir: "開啟 v{ver} 的目錄",
      delVer: "刪除此版本",
      delAria: "刪除版本 v{ver}？",
      delLead: "刪除 ",
      delTail: "？此動作無法復原。",
      cancel: "取消",
      delConfirm: "刪除",
      validPort: "請輸入有效的連接埠（1–65535）",
      refreshed: "版本清單已重新整理",
      stuckHint: "60 秒無進度 — 仍在處理…",
      completed: "完成",
      doneSwitched: "已切換",
      doneVersionSet: "已設定版本",
      doneDownloaded: "已下載",
      doneUpdated: "已更新",
      doneStarted: "已啟動",
      doneStopped: "已停止",
      doneRestarted: "已重新啟動",
      donePortSet: "已設定連接埠",
    },
    "zh-Hans": {
      brandGithub: "在 GitHub 上打开 Harness Launcher",
      languageName: "语言",
      quit: "退出",
      engineStatus: "引擎状态",
      engineVersion: "引擎版本",
      port: "端口",
      openBrowser: "在浏览器中打开",
      start: "启动",
      stop: "停止",
      restart: "重新启动",
      updateBtn: "更新",
      updateAvail: "有可用更新：v{active} → v{latest}",
      active: "使用中",
      activeTitle: "使用中的版本",
      versionCol: "版本",
      directory: "目录",
      directoryTitle: "打开版本目录",
      actions: "操作",
      installAria: "要安装的版本",
      refreshList: "刷新版本列表",
      downloadVer: "下载所选版本",
      noOutput: "暂无输出",
      noLogs: "（暂无日志）",
      goLast: "跳到最后一行",
      showLog: "显示日志",
      hideLog: "隐藏日志",
      noVersions: "尚未安装任何版本 — 请在下方挑选版本安装。",
      activeIs: "这是使用中的版本",
      makeActive: "将 v{ver} 设为使用中版本",
      openDir: "打开 v{ver} 的目录",
      delVer: "删除此版本",
      delAria: "删除版本 v{ver}？",
      delLead: "删除 ",
      delTail: "？此操作无法撤销。",
      cancel: "取消",
      delConfirm: "删除",
      validPort: "请输入有效端口（1–65535）",
      refreshed: "版本列表已刷新",
      stuckHint: "60 秒无进度 — 仍在处理…",
      completed: "完成",
      doneSwitched: "已切换",
      doneVersionSet: "已设置版本",
      doneDownloaded: "已下载",
      doneUpdated: "已更新",
      doneStarted: "已启动",
      doneStopped: "已停止",
      doneRestarted: "已重新启动",
      donePortSet: "已设置端口",
    },
    ja: {
      brandGithub: "GitHub で Harness Launcher を開く",
      languageName: "言語",
      quit: "終了",
      engineStatus: "エンジンの状態",
      engineVersion: "エンジンのバージョン",
      port: "ポート",
      openBrowser: "ブラウザで開く",
      start: "起動",
      stop: "停止",
      restart: "再起動",
      updateBtn: "更新",
      updateAvail: "更新があります：v{active} → v{latest}",
      active: "使用中",
      activeTitle: "使用中のバージョン",
      versionCol: "バージョン",
      directory: "ディレクトリ",
      directoryTitle: "バージョンディレクトリを開く",
      actions: "操作",
      installAria: "インストールするバージョン",
      refreshList: "バージョン一覧を更新",
      downloadVer: "選択したバージョンをダウンロード",
      noOutput: "出力はまだありません",
      noLogs: "（ログはまだありません）",
      goLast: "最終行へ移動",
      showLog: "ログを表示",
      hideLog: "ログを非表示",
      noVersions: "バージョンがインストールされていません — 下から選んでインストールしてください。",
      activeIs: "これが使用中のバージョンです",
      makeActive: "v{ver} を使用中のバージョンにする",
      openDir: "v{ver} のディレクトリを開く",
      delVer: "このバージョンを削除",
      delAria: "バージョン v{ver} を削除しますか？",
      delLead: "削除 ",
      delTail: "？この操作は取り消せません。",
      cancel: "キャンセル",
      delConfirm: "削除",
      validPort: "有効なポートを入力してください（1–65535）",
      refreshed: "バージョン一覧を更新しました",
      stuckHint: "60 秒間進捗なし — 処理中…",
      completed: "完了",
      doneSwitched: "切替済み",
      doneVersionSet: "バージョンを設定済み",
      doneDownloaded: "ダウンロード済み",
      doneUpdated: "更新済み",
      doneStarted: "起動済み",
      doneStopped: "停止済み",
      doneRestarted: "再起動済み",
      donePortSet: "ポートを設定済み",
    },
    es: {
      brandGithub: "Abrir Harness Launcher en GitHub",
      languageName: "Idioma",
      quit: "Salir",
      engineStatus: "Estado del motor",
      engineVersion: "Versión del motor",
      port: "Puerto",
      openBrowser: "Abrir en el navegador",
      start: "Iniciar",
      stop: "Detener",
      restart: "Reiniciar",
      updateBtn: "Actualizar",
      updateAvail: "Actualización disponible: v{active} → v{latest}",
      active: "Activa",
      activeTitle: "Versión activa",
      versionCol: "Versión",
      directory: "Directorio",
      directoryTitle: "Abrir el directorio de la versión",
      actions: "Acciones",
      installAria: "Versión a instalar",
      refreshList: "Actualizar la lista de versiones",
      downloadVer: "Descargar la versión seleccionada",
      noOutput: "Sin salida todavía",
      noLogs: "(sin registros todavía)",
      goLast: "Ir a la última línea",
      showLog: "Mostrar registro",
      hideLog: "Ocultar registro",
      noVersions: "No hay versiones instaladas — elige una abajo para instalar.",
      activeIs: "Esta es la versión activa",
      makeActive: "Convertir v{ver} en la versión activa",
      openDir: "Abrir el directorio de v{ver}",
      delVer: "Eliminar esta versión",
      delAria: "¿Eliminar la versión v{ver}?",
      delLead: "Eliminar ",
      delTail: "? Esta acción no se puede deshacer.",
      cancel: "Cancelar",
      delConfirm: "Eliminar",
      validPort: "Introduce un puerto válido (1–65535)",
      refreshed: "Lista de versiones actualizada",
      stuckHint: "sin progreso durante 60 s — trabajando…",
      completed: "Completado",
      doneSwitched: "Cambiado",
      doneVersionSet: "Versión establecida",
      doneDownloaded: "Descargado",
      doneUpdated: "Actualizado",
      doneStarted: "Iniciado",
      doneStopped: "Detenido",
      doneRestarted: "Reiniciado",
      donePortSet: "Puerto establecido",
    },
  };

  let lang = "en";
  function t(key) {
    return LOCALES[lang]?.[key] || LOCALES.en[key] || key;
  }
  function fmt(key, params) {
    let s = t(key);
    for (const k in params) s = s.split(`{${k}}`).join(params[k]);
    return s;
  }
  // Repaint every localizable string for language l and mark the matching
  // segment pressed. renderStatus calls this on every poll, so the panel
  // converges on the saved language even after a restart.
  function applyLanguage(l) {
    if (!LOCALES[l]) l = "en";
    lang = l;
    try {
      document.documentElement.lang = l;
    } catch (_e) {}
    let i, els;
    els = document.querySelectorAll("[data-i18n]");
    for (i = 0; i < els.length; i++) els[i].textContent = t(els[i].getAttribute("data-i18n"));
    els = document.querySelectorAll("[data-i18n-title]");
    for (i = 0; i < els.length; i++) els[i].title = t(els[i].getAttribute("data-i18n-title"));
    els = document.querySelectorAll("[data-i18n-aria]");
    for (i = 0; i < els.length; i++)
      els[i].setAttribute("aria-label", t(els[i].getAttribute("data-i18n-aria")));
    const seg = $("lang-seg");
    if (seg?.querySelectorAll) {
      const btns = seg.querySelectorAll("button");
      for (i = 0; i < btns.length; i++) {
        const b = btns[i];
        if (b.getAttribute && b.setAttribute)
          b.setAttribute("aria-pressed", b.getAttribute("data-lang") === l ? "true" : "false");
      }
    }
  }
  // Persist the choice, then repaint from the canonical status payload.
  function setLanguage(l) {
    if (!LOCALES[l] || l === lang) return;
    applyLanguage(l);
    invoke("set_language", { language: l })
      .then(() => refresh())
      .catch((e) => {
        setResult(String(e.message || e), true);
        return refresh();
      });
  }

  const sel = {
    stIcon: $("st-icon"),
    toggle: $("btn-toggle"),
    restart: $("btn-restart"),
    engineVer: $("sel-version"),
    verWrap: $("ver-wrap"),
    verChevron: $("ver-chevron"),
    host: $("host-text"),
    hostWrap: $("host-wrap"),
    port: $("inp-port"),

    installSel: $("sel-install-ver"),
    btnInstall: $("btn-install"),
    btnRefreshVer: $("btn-refresh-versions"),
    verTbody: $("ver-tbody"),
    feedsBox: $("op-feeds"),
    logFollow: $("btn-log-follow"),
    logTail: $("log-tail"),
    logToggle: $("btn-log-toggle"),
    logDrawer: $("log-drawer"),
    banner: $("update-banner"),
    bannerText: $("update-banner-text"),
    bannerBtn: $("btn-banner-update"),
    appVer: $("app-ver"),
    langSeg: $("lang-seg"),
    browser: $("btn-open"),
    quit: $("btn-quit"),
  };

  let followOn = true;
  let lastPhase = "stopped";

  // ---------- merged single view ----------
  // Engine + Versions are one panel with no tab bar; the log lives in a
  // bottom drawer toggled by the bottom-right button below.

  // ---------- operation feed (terminal cell of the versions table) ----------
  function fmtTime(secs) {
    secs = Math.max(0, Math.floor(secs));
    const m = Math.floor(secs / 60),
      s = secs % 60;
    return `${m}:${s < 10 ? "0" : ""}${s}`;
  }
  // ---------- operation feeds: one reusable terminal per operation ----------
  // Every in-flight operation (parallel version downloads included) owns one
  // feed — head (npm command + elapsed) and terminal output — so concurrent
  // downloads never fight over a single output div. There is no stop/cancel
  // control: stuck npm operations are bounded by the run timeout.
  // Download-ish operations during npm phases show the header (command +
  // elapsed); pure message phases (registry checks, engine stop/start/
  // restart, port changes, notices) render as terminal info lines only.
  const downloadingOps = { download: 1, switch: 1, update: 1, rollback: 1, install: 1, select: 1 };
  const downloadingPhases = { installing: 1, verifying: 1 };
  // Ops that stop/start/restart the engine: while any runs, engine controls
  // are disabled. Plain downloads (kind "download") never touch the engine.
  const engineBusyOps = { switch: 1, update: 1, rollback: 1, engine: 1, port: 1 };
  const feeds = {}; // op key -> feed object
  const closedOps = {}; // op key -> true once the op reached a terminal phase
  // (done/failed/cancelled); late console lines for
  // closed ops must never re-create their feed
  let noticeFeed = null; // terminal surface for launcher-level notices
  let lastResultText = "",
    lastResultAt = 0;

  function opKind(op) {
    return String(op || "").split(":")[0];
  }

  function makeFeed(opKey) {
    const el = document.createElement("div");
    el.className = "op-feed idle";
    el.hidden = true;
    el.innerHTML =
      '<div class="op-feed-head" hidden>' +
      '<i class="ico sm bi" style="color:var(--accent2)" data-ico="terminal-fill"></i>' +
      '<span class="op-feed-cmd">Harness Launcher #</span>' +
      '<span class="op-feed-elapsed">0:00</span>' +
      "</div>" +
      '<div class="op-feed-body">' +
      '<pre class="term-out terminal" aria-live="polite"></pre>' +
      "</div>";
    sel.feedsBox.appendChild(el);
    loadIconsIn(el);
    const head = el.querySelector(".op-feed-head");
    const cmd = el.querySelector(".op-feed-cmd");
    const elapsedEl = el.querySelector(".op-feed-elapsed");
    const out = el.querySelector(".term-out");
    const feed = {
      opKey: opKey,
      el: el,
      downloading: false,
      done: false,
      start: null,
      lastPhase: null,
      elapsedTimer: null,
      stuckTimer: null,
      hideTimer: null,
      append: (stream, text) => {
        const ln = document.createElement("span");
        ln.className = `ln ${stream === "err" ? "err" : stream === "info" ? "info" : "out"}`;
        ln.textContent = text;
        out.appendChild(ln);
        while (out.childElementCount > 400) out.removeChild(out.firstChild);
        out.scrollTop = out.scrollHeight;
        el.hidden = false;
      },
      setHeader: (text) => {
        cmd.textContent = text;
        head.hidden = false;
        el.hidden = false;
      },
      setDownloading: (on) => {
        feed.downloading = on;
        el.classList.toggle("idle", !on);
        el.hidden = false;
        head.hidden = !on;
        if (!on) {
          if (feed.elapsedTimer) {
            clearInterval(feed.elapsedTimer);
            feed.elapsedTimer = null;
          }
          if (feed.stuckTimer) {
            clearTimeout(feed.stuckTimer);
            feed.stuckTimer = null;
          }
        } else {
          if (feed.hideTimer) {
            clearTimeout(feed.hideTimer);
            feed.hideTimer = null;
          }
          if (!feed.start) feed.start = Date.now();
          if (!feed.elapsedTimer) {
            feed.elapsedTimer = setInterval(() => {
              elapsedEl.textContent = fmtTime((Date.now() - feed.start) / 1000);
            }, 1000);
          }
        }
      },
      // After 60 s with no progress output, surface a hint in the terminal.
      armStuck: () => {
        if (feed.stuckTimer) clearTimeout(feed.stuckTimer);
        feed.stuckTimer = setTimeout(() => {
          if (feed.downloading) feed.append("info", t("stuckHint"));
        }, 60000);
      },
      // Keep the terminal visible briefly after the operation settles, then
      // animate the feed out and remove it — the container collapses. The
      // dismissal is scheduled EXACTLY ONCE: repeat terminal events (a
      // cancelled op re-reported on refresh) must not extend the wait, or a
      // stopped download would linger on screen.
      finishSoon: (ms) => {
        feed.done = true;
        feed.setDownloading(false);
        if (feed.hideTimer) return;
        feed.hideTimer = setTimeout(() => {
          if (feed.downloading) return;
          el.classList.add("leaving");
          setTimeout(() => {
            if (el.parentNode) el.parentNode.removeChild(el);
            delete feeds[opKey];
          }, 180);
        }, ms);
      },
    };
    feeds[opKey] = feed;
    return feed;
  }

  function feedFor(op) {
    return feeds[op] || makeFeed(op);
  }

  // Launcher-level notices (delete confirmations, engine results) surface in
  // a dedicated notice feed inside the feeds section below the versions
  // table, instead of a stray line under the install row.
  function setResult(text, isErr) {
    if (!text) return;
    const now = Date.now();
    // The same final message often arrives twice (progress event + command
    // result); appending both would duplicate the line.
    if (text === lastResultText && now - lastResultAt < 2500) {
      lastResultAt = now;
      return;
    }
    lastResultText = text;
    lastResultAt = now;
    let f = noticeFeed;
    if (!f || f.done) f = noticeFeed = makeFeed("__notice__");
    f.append(isErr ? "err" : "info", text);
    f.finishSoon(8000);
  }

  function busy(on) {
    [sel.toggle, sel.restart].forEach((b) => {
      b.disabled = on;
    });
  }

  function applyProgress(p) {
    if (!p) return;
    const kind = opKind(p.op);
    const f = feedFor(p.op || "");
    // Download-ish operations during download phases show the feed header
    // (npm command + elapsed timer); pure message phases (engine
    // stop/start/restart, port changes, verification-only runs, notices)
    // stream as terminal info lines only. There is no stop/cancel control.
    const downloading = !!downloadingOps[kind] && !!downloadingPhases[p.phase];
    if (downloading) {
      f.done = false;
      f.setDownloading(true);
      f.setHeader("Harness Launcher #");
    }
    // Milestone messages (registry → installing → verifying, stopping →
    // starting → running) stream in as info lines so the terminal tells the
    // whole story for engine-affecting operations too.
    if (p.message && p.phase !== f.lastPhase) {
      f.lastPhase = p.phase;
      if (downloading || engineBusyOps[kind]) f.append("info", p.message);
    }
    if (downloading && p.message) f.armStuck();
    if (engineBusyOps[kind]) busy(true);
    if (p.phase === "done" || p.phase === "failed" || p.phase === "cancelled") {
      closedOps[p.op || ""] = true;
      if (f.lastPhase !== p.phase) {
        f.lastPhase = p.phase;
        f.append(
          p.phase === "done" ? "info" : "err",
          p.message || (p.phase === "done" ? t("completed") : p.phase),
        );
      }
      f.finishSoon(p.phase === "done" ? 8000 : 10000);
      if (engineBusyOps[kind])
        setTimeout(() => {
          busy(false);
        }, 2500);
      setTimeout(refresh, 700);
    }
    if (p.phase === "running" || p.phase === "stopped") {
      setTimeout(() => {
        busy(false);
      }, 1200);
      if (p.phase === "stopped") setTimeout(refresh, 700);
    }
  }

  function closePopovers() {
    const ps = document.querySelectorAll(".pop.open");
    for (let i = 0; i < ps.length; i++) ps[i].classList.remove("open");
  }
  function renderVersions(s) {
    const list = (s.installedVersions || []).slice().sort();
    sel.verTbody.innerHTML = "";
    if (!list.length) {
      const empty = document.createElement("div");
      empty.className = "vrow";
      const emptyTxt = document.createElement("span");
      emptyTxt.className = "hint";
      emptyTxt.textContent = t("noVersions");
      empty.appendChild(emptyTxt);
      sel.verTbody.appendChild(empty);
      return;
    }
    list.forEach((v) => {
      const row = document.createElement("div");
      row.className = "vrow";
      const isActive = s.activeVersion === v;

      // Active column: a radio that selects this version as the active one.
      const defCell = document.createElement("span");
      defCell.className = "vc-def";
      const radio = document.createElement("input");
      radio.type = "radio";
      radio.name = "dsh-active-version";
      radio.className = "vc-radio";
      radio.checked = isActive;
      radio.disabled = isActive || !!s.currentOp;
      radio.title = isActive ? t("activeIs") : fmt("makeActive", { ver: v });
      radio.addEventListener("change", () => {
        if (radio.checked && !isActive) {
          withCmd(() => invoke("install_and_switch", { version: v }), t("doneSwitched"));
        }
      });
      defCell.appendChild(radio);

      const ver = document.createElement("span");
      ver.className = "vc-ver";
      ver.textContent = `v${v}`;

      // Directory column: opens the directory this version is installed in.
      const dirCell = document.createElement("span");
      dirCell.className = "vc-dir";
      const dirBtn = document.createElement("button");
      dirBtn.className = "btn dir";
      dirBtn.title = fmt("openDir", { ver: v });
      dirBtn.setAttribute("aria-label", fmt("openDir", { ver: v }));
      dirBtn.innerHTML = '<i class="ico sm bi" data-ico="folder2-open"></i>';
      dirBtn.addEventListener("click", () => {
        invoke("open_version_dir", { version: v }).catch((e) => {
          setResult(String(e?.message || e), true);
        });
      });
      dirCell.appendChild(dirBtn);

      // Actions column: circular red trash button with confirmation popover.
      const acts = document.createElement("span");
      acts.className = "vc-acts";
      const del = document.createElement("button");
      del.className = "btn del";
      del.title = t("delVer");
      del.disabled = isActive || !!s.currentOp;
      del.innerHTML = '<i class="ico sm bi" data-ico="trash-fill"></i>';
      const pop = document.createElement("div");
      pop.className = "pop";
      pop.setAttribute("role", "dialog");
      pop.setAttribute("aria-label", fmt("delAria", { ver: v }));
      // Built with DOM nodes (never innerHTML with the version string): `v`
      // comes from the registry version list and must render as inert text.
      const popTxt = document.createElement("div");
      popTxt.className = "pop-txt";
      popTxt.appendChild(document.createTextNode(t("delLead")));
      const popVer = document.createElement("b");
      popVer.textContent = `v${v}`;
      popTxt.appendChild(popVer);
      popTxt.appendChild(document.createTextNode(t("delTail")));
      const popActs = document.createElement("div");
      popActs.className = "pop-acts";
      const cancelBtn = document.createElement("button");
      cancelBtn.className = "btn";
      cancelBtn.setAttribute("data-act", "cancel");
      cancelBtn.textContent = t("cancel");
      const confirmBtn = document.createElement("button");
      confirmBtn.className = "btn danger";
      confirmBtn.setAttribute("data-act", "confirm");
      confirmBtn.textContent = t("delConfirm");
      popActs.appendChild(cancelBtn);
      popActs.appendChild(confirmBtn);
      pop.appendChild(popTxt);
      pop.appendChild(popActs);
      del.addEventListener("click", (e) => {
        e.stopPropagation();
        closePopovers();
        pop.classList.add("open");
        document.addEventListener("click", function outer(ev) {
          if (!pop.contains(ev.target) && ev.target !== del) {
            pop.classList.remove("open");
            document.removeEventListener("click", outer);
          }
        });
      });
      pop.querySelector('[data-act="cancel"]').addEventListener("click", () => {
        pop.classList.remove("open");
      });
      pop.querySelector('[data-act="confirm"]').addEventListener("click", () => {
        pop.classList.remove("open");
        invoke("delete_version", { version: v })
          .then(() => refresh())
          .catch((e) => {
            setResult(String(e?.message || e), true);
            return refresh();
          });
      });
      acts.appendChild(del);
      row.appendChild(defCell);
      row.appendChild(ver);
      row.appendChild(dirCell);
      row.appendChild(acts);
      row.appendChild(pop);
      sel.verTbody.appendChild(row);
    });
    // Icons created above use data-ico; let the icon loader pick them up.
    loadIconsIn(sel.verTbody);
  }

  function renderStatus(s) {
    const running = !!s.running;
    sel.appVer.textContent = s.launcherVersion || "";
    // Saved language wins: the panel converges here on every poll, so a
    // restart or an external settings edit still lands on the right locale.
    if (s.language) applyLanguage(s.language);

    const phase = s.enginePhase || (running ? "running" : "stopped");
    lastPhase = phase;

    // Status icon
    let iconCls = "stopped";
    let iconIco = "moon";
    if (phase === "running") {
      iconCls = "running";
      iconIco = "gear";
    } else if (phase === "stopped") {
      iconCls = "stopped";
      iconIco = "moon";
    } else {
      iconCls = "error";
      iconIco = "exclamation-circle-fill";
    }
    sel.stIcon.className = `ec-status-icon ${iconCls}`;
    sel.stIcon.innerHTML = `<i class="ico bi" data-ico="${iconIco}"></i>`;
    // innerHTML replacement discards any previously swapped-in SVG; re-run the
    // loader so the status icon renders on every status update.
    loadIconsIn(sel.stIcon);

    // Running state: show version as text, port as text
    // Stopped state: show version as dropdown, port as input
    const runningLike = phase !== "stopped";

    // The version dropdown is offered in BOTH states: while stopped it picks
    // the version to run next (set_version); while running it performs a live
    // switch — the backend stops the engine, installs/sets the version and
    // starts the new one again.
    sel.verWrap.innerHTML = "";
    sel.verWrap.appendChild(sel.engineVer);
    sel.verWrap.appendChild(sel.verChevron);
    sel.engineVer.style.display = "";
    sel.verChevron.style.display = "";
    sel.engineVer.disabled = false;
    sel.verWrap.style.cursor = "pointer";

    if (runningLike) {
      // Port as text
      sel.port.type = "text";
      sel.port.readOnly = true;
      sel.port.value = s.actualPort || 0;
    } else {
      // Port as input
      sel.port.type = "number";
      sel.port.readOnly = false;
      sel.port.disabled = false;
    }

    // Toggle button (icon only, circular). innerHTML resets the icon to a
    // placeholder <i>; re-run the loader or the Start/Stop glyph vanishes.
    if (runningLike) {
      sel.toggle.className = "ec-btn icon-stop";
      sel.toggle.innerHTML = '<i class="ico bi" data-ico="pause-fill"></i>';
      sel.toggle.title = t("stop");
    } else {
      sel.toggle.className = "ec-btn icon-start";
      sel.toggle.innerHTML = '<i class="ico bi" data-ico="play-fill"></i>';
      sel.toggle.title = t("start");
    }
    loadIconsIn(sel.toggle);
    sel.toggle.disabled = false;
    sel.restart.disabled = !runningLike;

    // version dropdown lists downloaded versions only
    const inst = s.installedVersions || [];
    const curV = s.activeVersion;
    sel.engineVer.innerHTML = "";
    inst.forEach((v) => {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = `v${v}`;
      sel.engineVer.appendChild(o);
    });
    if (curV && inst.indexOf(curV) !== -1) sel.engineVer.value = curV;

    if (!runningLike && sel.port.value === "") sel.port.value = String(s.port);

    if (s.updateAvailable && s.latestRemote) {
      sel.banner.style.display = "";
      sel.bannerText.textContent = fmt("updateAvail", {
        active: s.activeVersion || "?",
        latest: s.latestRemote,
      });
    } else {
      sel.banner.style.display = "none";
    }

    fillVersionSelects(s);
    renderVersions(s);

    // Rebuild feed state: every in-flight operation (parallel downloads
    // included) gets its own feed, then each feed's terminal is seeded ONCE
    // from the console snapshot so it tells the whole story from the start
    // (refresh polls every few seconds and must not duplicate lines).
    const activeOps = s.currentOps || (s.currentOp ? [s.currentOp] : []);
    activeOps.forEach((p) => {
      if (p) applyProgress(p);
    });
    if (!activeOps.length) busy(false);
    if (s.console?.length) {
      s.console.forEach((line) => {
        if (!line?.text || !line.op) return;
        const f = feeds[line.op];
        if (f && !f.seeded) {
          f.seeded = true;
          f.append(line.stream || "out", line.text);
        }
      });
    }
  }

  function fillVersionSelects(s) {
    const all = s.versions || [];
    const cur = sel.installSel.value;
    sel.installSel.innerHTML = "";
    (all.length ? all : s.installedVersions || []).forEach((v) => {
      const o = document.createElement("option");
      o.value = v;
      o.textContent = `v${v}${s.installedVersions && s.installedVersions.indexOf(v) !== -1 ? "  ✓" : ""}`;
      sel.installSel.appendChild(o);
    });
    // The dropdown defaults to the NEWEST published version — the backend
    // returns the list sorted ascending, so the last option is the latest.
    // When that version is not installed yet it renders without a ✓, so the
    // versions table instantly shows what's worth downloading next and a
    // separate "Get latest" button is unnecessary (the installed-versions
    // table + radio already cover switching the active version).
    const latest =
      s.latestRemote && all.indexOf(s.latestRemote) !== -1
        ? s.latestRemote
        : all.length
          ? all[all.length - 1]
          : null;
    if (cur && all.indexOf(cur) !== -1) {
      sel.installSel.value = cur;
      return;
    }
    if (latest && all.indexOf(latest) !== -1) {
      sel.installSel.value = latest;
      return;
    }
    if (s.activeVersion && all.indexOf(s.activeVersion) !== -1) sel.installSel.value = s.activeVersion;
  }

  function refresh() {
    return invoke("get_status")
      .then((s) => {
        renderStatus(s);
      })
      .catch((e) => {
        setResult(String(e?.message || e), true);
      });
  }

  function withCmd(fn, label) {
    fn()
      .then((res) => {
        setResult(res || `${label} ✓`, false);
        return refresh();
      })
      .catch((e) => {
        setResult(String(e?.message || e), true);
        return refresh();
      });
  }

  // Log lines render as coloured .ln spans — the same palette the operation
  // feeds use — and ANSI escapes from the harness are stripped.
  function logLineClass(line) {
    if (/error|fail(?:ed|ure)?|panic|exception|ENOENT|EADDR|exited with code/i.test(line)) return "err";
    if (
      /^\[launcher\]|dsh web:|engine (running|stopped|starting)|installed|updated|update available|checking/i.test(
        line,
      )
    )
      return "info";
    return "out";
  }
  function refreshLog() {
    invoke("tail_logs", { lines: 300 })
      .then((text) => {
        const pre = sel.logTail;
        const atBottom = (pre.scrollHeight - (pre.scrollTop + pre.clientHeight)) / pre.scrollHeight <= 0.01;
        // biome-ignore lint/suspicious/noControlCharactersInRegex: strips ANSI color codes from engine log output
        let raw = String(text || "").replace(/\u001b\[[0-9;]*m/g, "");
        // Censor auth tokens in log output (token=... query params and bearer tokens)
        raw = raw
          .replace(/token=[^\s&"')\]]+/gi, "token=[CENSORED]")
          .replace(/bearer\s+[a-zA-Z0-9_\-.]+/gi, "bearer [CENSORED]");
        const lines = raw.split("\n");
        pre.textContent = "";
        for (let i = 0; i < lines.length; i++) {
          if (!lines[i].trim()) continue;
          const ln = document.createElement("span");
          ln.className = `ln ${logLineClass(lines[i])}`;
          ln.textContent = lines[i] || " ";
          pre.appendChild(ln);
        }
        if (!pre.childElementCount) {
          const empty = document.createElement("span");
          empty.className = "ln info";
          empty.textContent = t("noLogs");
          pre.appendChild(empty);
        }
        if (followOn || atBottom) pre.scrollTop = pre.scrollHeight;
      })
      .catch(() => {});
  }

  // engine
  sel.engineVer.addEventListener("change", () => {
    const v = sel.engineVer.value;
    if (!v) return;
    // When stopped, selecting a version just records the choice (does not
    // auto-launch); press Start to run it. When running, switching hands over
    // live: the backend stops the engine, installs/sets the version and
    // starts the new one — no manual stop required.
    if (lastPhase !== "stopped") {
      withCmd(() => invoke("install_and_switch", { version: v }), "Switched");
    } else {
      withCmd(() => invoke("set_version", { version: v }), t("doneVersionSet"));
    }
  });
  sel.toggle.addEventListener("click", () => {
    if (lastPhase === "running") {
      withCmd(() => invoke("engine_stop", {}), t("doneStopped"));
    } else {
      withCmd(() => invoke("engine_start", {}), t("doneStarted"));
    }
  });
  sel.restart.addEventListener("click", () => {
    withCmd(() => invoke("engine_restart", {}), t("doneRestarted"));
  });

  // port: save on blur (and Enter). When stopped this only persists the port;
  // it does NOT auto-start the engine (handled in set_port on the backend).
  function applyPort() {
    const p = parseInt(sel.port.value, 10);
    if (!p || p < 1 || p > 65535) return setResult(t("validPort"), true);
    withCmd(() => invoke("set_port", { port: p }), t("donePortSet"));
  }
  sel.port.addEventListener("blur", applyPort);
  sel.port.addEventListener("keydown", (e) => {
    if (e.key === "Enter") sel.port.blur();
  });

  // versions table
  sel.btnInstall.addEventListener("click", () => {
    const v = sel.installSel.value;
    if (!v) return;
    // Download only: never switches the active version and never touches a
    // running engine — downloading works while the engine runs. Switching is
    // the versions-table radio / engine-card dropdown.
    withCmd(() => invoke("download_version", { version: v }), t("doneDownloaded"));
  });
  sel.installSel.addEventListener("change", () => {
    closePopovers();
  });
  // Manual version-list refresh: bypass the status-poll cache and re-fetch
  // the published list now. The button busy-disables while npm answers;
  // the repaint keeps the user's selection via fillVersionSelects.
  sel.btnRefreshVer.addEventListener("click", () => {
    sel.btnRefreshVer.disabled = true;
    invoke("refresh_versions", {})
      .then((res) => {
        setResult(res || t("refreshed"), false);
        return refresh();
      })
      .catch((e) => {
        setResult(String(e.message || e), true);
        return refresh();
      })
      .then(() => {
        sel.btnRefreshVer.disabled = false;
      });
  });
  sel.bannerBtn.addEventListener("click", () => {
    withCmd(() => invoke("update_to_latest", {}), t("doneUpdated"));
  });

  // logs
  // Log drawer toggle: bottom-right circular image-only button. The drawer
  // is hidden by default; the button shows/hides it (aria-pressed tracks state).
  sel.logToggle.addEventListener("click", () => {
    const open = sel.logDrawer.hidden;
    sel.logDrawer.hidden = !open;
    sel.logDrawer.classList.toggle("open", open);
    sel.logToggle.setAttribute("aria-pressed", open ? "true" : "false");
    sel.logToggle.title = open ? t("hideLog") : t("showLog");
    sel.logToggle.setAttribute("aria-label", sel.logToggle.title);
    if (open) refreshLog();
  });
  // Go to last line: jump to the end of the log and keep following it.
  sel.logFollow.addEventListener("click", () => {
    followOn = true;
    sel.logFollow.setAttribute("aria-pressed", "true");
    sel.logFollow.classList.add("pressed");
    refreshLog();
  });

  sel.browser.addEventListener("click", () => {
    invoke("open_in_browser", {}).catch((e) => {
      setResult(String(e.message || e), true);
    });
  });
  // Brand header (logo, app name, version): opens the project GitHub page.
  // preventDefault keeps the panel on dsh-ui:// instead of navigating away.
  $("brand-link").addEventListener("click", (e) => {
    e.preventDefault();
    invoke("open_repo_page", {}).catch((err) => {
      setResult(String(err.message || err), true);
    });
  });
  sel.quit.addEventListener("click", () => {
    invoke("quit_app", {});
  });
  // Language segments: delegated so the five buttons share one listener.
  // setLanguage persists via the backend; the next status poll repaints
  // from the canonical saved value.
  sel.langSeg.addEventListener("click", (e) => {
    const el = e?.target?.closest ? e.target.closest("button[data-lang]") : null;
    const l = el?.getAttribute ? el.getAttribute("data-lang") : null;
    if (l) setLanguage(l);
  });

  // events
  listen("launcher://progress", (p) => {
    if (p) applyProgress(p);
  });
  listen("launcher://console", (line) => {
    if (!line?.text) return;
    // Lines tagged with an operation stream into THAT operation's feed;
    // stray launcher-level lines (like delete confirmations) surface as
    // notifications instead of piling up invisibly.
    if (line.op && !closedOps[line.op]) {
      const f = feedFor(line.op);
      f.seeded = true; // live lines supersede the snapshot seed
      f.append(line.stream || "out", line.text);
      if (f.downloading) f.armStuck();
    } else if (!line.op) {
      setResult(line.text, line.stream === "err");
    }
  });
  listen("launcher://status", () => {
    refresh();
  });

  loadIconsIn(document);

  refresh();
  refreshLog();
  setInterval(refresh, 3000);
  setInterval(refreshLog, 5000);
})();
