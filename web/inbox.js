"use strict";
let inboxTimer = null;
async function inboxPage(page = 0) {
  if (inboxTimer) clearTimeout(inboxTimer);
  const d = await api("/api/inbox?page=" + page);
  $("#content").innerHTML = `<div class="page-heading"><div><span class="eyebrow">DEINE AUTOMATISCHE ABLAGE</span><h1>Eingang</h1><p>Dateien oder ganze Ordner hier ablegen. Folder-Mind ordnet sie zu und fragt nach, wenn das Ziel unklar ist.</p></div><span class="badge">${d.paused ? "Pausiert" : d.running ? "Wird verarbeitet" : d.enabled ? "Aktiv" : "Deaktiviert"}</span></div><section class="card"><h2>Dein Input-Ordner im Explorer</h2><p class="path-display">${esc(d.path)}</p><div class="row" id="inbox-tools"></div><p class="muted">Alle Dateitypen bleiben erhalten. Inhalte werden bei unterstützten Formaten gelesen. Nur Archive mit exakt dem Namen <code>unzip.zip</code> werden entpackt; das Originalarchiv bleibt in <code>eingangsarchive</code>. Andere ZIP-Dateien werden als Datei einsortiert.</p><p>${d.auto_sort ? "Eindeutige Regeln und sichere KI-Zuordnungen werden automatisch ausgeführt." : "Automatisches Verschieben ist ausgeschaltet. Vorschläge warten auf deine Bestätigung."} <a href="/einstellungen">Einstellungen ändern →</a></p></section><section><div class="section-title"><h2>Rückfragen & laufende Eingänge <span class="badge">${d.pending_count}</span></h2><button class="secondary" id="refresh-inbox">Aktualisieren</button></div><div id="inbox-items"></div></section><details class="card" id="training"><summary>Einlernen · ${d.rules.length} Regeln · ${d.example_count} bestätigte Beispiele</summary><p>Erkläre, welche Unterlagen wohin gehören. Ein Schlüsselwort oder eine Dateiendung bildet eine eindeutige Regel. Freie Anweisungen und bestätigte Beispiele helfen der KI bei der nächsten Zuordnung.</p><form id="rule-form"><div class="grid"><label>Bezeichnung<input id="rule-label" placeholder="Zum Beispiel Netzwerk-Unterlagen"></label><label>Zielordner<input id="rule-target" list="inbox-folders" placeholder="unterlagen/netzwerk" required></label><label>Schlüsselwort (optional)<input id="rule-pattern" placeholder="netzwerk"></label><label>Dateiendung (optional)<input id="rule-extension" placeholder="pdf"></label></div><label>Zusätzliche Anweisung (optional)<textarea id="rule-instruction" placeholder="Zum Beispiel: Unterlagen zu IPv4 und IPv6 gehören zur Netzwerktechnik."></textarea></label><button>Zuordnung lernen</button></form><div id="routing-rules"></div><details><summary>Gespeicherte Beispiele ansehen</summary><div id="routing-examples"></div></details></details><section class="card"><div class="section-title"><h2>Verlauf</h2><span class="badge">${d.history_count} Vorgänge</span></div><p class="muted">Hier siehst du, was eingegangen ist, wohin es verschoben wurde und warum.</p><div class="table-scroll"><table><thead><tr><th>Zeit</th><th>Vorgang</th><th>Von</th><th>Nach</th><th>Grund</th><th>Aktion</th></tr></thead><tbody id="input-history"></tbody></table></div><div id="history-pages" class="row"></div></section><datalist id="inbox-folders">${d.directories.map((x) => `<option value="${esc(x)}">`).join("")}</datalist>`;
  $("#inbox-tools").append(button("Pfad kopieren", async () => {await navigator.clipboard.writeText(d.path);notice("Pfad kopiert.",true);}, true));
  if (d.windows && state.is_local) $("#inbox-tools").append(button("Im Explorer öffnen", () => api("/api/inbox/open", {}), true));
  $("#inbox-tools").append(button("Jetzt prüfen", async () => {await api("/api/inbox/process", {});notice("Die Prüfung läuft im Hintergrund. Der Verlauf wird automatisch aktualisiert.",true);schedule();}));
  $("#refresh-inbox").onclick = () => action(() => inboxPage(page));
  if (!d.items.length) $("#inbox-items").innerHTML = '<div class="empty-state"><h3>Alles im Blick</h3><p>Lege neue Dateien in deinen Input-Ordner. Offene Rückfragen erscheinen hier.</p></div>';
  for (const item of d.items) {
    const card = document.createElement("article");card.className="card inbox-item";
    card.innerHTML=`<div class="section-title"><h3>${esc(item.source.split("/").pop())}</h3><span class="badge">${esc(item.status.replaceAll("_"," "))}</span></div><p class="path-display">${esc(item.source)}</p>${item.origin?`<p class="muted">Aus Archiv: ${esc(item.origin)}</p>`:""}<p>${esc(item.reason)}</p>${item.question?`<p class="inbox-question">${esc(item.question)}</p>`:""}<details><summary>Textauszug ansehen</summary><pre>${esc(item.excerpt)||"Für diesen Dateityp ist kein Textauszug verfügbar. Dateiname und Zuordnungsregeln werden berücksichtigt."}</pre></details><div class="decision-fields"><label>Wohin gehört diese Datei?<input class="decision-target" list="inbox-folders" placeholder="unterlagen/fach/thema" value="${esc(item.target.startsWith("unterlagen")?item.target:"")}"></label><label><input class="decision-remember" type="checkbox" checked> Diese Zuordnung als Beispiel merken</label><label>Ähnliche Dateien künftig automatisch zuordnen, wenn dieses Wort vorkommt (optional)<input class="decision-pattern" placeholder="Schlüsselwort"></label><div class="row decision-actions"></div></div>`;
    const tools=card.querySelector(".decision-actions");
    card.querySelectorAll("input").forEach((input)=>input.addEventListener("input",()=>{card.dataset.dirty="true";}));
    tools.append(button("Ziel bestätigen & verschieben",async()=>{
      const target=card.querySelector(".decision-target").value.trim();if(!target)throw Error("Bitte einen Zielordner angeben.");
      await api("/api/inbox/answer",{id:item.id,target,remember:card.querySelector(".decision-remember").checked,pattern:card.querySelector(".decision-pattern").value});
      await inboxPage(page);notice("Datei verschoben. Deine bestätigte Zuordnung wurde berücksichtigt.",true);
    }),button("Erneut zuordnen",async()=>{await api("/api/inbox/retry",{id:item.id});await api("/api/inbox/process",{});await inboxPage(page);},true));
    if (!['rückfrage','wartet_auf_zuordnung'].includes(item.status)) card.querySelector(".decision-fields").hidden=true;
    $("#inbox-items").append(card);
  }
  $("#rule-form").onsubmit=(event)=>{event.preventDefault();action(async()=>{
    await api("/api/inbox/rules",{id:"",label:$("#rule-label").value,pattern:$("#rule-pattern").value,extension:$("#rule-extension").value,target:$("#rule-target").value,instruction:$("#rule-instruction").value,enabled:true});
    await inboxPage(page);$("#training").open=true;notice("Zuordnungsregel gespeichert. Offene Rückfragen kannst du erneut zuordnen lassen.",true);
  });};
  for(const rule of d.rules){
    const row=document.createElement("div");row.className="training-row";
    row.innerHTML=`<div><strong>${esc(rule.label||rule.pattern||rule.extension||"Zuordnungsanweisung")}</strong><p>${esc(rule.pattern?"Wort: "+rule.pattern:"")} ${esc(rule.extension?"Dateityp: ."+rule.extension:"")} → <code>${esc(rule.target)}</code></p><p class="muted">${esc(rule.instruction)}</p></div>`;
    row.append(button("Entfernen",async()=>{await api("/api/inbox/rules/delete",{id:rule.id});await inboxPage(page);$("#training").open=true;},true));$("#routing-rules").append(row);
  }
  for(const example of d.examples){
    const row=document.createElement("div");row.className="training-row";row.innerHTML=`<p>${esc(example.filename)} → <code>${esc(example.target)}</code></p>`;
    row.append(button("Beispiel vergessen",async()=>{await api("/api/inbox/examples/delete",{id:example.id});await inboxPage(page);$("#training").open=true;},true));$("#routing-examples").append(row);
  }
  for(const event of d.history){
    const label={"eingang-verschoben":"Einsortiert","zip-entpackt":"ZIP entpackt","chat-verschoben":"Im Chat verschoben","zurückgelegt":"Zurückgelegt"}[event.kind]||event.kind;
    const tr=document.createElement("tr");tr.innerHTML=`<td>${esc(new Date(event.created_at*1000).toLocaleString("de-CH"))}</td><td>${esc(label)}${event.reverted?" · zurückgelegt":""}</td><td>${esc(event.source)}</td><td>${esc(event.target)}</td><td>${esc(event.detail)}</td><td class="history-tools"></td>`;
    if(event.kind==="eingang-verschoben"&&!event.reverted)tr.querySelector(".history-tools").append(button("Zurück in Eingang",async()=>{
      if(await confirmAction("Diese Datei zurück in den Eingang legen? Geänderte oder ersetzte Dateien werden nicht überschrieben.")){await api("/api/inbox/undo",{id:event.id});await inboxPage(page);}
    },true));
    $("#input-history").append(tr);
  }
  if(!d.history.length)$("#input-history").innerHTML='<tr><td colspan="6">Noch keine Vorgänge.</td></tr>';
  if(page>0)$("#history-pages").append(button("← Neuere Vorgänge",()=>inboxPage(page-1),true));
  if((page+1)*50<d.history_count)$("#history-pages").append(button("Ältere Vorgänge →",()=>inboxPage(page+1),true));
  function schedule(){inboxTimer=setTimeout(()=>{
    const editing=$("#content").contains(document.activeElement)&&['INPUT','TEXTAREA','SELECT'].includes(document.activeElement.tagName);
    if(editing||$(".inbox-item[data-dirty]")||$("#training").open||$("#confirm").open){schedule();return;}
    action(()=>inboxPage(page));
  },7000);}
  schedule();
}
function inboxSettings(c){
  const section=document.createElement("section");section.className="card";
  section.innerHTML=`<h2>Automatische Eingangsablage</h2><p>Der Ordner <code>input</code> nimmt Dateien und Unterordner entgegen. Unklare Zuordnungen erscheinen auf der Seite <a href="/eingang">Eingang</a>.</p><label><input id="inbox-enabled" type="checkbox" ${c.inbox_enabled?"checked":""}> Eingangsordner im Hintergrund verarbeiten</label><label><input id="inbox-auto-sort" type="checkbox" ${c.inbox_auto_sort?"checked":""}> Eindeutige Zuordnungen automatisch verschieben</label><label>Mindest-Sicherheit für KI-Zuordnungen (0,5 bis 1)<input id="inbox-confidence" type="number" min="0.5" max="1" step="0.01" value="${c.inbox_confidence??0.9}"></label><p class="muted">Die KI bewertet ihre Zuordnung selbst. Bei Unsicherheit fragt Folder-Mind nach. Bestätigte Regeln gelten unabhängig von dieser Einschätzung.</p>`;
  $("#extra-settings").append(section);
}
