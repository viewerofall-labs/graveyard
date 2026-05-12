const STATE_KV = "global_state";
const MASTER_KEY = "CHElc&3=5g";

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const authToken = request.headers.get("Authorization")?.split("Bearer ")[1];

    // Dashboard UI (GET / - no auth needed, login form will verify)
    if (url.pathname === "/" && request.method === "GET") {
      return new Response(getDashboardHTML(), {
        headers: { "Content-Type": "text/html" }
      });
    }

    // Check auth for all other routes (use master key)
    if (!authToken || authToken !== MASTER_KEY) {
      return new Response(JSON.stringify({ error: "Unauthorized" }), {
        status: 401,
        headers: { "Content-Type": "application/json" }
      });
    }

    // GET /state - fetch current state
    if (url.pathname === "/state" && request.method === "GET") {
      const state = await env[STATE_KV].get("config");
      return new Response(JSON.stringify(state ? JSON.parse(state) : {}), {
        headers: { "Content-Type": "application/json" }
      });
    }

    // POST /state - update state
    if (url.pathname === "/state" && request.method === "POST") {
      const body = await request.json();
      await env[STATE_KV].put("config", JSON.stringify(body));
      return new Response(JSON.stringify({ success: true }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    // POST /api/file-hub/lock - Lock file hub
    if (url.pathname === "/api/file-hub/lock" && request.method === "POST") {
      const state = await env[STATE_KV].get("config");
      const config = state ? JSON.parse(state) : { banner: { enabled: false, content: "", color: "warning" }, lockdown: { enabled: false, sites: [] } };
      config.lockdown = { enabled: true, sites: ["html-hub.joemomanugget.workers.dev"] };
      await env[STATE_KV].put("config", JSON.stringify(config));
      return new Response(JSON.stringify({ status: "locked", site: "html-hub.joemomanugget.workers.dev" }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    // POST /api/file-hub/unlock - Unlock file hub
    if (url.pathname === "/api/file-hub/unlock" && request.method === "POST") {
      const state = await env[STATE_KV].get("config");
      const config = state ? JSON.parse(state) : { banner: { enabled: false, content: "", color: "warning" }, lockdown: { enabled: false, sites: [] } };
      config.lockdown = { enabled: false, sites: [] };
      await env[STATE_KV].put("config", JSON.stringify(config));
      return new Response(JSON.stringify({ status: "unlocked", site: "html-hub.joemomanugget.workers.dev" }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    // POST /api/proxy/lock - Lock proxy
    if (url.pathname === "/api/proxy/lock" && request.method === "POST") {
      const state = await env[STATE_KV].get("config");
      const config = state ? JSON.parse(state) : { banner: { enabled: false, content: "", color: "warning" }, lockdown: { enabled: false, sites: [] } };
      config.lockdown = { enabled: true, sites: ["gh-proxy-test.joemomanugget.dev"] };
      await env[STATE_KV].put("config", JSON.stringify(config));
      return new Response(JSON.stringify({ status: "locked", site: "gh-proxy-test.joemomanugget.dev" }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    // POST /api/verify - Verify ACCESS password (for login)
    if (url.pathname === "/api/verify" && request.method === "POST") {
      const body = await request.json();
      const accessSecret = env.ACCESS;

      if (!accessSecret || body.password !== accessSecret) {
        return new Response(JSON.stringify({ error: "Invalid password" }), {
          status: 401,
          headers: { "Content-Type": "application/json" }
        });
      }

      return new Response(JSON.stringify({ success: true, token: MASTER_KEY }), {
        headers: { "Content-Type": "application/json" }
      });
    }

    return new Response("Not Found", { status: 404 });
  }
};

function getDashboardHTML() {
  return `<!DOCTYPE html>
  <html>
  <head>
  <title>Global Control Panel</title>
  <style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    background: #0a0010;
    color: #c792ea;
    font-family: Inconsolata, monospace;
    padding: 20px;
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .panel {
    max-width: 800px;
    margin: 0 auto;
    background: #1a1a2e;
    border: 2px solid #c792ea;
    border-radius: 8px;
    padding: 30px;
    width: 100%;
  }
  h1 { margin-bottom: 30px; text-align: center; color: #00e5c8; }
  .section { margin-bottom: 30px; padding-bottom: 20px; border-bottom: 1px solid #c792ea; }
  .section:last-child { border-bottom: none; }
  h3 { margin-bottom: 15px; color: #00e5c8; }
  label { display: block; margin: 10px 0; cursor: pointer; }
  input[type="text"], input[type="password"], textarea {
    background: #0a0010;
    color: #c792ea;
    border: 1px solid #c792ea;
    padding: 10px;
    width: 100%;
    margin: 8px 0;
    border-radius: 4px;
    font-family: Inconsolata, monospace;
    box-sizing: border-box;
  }
  input[type="checkbox"] {
    margin-right: 8px;
    width: 18px;
    height: 18px;
    cursor: pointer;
  }
  button {
    background: #c792ea;
    color: #0a0010;
    border: none;
    padding: 12px 20px;
    margin-top: 10px;
    margin-right: 10px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
    font-family: Inconsolata, monospace;
    font-size: 14px;
    width: 100%;
  }
  button:hover { background: #00e5c8; }
  .quick-cmd {
    background: #0a0010;
    border: 1px solid #00e5c8;
    padding: 10px;
    margin: 8px 0;
    border-radius: 4px;
    font-size: 12px;
    overflow-x: auto;
    cursor: pointer;
    word-break: break-all;
  }
  .quick-cmd:hover { background: #1a1a2e; }
  .site-toggle {
    display: flex;
    align-items: center;
    margin: 8px 0;
  }
  .status {
    margin-top: 15px;
    padding: 10px;
    border-radius: 4px;
    text-align: center;
  }
  .status.success { background: #00e5c8; color: #0a0010; }
  .status.error { background: #ff6b6b; color: #0a0010; }
  .load { margin-top: 20px; padding: 10px; background: #0a0010; border: 1px solid #c792ea; border-radius: 4px; }
  #loginForm { display: block; }
  #dashboard { display: none; }
  </style>
  </head>
  <body>
  <div class="panel">
  <!-- Login Form -->
  <div id="loginForm">
  <h1>🔐 Control Panel Login</h1>
  <div class="section">
  <label for="password">Password:</label>
  <input type="password" id="password" placeholder="Enter your password" />
  <button onclick="login()">Access</button>
  <div id="loginStatus"></div>
  </div>
  </div>

  <!-- Dashboard (hidden until login) -->
  <div id="dashboard">
  <h1>🌐 Global Control Panel</h1>

  <!-- Banner Section -->
  <div class="section">
  <h3>📢 Announcement Banner</h3>
  <label>
  <input type="checkbox" id="bannerEnabled" /> Enable Banner
  </label>
  <textarea id="bannerContent" placeholder="Banner message..." rows="3"></textarea>
  <input type="text" id="bannerColor" placeholder="Color (e.g., #ffaa44 or warning)" value="warning" />
  <button onclick="saveBanner()">Save Banner</button>
  <div id="bannerStatus"></div>
  </div>

  <!-- Lockdown Section -->
  <div class="section">
  <h3>🔒 Lockdown Control</h3>
  <div class="site-toggle">
  <label>
  <input type="checkbox" id="site1Lock" /> html-hub.joemomanugget.workers.dev
  </label>
  </div>
  <div class="site-toggle">
  <label>
  <input type="checkbox" id="site2Lock" /> gh-proxy-test.joemomanugget.dev
  </label>
  </div>
  <button onclick="saveLockdown()">Save Lockdown</button>
  <div id="lockdownStatus"></div>
  </div>

  <!-- Quick Curl Commands -->
  <div class="section">
  <h3>⚡ Quick Curl Commands</h3>
  <p style="font-size: 12px; color: #00e5c8; margin-bottom: 10px;">Click to copy:</p>
  <div class="quick-cmd" onclick="copyCmd(this)" title="Click to copy">
  curl -X POST https://devpanel.joemomanugget.workers.dev/api/file-hub/lock -H "Authorization: Bearer \${TOKEN}"
  </div>
  <div class="quick-cmd" onclick="copyCmd(this)" title="Click to copy">
  curl -X POST https://devpanel.joemomanugget.workers.dev/api/file-hub/unlock -H "Authorization: Bearer \${TOKEN}"
  </div>
  <div class="quick-cmd" onclick="copyCmd(this)" title="Click to copy">
  curl -X POST https://devpanel.joemomanugget.workers.dev/api/proxy/lock -H "Authorization: Bearer \${TOKEN}"
  </div>
  </div>

  <!-- Load State -->
  <div class="load">
  <button onclick="loadState()" style="width: 100%; background: #00e5c8;">Reload Current State</button>
  </div>
  </div>
  </div>

  <script>
  let TOKEN = null;
  const PANEL_URL = "https://devpanel.joemomanugget.workers.dev";
  const STORED_TOKEN_KEY = "devpanel_token";

  // Check if already logged in
  window.addEventListener('load', async () => {
    const stored = localStorage.getItem(STORED_TOKEN_KEY);
    if (!stored) return;
    TOKEN = stored;
    try {
      const res = await fetch(\`\${PANEL_URL}/state\`, {
        headers: { "Authorization": \`Bearer \${TOKEN}\` }
      });
      if (!res.ok) {
        localStorage.removeItem(STORED_TOKEN_KEY);
        TOKEN = null;
        return;
      }
      showDashboard();
      loadState();
    } catch {
      localStorage.removeItem(STORED_TOKEN_KEY);
      TOKEN = null;
    }
  });

  async function login() {
    const password = document.getElementById('password').value;
    if (!password) {
      showLoginStatus('error', '✗ Please enter a password');
      return;
    }

    // Verify password against server (ACCESS secret validation)
    try {
      const res = await fetch(\`\${PANEL_URL}/api/verify\`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ password: password })
      });

      if (res.ok) {
        const data = await res.json();
        TOKEN = data.token;
        localStorage.setItem(STORED_TOKEN_KEY, TOKEN);
        showDashboard();
        loadState();
      } else {
        localStorage.removeItem(STORED_TOKEN_KEY);
        showLoginStatus('error', '✗ Wrong password');
        document.getElementById('password').value = '';
      }
    } catch (e) {
      showLoginStatus('error', '✗ Connection error');
    }
  }

  function showLoginStatus(type, message) {
    const el = document.getElementById('loginStatus');
    el.className = \`status \${type}\`;
    el.textContent = message;
    if (type === 'error') {
      setTimeout(() => { el.textContent = ''; el.className = ''; }, 3000);
    }
  }

  function showDashboard() {
    document.getElementById('loginForm').style.display = 'none';
    document.getElementById('dashboard').style.display = 'block';
  }

  function copyCmd(el) {
    const text = el.textContent.trim().replace('\${TOKEN}', TOKEN);
    navigator.clipboard.writeText(text).then(() => {
      const old = el.textContent;
      el.textContent = '✓ Copied!';
      setTimeout(() => { el.textContent = old; }, 2000);
    });
  }

  async function loadState() {
    try {
      const res = await fetch(\`\${PANEL_URL}/state\`, {
        headers: { "Authorization": \`Bearer \${TOKEN}\` }
      });
      if (!res.ok) throw new Error("Failed to load state");
      const state = await res.json();

      document.getElementById('bannerEnabled').checked = state.banner?.enabled || false;
      document.getElementById('bannerContent').value = state.banner?.content || '';
      document.getElementById('bannerColor').value = state.banner?.color || 'warning';
      document.getElementById('site1Lock').checked = (state.lockdown?.sites || []).includes('html-hub.joemomanugget.workers.dev');
      document.getElementById('site2Lock').checked = (state.lockdown?.sites || []).includes('gh-proxy-test.joemomanugget.dev');
    } catch (err) {
      showStatus('bannerStatus', 'error', \`Load failed: \${err.message}\`);
    }
  }

  async function saveBanner() {
    try {
      const data = {
        banner: {
          enabled: document.getElementById('bannerEnabled').checked,
          content: document.getElementById('bannerContent').value,
          color: document.getElementById('bannerColor').value
        }
      };
      const res = await fetch(\`\${PANEL_URL}/state\`, {
        method: 'POST',
        headers: {
          "Authorization": \`Bearer \${TOKEN}\`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify(data)
      });
      if (!res.ok) throw new Error("Save failed");
      showStatus('bannerStatus', 'success', '✓ Banner saved');
    } catch (err) {
      showStatus('bannerStatus', 'error', \`✗ \${err.message}\`);
    }
  }

  async function saveLockdown() {
    try {
      const sites = [];
      if (document.getElementById('site1Lock').checked) sites.push('html-hub.joemomanugget.workers.dev');
      if (document.getElementById('site2Lock').checked) sites.push('gh-proxy-test.joemomanugget.dev');
      const data = {
        lockdown: {
          enabled: sites.length > 0,
          sites: sites
        }
      };
      const res = await fetch(\`\${PANEL_URL}/state\`, {
        method: 'POST',
        headers: {
          "Authorization": \`Bearer \${TOKEN}\`,
          "Content-Type": "application/json"
        },
        body: JSON.stringify(data)
      });
      if (!res.ok) throw new Error("Save failed");
      showStatus('lockdownStatus', 'success', '✓ Lockdown saved');
    } catch (err) {
      showStatus('lockdownStatus', 'error', \`✗ \${err.message}\`);
    }
  }

  function showStatus(elementId, type, message) {
    const el = document.getElementById(elementId);
    el.className = \`status \${type}\`;
    el.textContent = message;
    setTimeout(() => { el.textContent = ''; el.className = ''; }, 3000);
  }

  // Allow Enter key to login
  document.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && document.getElementById('loginForm').style.display !== 'none') {
      login();
    }
  });
  </script>
  </body>
  </html>`;
}
