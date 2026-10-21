"use strict";
const $ = (s) => document.querySelector(s),
  esc = (s) =>
    String(s ?? "").replace(
      /[&<>"']/g,
      (c) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          '"': "&quot;",
          "'": "&#39;",
        })[c],
    );
let state = {},
  selectedPath = "",
  wizard = { name: "", template: "lehre-informatik", path: "" };
async function api(path, data) {
  const r = await fetch(path, {
    method: data === undefined ? "GET" : "POST",
    headers: {
      "Content-Type": "application/json",
      "x-csrf-token": state.csrf || "",
    },
    body: data === undefined ? undefined : JSON.stringify(data),
  });
  const body = await r.text();
  let value;
  try {
    value = JSON.parse(body);
  } catch {
    if (!r.ok) throw Error(body || "Anfrage fehlgeschlagen");
    throw Error("Unerwartete Antwort vom Server");
  }
  if (!r.ok) throw Error(value.error || "Anfrage fehlgeschlagen");
  return value;
}
function notice(e, ok = false) {
  $("#notice").className = ok ? "success" : "error";
  $("#notice").textContent = e;
  window.scrollTo(0, 0);
}
async function action(fn) {
  try {
    await fn();
  } catch (e) {
    notice(e.message);
  }
}
function button(label, fn, secondary = false) {
  const b = document.createElement("button");
  b.textContent = label;
  if (secondary) b.className = "secondary";
  b.onclick = () => action(fn);
  return b;
}
function confirmAction(text) {
  return new Promise((resolve) => {
    const d = $("#confirm");
    $("#confirm-text").textContent = text;
    d.showModal();
    $("#confirm-yes").onclick = () => {
      d.close();
      resolve(true);
    };
    d.oncancel = (e) => {
      e.preventDefault();
      d.close();
      resolve(false);
    };
    $("#confirm-no").onclick = () => {
      d.close();
      resolve(false);
    };
  });
}
const pages = [
  ["/", "Start"],
  ["/unterlagen", "Unterlagen"],
  ["/kapitel", "Kapitel"],
  ["/chat", "Chat"],
  ["/suche", "Suche"],
  ["/warteschlange", "Warteschlange"],
  ["/einstellungen", "Einstellungen"],
  ["/sicherung", "Sicherung"],
  ["/protokoll", "Protokoll"],
];
function nav() {
  $("#nav").innerHTML = pages
    .map(
      ([p, t]) =>
        `<a href="${p}" class="${location.pathname === p ? "active" : ""}">${t}</a>`,
    )
    .join("");
}
async function browse(path = "") {
  const d = await api("/api/browse?path=" + encodeURIComponent(path));
  wizard.path = d.path;
  $("#folder-path").value = d.path;
  $("#directories").replaceChildren();
  if (d.parent)
    $("#directories").append(
      button("↑ Übergeordneter Ordner", () => browse(d.parent), true),
    );
  for (const dir of d.directories)
    $("#directories").append(button(dir, () => browse(dir), true));
}
async function setup() {
  let step = 0;
  function render() {
    const c = $("#content");
    c.innerHTML = `<h1>Willkommen bei Folder-Mind</h1><p class="step">Einrichtung · Schritt ${step + 1} von 6</p><div class="card" id="wizard"></div><div class="row" id="wizard-nav"></div>`;
    const w = $("#wizard");
    if (step === 0)
      w.innerHTML = `<h2>Sprache und Name</h2><p>Folder-Mind spricht Deutsch und läuft auf deinem Gerät.</p><label>Dein Name (optional)<input id="name" value="${esc(wizard.name)}"></label>`;
    if (step === 1) {
      w.innerHTML =
        '<h2>Datenordner wählen</h2><p>Nur in diesem Ordner verwaltet Folder-Mind deine Lernunterlagen.</p><label>Gewählter Ordner<input id="folder-path"></label><div class="row" id="folder-actions"></div><div id="directories" class="grid"></div>';
      $("#folder-actions").append(
        button("Pfad öffnen", () => browse($("#folder-path").value)),
        button(
          "Neuen Ordner anlegen",
          async () => {
            const name = prompt("Name des neuen Ordners");
            if (name) {
              await api("/api/browse/create", {
                path: $("#folder-path").value,
                name,
              });
              await browse($("#folder-path").value + "/" + name);
            }
          },
          true,
        ),
      );
      action(() => browse(wizard.path));
    }
    if (step === 2)
      w.innerHTML = `<h2>Ordnerstruktur</h2><label>Vorlage<select id="template">${["lehre-informatik", "lehre-allgemein", "schule", "frei"].map((v, i) => `<option value="${v}" ${wizard.template === v ? "selected" : ""}>${["Lehre Informatik", "Lehre allgemein", "Schule", "Frei"][i]}</option>`).join("")}</select></label>`;
    if (step === 3) {
      w.innerHTML =
        "<h2>KI-Modell verbinden (optional)</h2>" + aiForm(wizard.config || {});
      bindProvider();
      $("#api-key").value = wizard.api_key || "";
      $("#test-connection").onclick = () =>
        action(async () => {
          wizard.config = readAiForm(wizard.config || {});
          const d = await api("/api/ai/test", {
            config: wizard.config,
            api_key: $("#api-key").value,
          });
          $("#connection-result").textContent =
            d.message + " · " + d.models.join(", ");
        });
    }
    if (step === 4)
      w.innerHTML =
        "<h2>Zugriff</h2><p>Standard: nur dieses Gerät. Eine Freigabe benötigt Whitelist, Passwort und HTTPS. MAC-Adressen funktionieren nur im gleichen lokalen Netz.</p>" +
        wizardNetwork(wizard.config || {});
    if (step === 5)
      w.innerHTML = `<h2>Zusammenfassung</h2><p>Name: ${esc(wizard.name) || "–"}</p><p>Datenordner: <strong>${esc(wizard.path)}</strong></p><p>Vorlage: ${esc(wizard.template)}</p><p>KI-Modell: ${esc(wizard.config?.chat_model) || "später verbinden"}</p><p>Zugriff: ${wizard.config?.network_enabled ? "Whitelist, Passwort und HTTPS" : "nur dieses Gerät"}</p>`;
    if (step > 0)
      $("#wizard-nav").append(
        button(
          "Zurück",
          () => {
            step--;
            render();
          },
          true,
        ),
      );
    $("#wizard-nav").append(
      button(step === 5 ? "Einrichtung abschliessen" : "Weiter", async () => {
        if (step === 0) wizard.name = $("#name").value;
        if (step === 1) {
          wizard.path = $("#folder-path").value;
          if (!wizard.path) throw Error("Bitte einen Datenordner wählen");
        }
        if (step === 2) wizard.template = $("#template").value;
        if (step === 3) {
          wizard.config = readAiForm(wizard.config || {});
          wizard.api_key = $("#api-key").value;
        }
        if (step === 4) {
          wizard.config = {
            ...(wizard.config || {}),
            network_enabled: $("#wizard-network").checked,
            allowed_ips: $("#wizard-ips")
              .value.split("\n")
              .map((x) => x.trim())
              .filter(Boolean),
            allowed_macs: $("#wizard-macs")
              .value.split("\n")
              .map((x) => x.trim())
              .filter(Boolean),
          };
          wizard.password = $("#wizard-password").value;
        }
        if (step === 5) {
          await api("/api/setup", wizard);
          location.href = "/";
          return;
        }
        step++;
        render();
      }),
    );
  }
  render();
}
async function dashboard() {
  const d = await api("/api/dashboard");
  $("#content").innerHTML =
    `<h1>Deine Wissensbasis</h1><p class="muted">Unterlagen sammeln. Zusammenhänge verstehen. Wissen festhalten.</p><div class="grid"><div class="card"><h2>${d.documents} Unterlagen</h2><a href="/unterlagen">Unterlagen öffnen →</a></div><div class="card"><h2>${d.pending} offene Freigaben</h2><a href="/kapitel">Wissen prüfen →</a></div><div class="card"><h2>${d.errors} Fehler</h2><a href="/warteschlange">Verarbeitung ansehen →</a></div></div><div class="card"><h2>Dein Datenordner</h2><p>${esc(d.path)}</p><div id="scan-action"></div><p hx-get="/api/health-fragment" hx-trigger="load, every 10s" hx-swap="textContent"></p></div>`;
  const recent = document.createElement("div");
  recent.className = "grid";
  recent.innerHTML = `<section class="card"><h2>Neue Unterlagen</h2>${(d.recent_documents || []).map((x) => `<p>${esc(x[0])} <span class="badge" data-status="${esc(x[1])}">${esc(x[1])}</span></p>`).join("") || "Noch keine Unterlagen."}</section><section class="card"><h2>Letzte Chats</h2>${(d.recent_chats || []).map((x) => `<p><a href="/chat?chat=${encodeURIComponent(x[0])}">${esc(x[1])}</a></p>`).join("") || "Noch keine gespeicherten Chats."}</section>`;
  $("#content").append(recent);
  $("#scan-action").append(
    button("Dateien jetzt einlesen", async () => {
      await api("/api/scan", {});
      await dashboard();
    }),
  );
  if (window.htmx) window.htmx.process($("#content"));
}
async function search() {
  $("#content").innerHTML =
    '<h1>Suche</h1><form id="search"><label>Suchbegriffe<input id="query" required autofocus placeholder="Zum Beispiel Netzwerk"></label><label>Suchart<select id="search-kind"><option value="fulltext">Volltext · ohne Modell</option><option value="semantic">Semantisch · Embedding-Modell</option></select></label><button>Suchen</button></form><div id="semantic-tools"></div><div id="results"></div>';
  $("#semantic-tools").append(
    button(
      "Semantischen Index aufbauen",
      async () => {
        const d = await api("/api/semantic/rebuild", {});
        notice(d.count + " Abschnitte semantisch indiziert.", true);
      },
      true,
    ),
  );
  $("#search").onsubmit = (e) => {
    e.preventDefault();
    action(async () => {
      const c =
        $("#search-kind").value === "semantic"
          ? await api("/api/semantic/search", { q: $("#query").value })
          : await api("/api/search?q=" + encodeURIComponent($("#query").value));
      $("#results").innerHTML = c.length
        ? c
            .map(
              (x) =>
                `<article class="card"><a href="/api/source?id=${x.id}" target="_blank" rel="noopener">${esc(x.path)} · ${esc(x.location)}</a><pre>${esc(x.text)}</pre></article>`,
            )
            .join("")
        : "<p>Keine passende Grundlage gefunden.</p>";
    });
  };
}
async function render() {
  state = await api("/api/state");
  nav();
  if (state.login_required) {
    await loginPage();
    return;
  }
  if (!state.configured) {
    if (location.pathname != "/einrichtung")
      history.replaceState({}, "", "/einrichtung");
    await setup();
    return;
  }
  switch (location.pathname) {
    case "/":
      await dashboard();
      break;
    case "/unterlagen":
      await documents();
      break;
    case "/suche":
      await search();
      break;
    case "/chat":
      await chatPage();
      break;
    case "/kapitel":
      await chapterPage();
      break;
    case "/einstellungen":
      await settingsPage();
      break;
    case "/warteschlange":
      await queuePage();
      break;
    case "/protokoll":
      await protocolPage();
      break;
    case "/sicherung":
      await backupPage();
      break;
    default:
      await dashboard();
  }
}
if (localStorage.getItem("theme") === "dark")
  document.documentElement.classList.add("dark");
$("#theme").onclick = () => {
  document.documentElement.classList.toggle("dark");
  localStorage.setItem(
    "theme",
    document.documentElement.classList.contains("dark") ? "dark" : "light",
  );
};
action(render);

function aiForm(c) {
  return `<p class="muted">Bei Cloud-Anbietern werden Textausschnitte an den Anbieter gesendet.</p><label>Anbieter<select id="provider"><option value="">Benutzerdefiniert</option><option value="https://api.openai.com/v1">OpenAI</option><option value="http://127.0.0.1:11434/v1">Ollama</option><option value="http://127.0.0.1:1234/v1">LM Studio</option></select></label><label>Basis-URL<input id="base-url" value="${esc(c.base_url || "http://127.0.0.1:11434/v1")}"></label><label>API-Key<input id="api-key" type="password" autocomplete="new-password" placeholder="Neuer Schlüssel; leer lässt gespeicherten Schlüssel unverändert"></label><label>Chat-Modell<input id="chat-model" list="model-list" value="${esc(c.chat_model || "")}"><datalist id="model-list"></datalist></label><label>Embedding-Modell (optional)<input id="embedding-model" value="${esc(c.embedding_model || "")}"></label><div class="grid"><label>Temperatur<input id="temperature" type="number" min="0" max="2" step="0.1" value="${c.temperature ?? 0.2}"></label><label>Maximale Antwort-Tokens<input id="max-tokens" type="number" min="1" max="32000" value="${c.max_tokens || 1500}"></label></div><label><input type="checkbox" id="consent" ${c.cloud_consent ? "checked" : ""}> Ich stimme zu, dass Textausschnitte an den gewählten Cloud-Anbieter gesendet werden. Für lokale Anbieter nicht erforderlich.</label><button type="button" id="test-connection" class="secondary">Verbindung testen</button><p id="connection-result" role="status"></p>`;
}
function bindProvider() {
  $("#provider").onchange = () => {
    if ($("#provider").value) $("#base-url").value = $("#provider").value;
    $("#consent").checked = false;
  };
  $("#base-url").oninput = () => {
    $("#consent").checked = false;
  };
}
function readAiForm(c) {
  return {
    ...c,
    base_url: $("#base-url").value,
    chat_model: $("#chat-model").value,
    embedding_model: $("#embedding-model").value,
    temperature: Number($("#temperature").value),
    max_tokens: Number($("#max-tokens").value),
    cloud_consent: $("#consent").checked,
  };
}
async function settingsPage() {
  const d = await api("/api/settings");
  let c = d.config;
  $("#content").innerHTML =
    `<h1>Einstellungen</h1><form id="settings"><section class="card"><h2>KI-Modell</h2><p>${d.has_key ? "Ein API-Key ist sicher hinterlegt." : "Kein API-Key hinterlegt."}</p>${aiForm(c)}<label><input id="clear-key" type="checkbox"> Gespeicherten API-Key entfernen</label></section><section class="card"><h2>Ordnerstruktur</h2><label>Name<input id="person-name" value="${esc(c.name)}"></label><label>Vorlage<select id="folder-template">${["lehre-informatik", "lehre-allgemein", "schule", "frei"].map((v, i) => `<option value="${v}" ${c.template === v ? "selected" : ""}>${["Lehre Informatik", "Lehre allgemein", "Schule", "Frei"][i]}</option>`).join("")}</select></label><label>Ebenennamen (mit Komma, ein bis vier)<input id="levels" value="${esc(c.levels.join(", "))}"></label><label><input id="naming-rules" type="checkbox" ${c.naming_rules ? "checked" : ""}> Kleinbuchstaben und Bindestriche für neue Ordner verwenden</label></section><section class="card"><h2>Automatisierung und Qualität</h2><label><input id="free-chat" type="checkbox" ${c.allow_free_chat ? "checked" : ""}> Freien Chat erlauben</label><label><input id="auto-approve" type="checkbox" ${c.auto_approve ? "checked" : ""}> Belegte Entwürfe automatisch freigeben</label><label>Stabilitätswartezeit in Sekunden<input id="stable-seconds" type="number" min="0" value="${c.stable_seconds}"></label><label>Maximale Dateigrösse in MB<input id="max-file" type="number" min="1" max="500" value="${c.max_file_mb}"></label></section><div id="extra-settings"></div><button>Einstellungen speichern</button></form>`;
  bindProvider();
  $("#test-connection").onclick = () =>
    action(async () => {
      const d = await api("/api/ai/test", {
        config: readAiForm(c),
        api_key: $("#api-key").value || undefined,
      });
      $("#connection-result").textContent = d.message;
      $("#model-list").innerHTML = d.models
        .map((m) => `<option value="${esc(m)}">`)
        .join("");
    });
  $("#settings").onsubmit = (e) => {
    e.preventDefault();
    action(async () => {
      c = {
        ...readAiForm(c),
        name: $("#person-name").value,
        template: $("#folder-template").value,
        levels: $("#levels")
          .value.split(",")
          .map((x) => x.trim()),
        naming_rules: $("#naming-rules").checked,
        allow_free_chat: $("#free-chat").checked,
        auto_approve: $("#auto-approve").checked,
        stable_seconds: Number($("#stable-seconds").value),
        max_file_mb: Number($("#max-file").value),
      };
      if (typeof readExtraSettings === "function") c = readExtraSettings(c);
      await api("/api/settings", {
        config: c,
        api_key: $("#clear-key").checked
          ? ""
          : $("#api-key").value || undefined,
        password: $("#network-password")?.value || undefined,
      });
      $("#api-key").value = "";
      notice("Einstellungen gespeichert.", true);
    });
  };
  if (typeof extraSettings === "function") extraSettings(c, d.path);
}
function sourceLinks(sources) {
  return sources
    .map(
      (c, i) =>
        `<a href="/api/source?id=${encodeURIComponent(c.id)}${c.location.startsWith("S. ") ? "#page=" + Number(c.location.match(/^S\.\s*(\d+)/)?.[1] || 1) : ""}" target="_blank" rel="noopener">[${i + 1}] ${esc(c.path)} · ${esc(c.location)}</a>`,
    )
    .join("<br>");
}
async function chatPage() {
  const d = await api("/api/documents");
  const history = await api("/api/chats");
  let chatId = null,
    historyMessages = [];
  $("#content").innerHTML =
    `<h1>Chat</h1><div class="grid"><label>Bereich<select id="scope"><option value="">Alle Unterlagen</option>${d.directories
      .filter((p) => p !== "unterlagen")
      .map((p) => `<option value="${esc(p)}">${esc(p)}</option>`)
      .join(
        "",
      )}</select></label><label>Modus<select id="mode"><option value="erklaeren">Erklären</option><option value="zusammenfassen">Zusammenfassen</option><option value="abfragen">Abfragen</option>${state.allow_free_chat ? '<option value="frei">Freier Chat · ohne Unterlagen</option>' : ""}</select></label></div><details class="card"><summary>Gespeicherte Chats</summary><div id="chat-history"></div></details><div id="messages" class="messages"></div><form id="chat-form"><label>Deine Nachricht<textarea id="question" required></textarea></label><label><input id="save-chat" type="checkbox"> Verlauf speichern</label><button id="send">Senden</button></form>`;
  for (const row of history.chats) {
    const div = document.createElement("div");
    div.className = "row";
    div.append(
      button(
        row[1],
        () => {
          chatId = row[0];
          historyMessages = history.messages
            .filter((m) => m[0] === chatId)
            .map((m) => ({ role: m[1], content: m[2] }));
          $("#save-chat").checked = true;
          $("#messages").innerHTML = history.messages
            .filter((m) => m[0] === chatId)
            .map((m) => {
              const ids = JSON.parse(m[3]);
              const sources = (history.sources || []).filter((c) =>
                ids.includes(c.id),
              );
              let text = m[2];
              sources.forEach((c, i) => {
                text = text.split("[" + c.id + "]").join("[" + (i + 1) + "]");
              });
              return `<div class="card"><strong>${m[1] === "user" ? "Du" : "Folder-Mind"}</strong><div class="answer">${esc(text)}</div><p>${sourceLinks(sources)}</p></div>`;
            })
            .join("");
        },
        true,
      ),
      button(
        "Umbenennen",
        async () => {
          const title = prompt("Neuer Titel", row[1]);
          if (title) {
            await api("/api/chats/action", {
              id: row[0],
              action: "rename",
              title,
            });
            await chatPage();
          }
        },
        true,
      ),
      button(
        "Löschen",
        async () => {
          if (
            await confirmAction("Diesen Chat in den Papierkorb verschieben?")
          ) {
            await api("/api/chats/action", {
              id: row[0],
              action: "delete",
              confirmed: true,
            });
            await chatPage();
          }
        },
        true,
      ),
    );
    $("#chat-history").append(div);
  }
  const selectedChat = new URLSearchParams(location.search).get("chat");
  if (selectedChat) {
    const row = history.chats.find((r) => r[0] === selectedChat);
    if (row) {
      const button = [...$("#chat-history").querySelectorAll("button")].find(
        (b) => b.textContent === row[1],
      );
      button?.click();
    }
  }
  $("#chat-form").onsubmit = (e) => {
    e.preventDefault();
    action(async () => {
      const question = $("#question").value;
      $("#send").disabled = true;
      const div = document.createElement("article");
      div.className = "card";
      div.innerHTML = `<strong>Du</strong><p>${esc(question)}</p><strong>Folder-Mind</strong><div class="answer"></div><p class="sources"></p>`;
      $("#messages").append(div);
      const answer = div.querySelector(".answer");
      try {
        const r = await fetch("/api/chat", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "x-csrf-token": state.csrf,
          },
          body: JSON.stringify({
            question,
            scope: $("#scope").value,
            mode: $("#mode").value,
            chat_id: $("#save-chat").checked ? chatId : null,
            history:
              $("#save-chat").checked && chatId
                ? []
                : historyMessages.slice(-12),
            save: $("#save-chat").checked,
          }),
        });
        if (!r.ok) {
          const j = await r.json();
          throw Error(j.error || "Chat fehlgeschlagen");
        }
        const reader = r.body.getReader(),
          decoder = new TextDecoder();
        let buffer = "",
          text = "",
          sources = [];
        while (true) {
          const { value, done } = await reader.read();
          buffer += decoder.decode(value || new Uint8Array(), {
            stream: !done,
          });
          let pos;
          while ((pos = buffer.indexOf("\n\n")) >= 0) {
            const event = buffer.slice(0, pos);
            buffer = buffer.slice(pos + 2);
            const type = event
              .split("\n")
              .find((l) => l.startsWith("event:"))
              ?.slice(6)
              .trim();
            const data = event
              .split("\n")
              .filter((l) => l.startsWith("data:"))
              .map((l) => l.slice(5).trim())
              .join("\n");
            if (!data) continue;
            const j = JSON.parse(data);
            if (type === "sources") {
              sources = j.sources;
              if (j.free)
                div.querySelector(".sources").textContent =
                  "Freier Chat · ohne Unterlagenbezug";
            }
            if (type === "token") {
              text += j.text;
              answer.textContent = text;
            }
            if (type === "done") {
              chatId = j.chat_id;
              historyMessages.push(
                { role: "user", content: question },
                { role: "assistant", content: text },
              );
              sources.forEach((c, i) => {
                text = text.split("[" + c.id + "]").join("[" + (i + 1) + "]");
              });
              answer.textContent = text;
              if (sources.length)
                div.querySelector(".sources").innerHTML = sourceLinks(sources);
            }
          }
          if (done) break;
        }
        $("#question").value = "";
      } finally {
        $("#send").disabled = false;
      }
    });
  };
}
async function chapterPage() {
  const d = await api("/api/documents");
  const dirs = d.directories.filter((x) => x !== "unterlagen");
  const selected =
    new URLSearchParams(location.search).get("path") || dirs.at(-1) || "";
  $("#content").innerHTML =
    `<h1>Kapitel</h1><label>Kapitel auswählen<select id="chapter-path">${dirs.map((p) => `<option value="${esc(p)}" ${p === selected ? "selected" : ""}>${esc(p)}</option>`).join("")}</select></label><div id="chapter-content"></div>`;
  async function load() {
    const path = $("#chapter-path").value;
    if (!path) {
      $("#chapter-content").textContent = "Lege zuerst einen Kapitelordner an.";
      return;
    }
    const d = await api("/api/chapter?path=" + encodeURIComponent(path));
    $("#chapter-content").innerHTML =
      `<div class="grid"><section class="card"><h2>Entwurf</h2><pre>${esc(d.generated) || "Noch kein Entwurf."}</pre><div id="draft-actions" class="row"></div></section><section class="card"><h2>Freigegebenes Wissen</h2><pre>${esc(d.approved) || "Noch keine Freigabe."}</pre></section><section class="card"><h2>Eigene Notizen</h2><label>Deine Gedanken<textarea id="notes">${esc(d.notes)}</textarea></label><div id="notes-actions"></div><h2>Quellen</h2>${sourceLinks(d.sources)}</section></div>`;
    $("#draft-actions").append(
      button("Entwurf erstellen", async () => {
        await api("/api/chapter/draft", { path });
        await load();
      }),
      button(
        "Entwurf freigeben",
        async () => {
          await api("/api/chapter/approve", { path });
          await load();
        },
        true,
      ),
    );
    $("#notes-actions").append(
      button("Notizen speichern", async () => {
        await api("/api/chapter/notes", { path, text: $("#notes").value });
        notice("Notizen gespeichert.", true);
      }),
    );
  }
  $("#chapter-path").onchange = () => action(load);
  await load();
}

async function loginPage() {
  $("#content").innerHTML =
    '<h1>Anmeldung</h1><p>Dieses Gerät ist freigegeben. Bitte melde dich mit dem Netzwerkpasswort an.</p><form id="login"><label>Passwort<input type="password" id="login-password" autocomplete="current-password" required></label><button>Anmelden</button></form>';
  $("#login").onsubmit = (e) => {
    e.preventDefault();
    action(async () => {
      await api("/api/login", { password: $("#login-password").value });
      location.href = "/";
    });
  };
}
async function documents() {
  const d = await api("/api/documents");
  $("#content").innerHTML =
    `<h1>Unterlagen</h1><p>Die Ordner bestimmen die Zuordnung deiner Quellen. Originale werden nur auf deinen ausdrücklichen Auftrag verändert.</p><div class="card"><label>Zielordner<select id="folder-select">${d.directories.map((p) => `<option value="${esc(p)}" ${selectedPath === p ? "selected" : ""}>${esc(p)}</option>`).join("")}</select></label><div class="row" id="folder-tools"></div><label>Dateien hochladen<input type="file" id="upload-files" multiple></label><div class="drop" id="drop-zone" tabindex="0">Dateien hier ablegen oder oben auswählen</div><p id="upload-status" role="status"></p></div><div class="card tree" id="folder-tree"></div>`;
  const path = () => $("#folder-select").value;
  $("#folder-select").onchange = () => {
    selectedPath = path();
  };
  $("#folder-tools").append(
    button("Ordner anlegen", async () => {
      const name = prompt("Neuer Ordnername");
      if (name) {
        await api("/api/folder", { action: "create", path: path(), name });
        selectedPath = path();
        await documents();
      }
    }),
    button(
      "Umbenennen",
      async () => {
        const name = prompt("Neuer Name", path().split("/").pop());
        if (name) {
          await api("/api/folder", { action: "rename", path: path(), name });
          selectedPath = "";
          await documents();
        }
      },
      true,
    ),
    button(
      "Leeren Ordner löschen",
      async () => {
        if (
          await confirmAction(
            "Den leeren Ordner " + path() + " in den Papierkorb verschieben?",
          )
        ) {
          await api("/api/folder", {
            action: "delete",
            path: path(),
            confirmed: true,
          });
          selectedPath = "";
          await documents();
        }
      },
      true,
    ),
    button(
      "Kapitel öffnen",
      () => {
        location.href = "/kapitel?path=" + encodeURIComponent(path());
      },
      true,
    ),
  );
  async function upload(files) {
    const form = new FormData();
    for (const f of files) form.append("files", f);
    $("#upload-status").textContent = "Dateien werden hochgeladen …";
    const r = await fetch("/api/upload?path=" + encodeURIComponent(path()), {
      method: "POST",
      headers: { "x-csrf-token": state.csrf },
      body: form,
    });
    const j = await r.json();
    if (!r.ok) throw Error(j.error || "Upload fehlgeschlagen");
    notice(
      j.files.length +
        " Datei(en) hochgeladen. Die Verarbeitung beginnt nach der Stabilitätswartezeit.",
      true,
    );
    selectedPath = path();
    await documents();
  }
  $("#upload-files").onchange = (e) => action(() => upload(e.target.files));
  const zone = $("#drop-zone");
  zone.ondragover = (e) => {
    e.preventDefault();
    zone.classList.add("card");
  };
  zone.ondragleave = () => zone.classList.remove("card");
  zone.ondrop = (e) => {
    e.preventDefault();
    zone.classList.remove("card");
    action(() => upload(e.dataTransfer.files));
  };
  zone.onkeydown = (e) => {
    if (e.key === "Enter" || e.key === " ") $("#upload-files").click();
  };
  for (const dir of d.directories) {
    const row = document.createElement("div");
    row.className = "tree-item";
    row.textContent =
      "📁 " +
      dir +
      " · " +
      d.documents.filter((x) => x.path.startsWith(dir + "/")).length +
      " Dateien";
    row.append(
      button(
        "Auswählen",
        () => {
          $("#folder-select").value = dir;
          selectedPath = dir;
        },
        true,
      ),
    );
    $("#folder-tree").append(row);
    for (const doc of d.documents.filter(
      (x) => x.path.substring(0, x.path.lastIndexOf("/")) === dir,
    )) {
      const item = document.createElement("div");
      item.className = "tree-item";
      item.innerHTML = `📄 ${esc(doc.path.split("/").pop())} <span class="badge" data-status="${esc(doc.status)}">${esc(doc.status)}</span> <small>${esc(doc.error)}</small>`;
      item.append(
        button(
          "Verschieben",
          async () => {
            const target = prompt(
              "Vollständiger Zielpfad innerhalb von unterlagen/",
              doc.path,
            );
            if (
              target &&
              (await confirmAction("Datei nach " + target + " verschieben?"))
            ) {
              await api("/api/folder", {
                action: "move",
                path: doc.path,
                target,
              });
              await api("/api/scan", {});
              await documents();
            }
          },
          true,
        ),
        button(
          "Löschen",
          async () => {
            if (
              await confirmAction(
                "Datei " + doc.path + " in den Papierkorb verschieben?",
              )
            ) {
              await api("/api/folder", {
                action: "delete",
                path: doc.path,
                confirmed: true,
              });
              await api("/api/scan", {});
              await documents();
            }
          },
          true,
        ),
      );
      $("#folder-tree").append(item);
    }
  }
}
async function queuePage() {
  const d = await api("/api/queue");
  $("#content").innerHTML =
    `<h1>Warteschlange</h1><p>Verarbeitung ${d.paused ? "pausiert" : "aktiv"}. Neue Dateien werden erst nach der eingestellten Stabilitätswartezeit gelesen.</p><div class="row" id="queue-actions"></div><div class="card"><table><thead><tr><th>Datei</th><th>Status</th><th>Hinweis</th></tr></thead><tbody>${d.documents.map((x) => `<tr><td>${esc(x.path)}</td><td>${esc(x.status)}</td><td>${esc(x.error)}</td></tr>`).join("")}</tbody></table></div>`;
  for (const [label, action] of [
    [d.paused ? "Fortsetzen" : "Pause", d.paused ? "resume" : "pause"],
    ["Erneut verarbeiten", "retry"],
    ["Index neu aufbauen", "reindex"],
  ])
    $("#queue-actions").append(
      button(
        label,
        async () => {
          await api("/api/queue/action", { action });
          await queuePage();
        },
        true,
      ),
    );
}
function extraSettings(c, path) {
  $("#extra-settings").innerHTML =
    `<section class="card"><h2>Zugriff</h2><p>MAC-Adressen sind nur im gleichen lokalen Netz sichtbar. Über Router oder VPN funktioniert nur die IP-Freigabe. IP und MAC können gefälscht werden; deshalb sind Passwort und HTTPS verpflichtend.</p><p>Nur <a href="http://127.0.0.1:8484/einstellungen">127.0.0.1</a> darf diese Einstellungen ändern.</p><label><input id="network-enabled" type="checkbox" ${c.network_enabled ? "checked" : ""}> Netzwerkzugriff aktivieren</label><label>HTTPS-Schnittstelle (IP:Port)<input id="network-bind" value="${esc(c.network_bind)}"></label><label>Erlaubte IP-Adressen / CIDR (eine pro Zeile)<textarea id="allowed-ips">${esc(c.allowed_ips.join("\n"))}</textarea></label><label>Erlaubte MAC-Adressen (eine pro Zeile)<textarea id="allowed-macs">${esc(c.allowed_macs.join("\n"))}</textarea></label><label>Neues Netzwerkpasswort (mindestens 12 Zeichen)<input id="network-password" type="password" autocomplete="new-password"></label><p>Öffne auf dem anderen Gerät https://IP-DIESES-COMPUTERS:8485. Das selbst erzeugte Zertifikat muss dort im Browser bestätigt werden.</p><div id="access-status"></div><label>IP für MAC-Auflösung<input id="resolve-ip" placeholder="192.168.1.25"></label><button type="button" id="resolve-mac" class="secondary">MAC-Adresse nachschlagen</button><p id="mac-result" role="status"></p></section><section class="card"><h2>Datenordner</h2><p>${esc(path)}</p><div id="data-directory-tools"></div></section>`;
  $("#resolve-mac").onclick = () =>
    action(async () => {
      const d = await api(
        "/api/access/resolve?ip=" + encodeURIComponent($("#resolve-ip").value),
      );
      $("#mac-result").textContent = d.mac;
    });
  action(async () => {
    const d = await api("/api/access/status");
    $("#access-status").innerHTML =
      `<p>${esc(d.error)}</p><p>SHA-256 des Zertifikats: <small>${esc(d.certificate_sha256) || "Wird bei Aktivierung erzeugt"}</small></p><h3>Zuletzt abgelehnte Geräte</h3><div id="denied-devices"></div>`;
    for (const row of d.denied) {
      const div = document.createElement("div");
      div.className = "row";
      div.textContent = row[0];
      div.append(
        button(
          "Dieses Gerät freigeben",
          async () => {
            if (await confirmAction("IP " + row[0] + " freigeben?")) {
              await api("/api/access/allow", { ip: row[0], confirmed: true });
              await settingsPage();
            }
          },
          true,
        ),
      );
      $("#denied-devices").append(div);
    }
  });
  if (typeof comfortSettings === "function") comfortSettings(c, path);
}
function readExtraSettings(c) {
  const next = {
    ...c,
    network_enabled: $("#network-enabled").checked,
    network_bind: $("#network-bind").value,
    allowed_ips: $("#allowed-ips")
      .value.split("\n")
      .map((x) => x.trim())
      .filter(Boolean),
    allowed_macs: $("#allowed-macs")
      .value.split("\n")
      .map((x) => x.trim())
      .filter(Boolean),
  };
  return typeof readComfortSettings === "function"
    ? readComfortSettings(next)
    : next;
}
async function protocolPage() {
  const d = await api("/api/protocol");
  $("#content").innerHTML =
    `<h1>Protokoll</h1><div class="card"><h2>Verarbeitung</h2><pre>${esc(d.log) || "Noch keine Meldungen."}</pre>${d.documents
      .filter((x) => x.error)
      .map((x) => `<p>${esc(x.path)}: ${esc(x.error)}</p>`)
      .join(
        "",
      )}</div><div class="card"><h2>Zugriffe</h2><table><thead><tr><th>IP</th><th>Ergebnis</th><th>Zeit (UTC)</th></tr></thead><tbody>${d.access.map((x) => `<tr><td>${esc(x[0])}</td><td>${x[1] === "1" ? "erlaubt" : "verweigert"}</td><td>${esc(new Date(Number(x[2]) * 1000).toISOString())}</td></tr>`).join("")}</tbody></table></div>`;
}

function wizardNetwork(c) {
  return `<label><input id="wizard-network" type="checkbox" ${c.network_enabled ? "checked" : ""}> Andere Geräte freigeben</label><label>IP/CIDR (eine pro Zeile)<textarea id="wizard-ips">${esc((c.allowed_ips || []).join("\n"))}</textarea></label><label>MAC (eine pro Zeile)<textarea id="wizard-macs">${esc((c.allowed_macs || []).join("\n"))}</textarea></label><label>Netzwerkpasswort (mindestens 12 Zeichen)<input id="wizard-password" type="password" autocomplete="new-password"></label>`;
}
function comfortSettings(c, path) {
  $("#data-directory-tools").innerHTML =
    '<label>Neuer Datenordner<input id="new-data-path"></label><label><input id="take-data" type="checkbox" checked> Vorhandene Daten in einen leeren Zielordner mitnehmen</label><div id="data-buttons" class="row"></div>';
  $("#data-buttons").append(
    button(
      "Ordner auswählen",
      async () => {
        const path = await chooseFolder();
        if (path) $("#new-data-path").value = path;
      },
      true,
    ),
    button(
      "Datenordner wechseln",
      async () => {
        const path = $("#new-data-path").value;
        if (
          path &&
          (await confirmAction(
            "Datenordner nach " +
              path +
              " wechseln? Die bisherigen Daten bleiben erhalten. Netzwerkzugriff wird deaktiviert.",
          ))
        ) {
          await api("/api/data/change", {
            path,
            take_data: $("#take-data").checked,
            confirmed: true,
          });
          location.href = "/";
        }
      },
      true,
    ),
  );
  const section = document.createElement("section");
  section.className = "card";
  section.innerHTML = `<h2>Hintergrundbetrieb</h2><label><input id="autostart" type="checkbox" ${c.autostart ? "checked" : ""}> Beim Anmelden an Windows automatisch starten</label><div class="grid"><label>Verarbeitung ab Stunde (lokale Windows-Zeit)<input id="work-start" type="number" min="0" max="23" value="${c.work_start_hour}"></label><label>Verarbeitung bis Stunde (24 = Mitternacht)<input id="work-end" type="number" min="0" max="24" value="${c.work_end_hour}"></label></div><p>0 bis 24 bedeutet jederzeit. Ein Beginn nach dem Ende erlaubt ein Zeitfenster über Mitternacht.</p>`;
  $("#extra-settings").append(section);
  $("#folder-template").onchange = () => {
    const levels = {
      "lehre-informatik": ["Lehrjahr", "Modul", "Kapitel"],
      "lehre-allgemein": ["Lehrjahr", "Fach", "Thema"],
      schule: ["Fach", "Thema"],
      frei: ["Bereich"],
    };
    $("#levels").value = levels[$("#folder-template").value].join(", ");
  };
}
function readComfortSettings(c) {
  return {
    ...c,
    autostart: $("#autostart").checked,
    work_start_hour: Number($("#work-start").value),
    work_end_hour: Number($("#work-end").value),
  };
}
function chooseFolder() {
  return new Promise((resolve) => {
    const dialog = document.createElement("dialog");
    dialog.innerHTML =
      '<h2>Ordner auswählen</h2><label>Pfad<input id="choose-path"></label><div class="row" id="choose-actions"></div><div id="choose-directories" class="grid"></div><div class="row" id="choose-finish"></div><p id="choose-error" role="status"></p>';
    document.body.append(dialog);
    const find = (s) => dialog.querySelector(s);
    async function load(path = "") {
      try {
        const d = await api("/api/browse?path=" + encodeURIComponent(path));
        find("#choose-path").value = d.path;
        find("#choose-directories").replaceChildren();
        if (d.parent)
          find("#choose-directories").append(
            button("↑ Übergeordnet", () => load(d.parent), true),
          );
        for (const dir of d.directories)
          find("#choose-directories").append(
            button(dir, () => load(dir), true),
          );
      } catch (e) {
        find("#choose-error").textContent = e.message;
      }
    }
    function finish(path) {
      dialog.close();
      dialog.remove();
      resolve(path);
    }
    find("#choose-actions").append(
      button("Pfad öffnen", () => load(find("#choose-path").value), true),
      button(
        "Neuer Ordner",
        async () => {
          const name = prompt("Ordnername");
          if (name) {
            await api("/api/browse/create", {
              path: find("#choose-path").value,
              name,
            });
            await load(find("#choose-path").value + "/" + name);
          }
        },
        true,
      ),
    );
    find("#choose-finish").append(
      button("Diesen Ordner wählen", () => finish(find("#choose-path").value)),
      button("Abbrechen", () => finish(null), true),
    );
    dialog.oncancel = (e) => {
      e.preventDefault();
      finish(null);
    };
    dialog.showModal();
    action(() => load());
  });
}
async function backupPage() {
  const d = await api("/api/backup");
  $("#content").innerHTML =
    '<h1>Sicherung</h1><p>Sicherungen enthalten Originale, Wissen, Notizen, Extrakte, Chats und Katalog. Der API-Key und der private HTTPS-Schlüssel bleiben ausgeschlossen.</p><div id="backup-tools" class="row"></div><div id="backup-list"></div><div id="backup-preview"></div>';
  $("#backup-tools").append(
    button("Sicherung erstellen", async () => {
      const d = await api("/api/backup", {});
      await backupPage();
      notice("Sicherung erstellt: " + d.name, true);
    }),
  );
  for (const f of d.files) {
    const div = document.createElement("div");
    div.className = "card";
    div.innerHTML = `<strong>${esc(f.name)}</strong><p>${(f.size / 1024 / 1024).toFixed(2)} MB</p><a class="button" href="/api/backup/download?name=${encodeURIComponent(f.name)}" download>ZIP herunterladen</a> `;
    div.append(
      button(
        "Prüfen / Probelauf",
        async () => {
          const d = await api("/api/backup/preview", { name: f.name });
          $("#backup-preview").innerHTML =
            `<div class="card"><h2>Wiederherstellung prüfen</h2><p>${esc(d.message)}</p><p>Archiv SHA-256: <small>${esc(d.hash)}</small></p><table><thead><tr><th>Datei</th><th>Aktion</th></tr></thead><tbody>${d.files.map((x) => `<tr><td>${esc(x.path)}</td><td>${x.exists ? "Ersetzen" : "Anlegen"}</td></tr>`).join("")}</tbody></table><div id="restore-action"></div></div>`;
          $("#restore-action").append(
            button("Wiederherstellung bestätigen", async () => {
              if (
                await confirmAction(
                  "Die aufgeführten Dateien aus dieser Sicherung wiederherstellen?",
                )
              ) {
                const result = await api("/api/backup/restore", {
                  name: f.name,
                  hash: d.hash,
                  confirmed: true,
                });
                notice(
                  "Wiederhergestellt. Vorheriger Stand gesichert unter " +
                    result.safety_backup,
                  true,
                );
                await backupPage();
              }
            }),
          );
        },
        true,
      ),
    );
    $("#backup-list").append(div);
  }
}
