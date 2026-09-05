// DeepSeek Harness Launcher overlay — floating control panel, bottom-left of the harness WebUI.
// Injected at documentStart by the Rust shell (initialization_script).
(function () {
  'use strict';
  if (window.__DSH_LAUNCHER_OVERLAY__) return;
  window.__DSH_LAUNCHER_OVERLAY__ = true;
  // Do not inject into launcher-owned pages (Control Panel / stopped page).
  if (window.location.protocol === 'dsh-ui:') return;

  function invoke(cmd, args) {
    const core = window.__TAURI__ && window.__TAURI__.core;
    if (core && typeof core.invoke === 'function') return core.invoke(cmd, args || {});
    if (window.__TAURI_INTERNALS__ && typeof window.__TAURI_INTERNALS__.invoke === 'function')
      return window.__TAURI_INTERNALS__.invoke(cmd, args || {}, undefined);
    return Promise.reject(new Error('Tauri IPC unavailable in this context'));
  }

  const css = `
  #dsh-lc { all: initial; position: fixed; left: 12px; bottom: 12px; z-index: 2147483647;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    color: #e6edf3; }
  #dsh-lc * { box-sizing: border-box; }
  #dsh-lc-toggle { display: flex; align-items: center; gap: 8px;
    background: rgba(15, 23, 42, 0.92); border: 1px solid #334155; border-radius: 999px;
    padding: 8px 14px; cursor: pointer; font-size: 13px; color: #e6edf3;
    box-shadow: 0 4px 14px rgba(0,0,0,0.35); user-select: none; }
  #dsh-lc-toggle:hover { background: rgba(30, 41, 59, 0.95); }
  #dsh-lc-dot { width: 9px; height: 9px; border-radius: 50%; background: #f59e0b; }
  #dsh-lc-dot.ok { background: #22c55e; } #dsh-lc-dot.err { background: #ef4444; }
  #dsh-lc-panel { display: none; position: absolute; left: 0; bottom: 48px; width: 340px;
    max-height: 70vh; overflow: auto; background: rgba(15, 23, 42, 0.97);
    border: 1px solid #334155; border-radius: 12px; padding: 14px;
    box-shadow: 0 12px 40px rgba(0,0,0,0.5); font-size: 12.5px; line-height: 1.45; }
  #dsh-lc-panel.open { display: block; }
  #dsh-lc h3 { margin: 12px 0 6px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.8px; color: #94a3b8; }
  #dsh-lc h3:first-child { margin-top: 0; }
  #dsh-lc .row { display: flex; align-items: center; gap: 8px; margin: 6px 0; flex-wrap: wrap; }
  #dsh-lc .big { font-weight: 600; }
  #dsh-lc select, #dsh-lc input[type=text], #dsh-lc input[type=number] {
    background: #0f172a; color: #e6edf3; border: 1px solid #475569; border-radius: 6px;
    padding: 5px 8px; font-size: 12.5px; min-width: 0; }
  #dsh-lc button { background: #2563eb; border: none; color: #fff; border-radius: 6px;
    padding: 6px 12px; font-size: 12.5px; cursor: pointer; }
  #dsh-lc button:hover { background: #1d4ed8; }
  #dsh-lc button:disabled { opacity: 0.5; cursor: default; }
  #dsh-lc button.ghost { background: transparent; border: 1px solid #475569; color: #cbd5e1; }
  #dsh-lc button.danger { background: #dc2626; }
  #dsh-lc .hint { color: #94a3b8; font-size: 11px; }
  #dsh-lc .err { color: #f87171; }
  #dsh-lc .ok2 { color: #4ade80; }
  #dsh-lc pre { background: #0b1220; border: 1px solid #1e293b; border-radius: 8px; padding: 8px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 11px;
    max-height: 180px; overflow: auto; white-space: pre-wrap; word-break: break-all; color: #a5b4fc; }
  #dsh-lc a { color: #60a5fa; }
  #dsh-lc .meta { display: flex; justify-content: space-between; align-items: center; }
  #dsh-lc .spin { display: inline-block; width: 10px; height: 10px; border: 2px solid #64748b;
    border-top-color: #fff; border-radius: 50%; animation: dsh-lc-spin 0.8s linear infinite; }
  @keyframes dsh-lc-spin { to { transform: rotate(360deg); } }
  #dsh-lc .term { display: none; margin-top: 8px; border: 1px solid #12233c; border-radius: 8px;
    background: #04080f; overflow: hidden; }
  #dsh-lc .term.on { display: block; }
  #dsh-lc .term-head { display: flex; align-items: center; gap: 6px; padding: 5px 9px;
    background: #0a1322; border-bottom: 1px solid #12233c; font-size: 10px; color: #8ba0bd; }
  #dsh-lc .term-title { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; white-space: nowrap;
    overflow: hidden; text-overflow: ellipsis; }
  #dsh-lc .term-body { position: relative; }
  #dsh-lc .term-out { max-height: 160px; overflow: auto; margin: 0; padding: 8px 9px 42px;
    font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 10.5px; line-height: 1.5;
    white-space: pre-wrap; word-break: break-all; background: transparent; }
  #dsh-lc .term-out .ln { display: block; }
  #dsh-lc .term-out .ln.out { color: #a5e3ff; }
  #dsh-lc .term-out .ln.info { color: #8ecdf7; font-weight: 600; }
  #dsh-lc .term-out .ln.err { color: #ff8a8a; }
  #dsh-lc button.sm { padding: 2px 8px; font-size: 11px; }
  `;

  const styleEl = document.createElement('style');
  styleEl.textContent = css;
  document.head.appendChild(styleEl);

  const rootEl = document.createElement('div');
  rootEl.id = 'dsh-lc';
  rootEl.innerHTML = `
    <div id="dsh-lc-panel">
      <div class="meta"><span class="big">Harness Launcher</span><span id="lc-ver" class="hint"></span></div>
      <div class="hint" style="margin:4px 0 0">Status display — engine actions live in the Control Panel (menu bar → Control Panel…).</div>
      <h3>Status</h3>
      <div class="row">
        <span id="lc-state" class="hint">loading…</span>
        <span id="lc-active" class="big"></span>
        <a id="lc-url" href="#" target="_blank" rel="noreferrer"></a>
      </div>
      <div id="lc-prog" style="display:none;margin-top:6px">
        <div class="row" style="margin:0 0 4px"><span class="spin"></span>
          <span id="lc-progphase" class="hint"></span><span id="lc-progmsg" class="hint"></span></div>
        <div style="height:6px;background:#1e293b;border-radius:3px;overflow:hidden">
          <div id="lc-progbar" style="height:100%;width:0%;background:#2563eb;transition:width .25s ease"></div>
        </div>
      </div>
      <div class="term" id="lc-term">
        <div class="term-head"><span class="term-title" id="lc-termtitle">Installing DeepSeek Harness version@…</span></div>
        <div class="term-body">
          <pre class="term-out" id="lc-termout"></pre>
        </div>
      </div>
      <h3>Version</h3>
      <div class="row">
        <select id="lc-versions" style="flex:1"></select>
        <button id="lc-install">Install &amp; Switch</button>
      </div>
      <div class="row">
        <button id="lc-delete" class="danger sm">Delete selected</button>
        <span id="lc-deletehint" class="hint"></span>
      </div>
      <div class="row">
        <button id="lc-update" class="ghost">Update to latest</button>
        <button id="lc-rollback" class="ghost">Rollback</button>
        <label class="hint" style="display:flex;align-items:center;gap:4px">
          <input type="checkbox" id="lc-prerelease"> Pre-release
        </label>
      </div>
      <h3>Port (localhost)</h3>
      <div class="row">
        <input id="lc-port" type="number" min="1" max="65535" style="width:90px" title="Desired loopback port">
        <button id="lc-apply-port" class="ghost">Apply</button>
        <span id="lc-actual" class="hint"></span>
      </div>
      <h3>Updates</h3>
      <div class="row">
        <label style="display:flex;align-items:center;gap:4px">
          <input type="checkbox" id="lc-autoupdate"> Auto-check every
        </label>
        <input id="lc-interval" type="number" min="1" max="720" style="width:64px"> h
      </div>
      <div class="row"><button id="lc-check" class="ghost">Check for updates now</button>
        <span id="lc-checkresult" class="hint"></span></div>
      <h3>Logs</h3>
      <div class="row"><button id="lc-logrefresh" class="ghost">Refresh tail</button>
        <button id="lc-logtoggle" class="ghost">Show/Hide</button></div>
      <pre id="lc-logs" style="display:none"></pre>
      <div id="lc-msg" class="hint"></div>
      <h3>Engine</h3>
      <div class="row">
        <button id="lc-start" class="ghost">Start</button>
        <button id="lc-stop" class="ghost">Stop</button>
        <button id="lc-settings" class="ghost" style="margin-left:auto">Settings…</button>
      </div>
      <div class="row" style="margin-top:10px">
        <button id="lc-browser" class="ghost">Open in Browser</button>
        <button id="lc-restart" class="ghost">Restart Harness</button>
        <button id="lc-quit" class="danger" style="margin-left:auto">Quit</button>
      </div>
    </div>
    <div id="dsh-lc-toggle" title="Harness Launcher controls">
      <span id="dsh-lc-dot"></span><span>Harness</span>
    </div>
  `;
  document.body.appendChild(rootEl);

  const $ = (id) => rootEl.querySelector('#' + id);
  const panel = rootEl.querySelector('#dsh-lc-panel');
  const toggle = rootEl.querySelector('#dsh-lc-toggle');
  const dot = rootEl.querySelector('#dsh-lc-dot');
  const msg = $('lc-msg');
  let logsVisible = false;
  let pollTimer = null;

  function setMsg(text, isErr) { msg.textContent = text || ''; msg.className = isErr ? 'err' : (text ? 'hint' : 'hint'); }
  function busy(btn, on) { if (btn) btn.disabled = on; }

  function stateDot(ok) { dot.className = ok ? 'ok' : 'err'; }

  var downloadingOps = { install: 1, update: 1, rollback: 1 };
  var downloadingPhases = { registry: 1, installing: 1, verifying: 1 };
  var termOpKey = null;

  function isDownloading(p) { return !!(p && downloadingOps[p.op] && downloadingPhases[p.phase]); }

  function showTerminal(show, op) {
    $('lc-term').classList.toggle('on', !!show);
    if (show && op) {
      $('lc-termtitle').textContent = 'Installing DeepSeek Harness version@' + (op.version || '…');
      $('lc-termout').innerHTML = '';
    }
  }
  function appendTermLine(stream, text) {
    var t = $('lc-term');
    if (!t.classList.contains('on')) return;
    var ln = document.createElement('span');
    ln.className = 'ln ' + (stream === 'err' ? 'err' : stream === 'info' ? 'info' : 'out');
    ln.textContent = text;
    var out = $('lc-termout');
    out.appendChild(ln);
    while (out.childElementCount > 400) out.removeChild(out.firstChild);
    out.scrollTop = out.scrollHeight;
  }

  function applyProgress(p) {
    if (!p) {
      $('lc-prog').style.display = 'none';
      showTerminal(false);
      return;
    }
    $('lc-prog').style.display = 'block';
    $('lc-progphase').textContent = p.phase || '';
    $('lc-progmsg').textContent = p.message || '';
    if (typeof p.percent === 'number') $('lc-progbar').style.width = Math.max(0, Math.min(100, p.percent)) + '%';
    if (isDownloading(p)) {
      var key = p.op + ':' + (p.version || '');
      if (termOpKey !== key) { termOpKey = key; showTerminal(true, p); }
      else { showTerminal(true); }
    }
    else if (p.phase === 'done' || p.phase === 'failed' || p.phase === 'cancelled') {
      setTimeout(function () { showTerminal(false); }, 2500);
    } else if (!downloadingOps[p.op]) {
      showTerminal(false);
    }
  }

  async function refreshStatus() {
    try {
      const s = await invoke('get_status');
      const ep = s.enginePhase || (s.running ? 'running' : 'stopped');
      $('lc-state').textContent = ep;
      stateDot(ep === 'running');
      $('lc-start').disabled = ep !== 'stopped';
      // Least privilege: the harness window may not mutate versions, engine
      // state (beyond start), ports or the app — those commands require the
      // Control Panel. Reflect that here so buttons never promise otherwise.
      var ro = 'Available in the Control Panel';
      ['lc-install', 'lc-delete', 'lc-update', 'lc-rollback',
       'lc-apply-port', 'lc-port', 'lc-autoupdate', 'lc-interval',
       'lc-stop', 'lc-restart', 'lc-quit'].forEach(function (id) {
        var b = $(id);
        if (b) { b.disabled = true; b.title = ro; }
      });
      applyProgress(s.currentOp || null);
      // Window opened mid-download: seed the terminal with the snapshot.
      if (isDownloading(s.currentOp) && !$('lc-termout').childElementCount && s.console && s.console.length) {
        showTerminal(true, s.currentOp);
        s.console.forEach(function (line) { appendTermLine(line.stream || 'out', line.text); });
      }
      $('lc-active').textContent = s.activeVersion ? 'v' + s.activeVersion : '(none)';
      $('lc-ver').textContent = s.launcherVersion ? 'app v' + s.launcherVersion : '';
      if (s.actualPort) {
        // Prefer the authenticated URL the engine printed (token engines
        // 401 on the plain URL); fall back to constructing it.
        const url = s.webUrl || ('http://127.0.0.1:' + s.actualPort);
        const a = $('lc-url'); a.textContent = url.replace('http://', ''); a.href = url;
        $('lc-actual').textContent = 'in use: ' + s.actualPort + (s.portChanged ? ' (fallback)' : '');
        if ($('lc-port').value === '') $('lc-port').value = String(s.port);
      }
      if (s.updateAvailable) {
        setMsg('Update available: v' + s.latestRemote + ' (' + (s.activeVersion || '?') + ' -> ' + s.latestRemote + ')', false);
      }
      fillVersions(s);
      fillSettings(s);
      populatePreRelease(s.includePrerelease);
      populateAutoUpdate(s.autoUpdateHarness, s.autoUpdateIntervalHours);
      const rb = $('lc-rollback'); rb.disabled = !s.previousVersion;
      rb.title = s.previousVersion ? 'Rollback to v' + s.previousVersion : 'No previous version installed';
      updateDeleteState(s);
    } catch (e) {
      stateDot(false);
      $('lc-state').textContent = 'error';
      setMsg(String(e && e.message || e), true);
    }
  }

  function fillSettings(s) {
    if ($('lc-port').value === '' && s.port) $('lc-port').value = String(s.port);
  }
  function populatePreRelease(on) { $('lc-prerelease').checked = !!on; }
  function populateAutoUpdate(on, hours) { $('lc-autoupdate').checked = !!on; if (hours) $('lc-interval').value = String(hours); }

  // Track the selected version so the Delete button is only usable on an
  // installed, non-active version.
  var selMeta = { installed: false, active: false, version: null };
  function updateDeleteState(s) {
    var v = $('lc-versions').value;
    selMeta.version = v;
    selMeta.installed = !!(s.installedVersions && s.installedVersions.indexOf(v) !== -1);
    selMeta.active = s.activeVersion === v;
    var del = $('lc-delete');
    var hint = $('lc-deletehint');
    if (s.currentOp) { del.disabled = true; hint.textContent = 'busy — try again after the operation finishes'; }
    else if (!v) { del.disabled = true; hint.textContent = 'select a version'; }
    else if (selMeta.active) { del.disabled = true; hint.textContent = 'the active version cannot be deleted'; }
    else if (!selMeta.installed) { del.disabled = true; hint.textContent = 'v' + v + ' is not installed'; }
    else { del.disabled = false; hint.textContent = 'removes the local install of v' + v; }
    del.removeAttribute('data-arm');
    del.textContent = 'Delete selected';
  }

  let remoteVersions = [];
  function fillVersions(s) {
    const sel = $('lc-versions');
    const all = s.versions || [];
    remoteVersions = all;
    const currentSel = sel.value;
    sel.innerHTML = '';
    const opts = all.map(function (v) {
      const o = document.createElement('option');
      o.value = v; o.textContent = 'v' + v + (s.installedVersions && s.installedVersions.indexOf(v) !== -1 ? ' (installed)' : '');
      return o;
    });
    opts.forEach(function (o) { sel.appendChild(o); });
    if (currentSel && all.indexOf(currentSel) !== -1) sel.value = currentSel;
    else if (s.latestRemote && all.indexOf(s.latestRemote) !== -1) sel.value = s.latestRemote;
  }

  async function withBusy(btn, fn) {
    busy(btn, true);
    try {
      const res = await fn();
      setMsg(res || 'done', false);
      await refreshStatus();
      return res;
    } catch (e) {
      setMsg(String(e && e.message || e), true);
    } finally {
      busy(btn, false);
    }
  }

  toggle.addEventListener('click', function () {
    panel.classList.toggle('open');
    refreshStatus();
    if (panel.classList.contains('open')) {
      pollTimer = setInterval(refreshStatus, 4000);
    } else if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  });

  $('lc-install').addEventListener('click', function () {
    const v = $('lc-versions').value;
    if (!v) return setMsg('select a version', true);
    withBusy(this, function () { return invoke('install_and_switch', { version: v }); });
  });

  $('lc-update').addEventListener('click', function () {
    withBusy(this, function () { return invoke('update_to_latest', {}); });
  });

  $('lc-rollback').addEventListener('click', function () {
    withBusy(this, function () { return invoke('rollback', {}); });
  });

  // Delete the currently selected version (two-step confirm).
  $('lc-delete').addEventListener('click', function () {
    var v = $('lc-versions').value;
    if (!v) return setMsg('select a version first', true);
    if (this.getAttribute('data-arm') !== '1') {
      this.setAttribute('data-arm', '1');
      this.textContent = 'Confirm delete v' + v + '?';
      setTimeout(function () { $('lc-delete').removeAttribute('data-arm'); $('lc-delete').textContent = 'Delete selected'; }, 3500);
      return;
    }
    var btn = this;
    btn.removeAttribute('data-arm');
    btn.textContent = 'Delete selected';
    btn.disabled = true;
    invoke('delete_version', { version: v }).then(function (res) {
      setMsg((res && res.message) || 'version deleted', false);
      return refreshStatus();
    }).catch(function (e) {
      setMsg(String(e && e.message || e), true);
    }).finally(function () { btn.disabled = false; });
  });

  $('lc-apply-port').addEventListener('click', function () {
    const port = parseInt($('lc-port').value, 10);
    if (!port || port < 1 || port > 65535) return setMsg('invalid port', true);
    withBusy(this, function () { return invoke('set_port', { port: port }); });
  });

  $('lc-prerelease').addEventListener('change', function () {
    invoke('set_prerelease', { include: this.checked }).then(refreshStatus).catch(function (e) { setMsg(String(e.message || e), true); });
  });

  $('lc-autoupdate').addEventListener('change', function () {
    invoke('set_auto_update', { enabled: this.checked }).catch(function (e) { setMsg(String(e.message || e), true); });
  });

  $('lc-interval').addEventListener('change', function () {
    const h = parseInt(this.value, 10);
    if (h >= 1) invoke('set_auto_update', { intervalHours: h }).catch(function (e) { setMsg(String(e.message || e), true); });
  });

  $('lc-check').addEventListener('click', function () {
    withBusy(this, function () { return invoke('check_updates', {}); });
  });

  $('lc-logrefresh').addEventListener('click', function () {
    busy(this, true);
    invoke('tail_logs', { lines: 200 }).then(function (text) {
      $('lc-logs').textContent = text || '(no logs yet)';
    }).catch(function (e) { setMsg(String(e.message || e), true); }).finally(function () { busy($('lc-logrefresh'), false); });
  });
  $('lc-logtoggle').addEventListener('click', function () {
    logsVisible = !logsVisible;
    $('lc-logs').style.display = logsVisible ? 'block' : 'none';
  });

  $('lc-browser').addEventListener('click', function () { invoke('open_in_browser', {}).catch(function (e) { setMsg(String(e.message || e), true); }); });
  $('lc-restart').addEventListener('click', function () { withBusy(this, function () { return invoke('restart_harness', {}); }); });
  $('lc-quit').addEventListener('click', function () { invoke('quit_app', {}); });

  $('lc-start').addEventListener('click', function () { withBusy(this, function () { return invoke('engine_start', {}); }); });
  $('lc-stop').addEventListener('click', function () { withBusy(this, function () { return invoke('engine_stop', {}); }); });
  $('lc-settings').addEventListener('click', function () { invoke('open_settings', {}).catch(function (e) { setMsg(String(e.message || e), true); }); });

  var lcEvents = window.__TAURI__ && window.__TAURI__.event;
  if (lcEvents && typeof lcEvents.listen === 'function') {
    lcEvents.listen('launcher://progress', function (e) { applyProgress(e.payload || null); }).catch(function () {});
    lcEvents.listen('launcher://console', function (e) {
      var line = e && e.payload;
      if (line && line.text) appendTermLine(line.stream || 'out', line.text);
    }).catch(function () {});
    lcEvents.listen('launcher://status', function () { refreshStatus(); }).catch(function () {});
  }

  $('lc-versions').addEventListener('change', function () { refreshStatus(); });

  document.addEventListener('keydown', function (e) { if (e.key === 'Escape') panel.classList.remove('open'); });
})();
