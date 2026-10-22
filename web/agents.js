"use strict";
async function agentsPage(){
  const [d,docs]=await Promise.all([api("/api/agents"),api("/api/documents")]);
  $("#content").innerHTML='<div class="page-heading"><div><span class="eyebrow">DEINE ASSISTENTEN</span><h1>Agenten</h1><p>Wähle einen Assistenten oder erstelle einen eigenen mit passenden Anweisungen.</p></div><a class="button" href="/chat?agent=agent-builder">Mit Agent-Builder erstellen ＋</a></div><div id="agent-cards" class="agent-grid"></div><section class="card"><h2 id="agent-form-title">Eigenen Agenten erstellen</h2><form id="agent-editor"><input type="hidden" id="edit-agent-id"><div class="grid"><label>Name<input id="edit-agent-name" required maxlength="100"></label><label>Kurzbeschreibung<input id="edit-agent-description" maxlength="400"></label></div><label>Anweisungen<textarea id="edit-agent-instructions" required maxlength="8000" placeholder="Was soll dieser Agent tun? Wie soll er antworten?"></textarea></label><label><input id="edit-agent-grounded" type="checkbox" checked> Antworten ausschliesslich aus meinen Unterlagen, mit Quellen</label><label>Standardbereich<select id="edit-agent-scope"><option value="">Alle Unterlagen</option>'+docs.directories.map((p)=>`<option value="${esc(p)}">${esc(p)}</option>`).join("")+'</select></label><div class="row"><button>Agent speichern</button><button type="button" class="secondary" id="reset-agent-form">Neues Profil</button></div><p class="muted">Eigene Agenten erhalten keine Dateisystem-Werkzeuge. Für Dateiaktionen gibt es den Ablage-Agenten mit Bestätigung.</p></form></section>';
  for(const agent of d.agents){
    const card=document.createElement("article");card.className="card agent-card";
    card.innerHTML=`<span class="agent-symbol">${agent.kind==="organizer"?"▤":agent.kind==="builder"?"＋":agent.kind==="summary"?"≡":"◈"}</span><h2>${esc(agent.name)}</h2><p>${esc(agent.description)}</p><span class="badge">${agent.builtin?"Mitgeliefert":"Eigenes Profil"}</span><div class="row agent-card-actions"></div>`;
    const link=document.createElement("a");link.href="/chat?agent="+encodeURIComponent(agent.id);link.className="button";link.textContent="Chat öffnen";card.querySelector(".agent-card-actions").append(link);
    if(!agent.builtin)card.querySelector(".agent-card-actions").append(button("Bearbeiten",()=>{
      $("#edit-agent-id").value=agent.id;$("#edit-agent-name").value=agent.name;$("#edit-agent-description").value=agent.description;$("#edit-agent-instructions").value=agent.instructions;$("#edit-agent-grounded").checked=agent.grounded;$("#edit-agent-scope").value=agent.scope;$("#agent-form-title").textContent="Agent bearbeiten";$("#agent-editor").scrollIntoView({behavior:"smooth"});
    },true),button("Löschen",async()=>{
      if(await confirmAction('Agent „'+agent.name+'“ löschen? Gespeicherte Chats bleiben erhalten.')){await api("/api/agents/delete",{id:agent.id});await agentsPage();}
    },true));
    $("#agent-cards").append(card);
  }
  $("#reset-agent-form").onclick=()=>{$("#agent-editor").reset();$("#edit-agent-id").value="";$("#agent-form-title").textContent="Eigenen Agenten erstellen";};
  $("#agent-editor").onsubmit=(e)=>{e.preventDefault();action(async()=>{
    await api("/api/agents",{id:$("#edit-agent-id").value,name:$("#edit-agent-name").value,description:$("#edit-agent-description").value,instructions:$("#edit-agent-instructions").value,grounded:$("#edit-agent-grounded").checked,scope:$("#edit-agent-scope").value});
    await agentsPage();notice("Agent gespeichert. Du kannst ihn jetzt im Chat auswählen.",true);
  });};
}
async function updatePage(){
  const d=await api("/api/update");
  $("#content").innerHTML=`<span class="eyebrow">PROGRAMM AKTUALISIEREN</span><h1>Update ohne neue Einrichtung</h1><section class="card"><h2>Aktuelle Version ${esc(d.version)}</h2><p>Deine gespeicherten Unterlagen, Chats, Notizen, Agenten und Zuordnungen liegen getrennt von der EXE in:</p><p class="path-display">${esc(d.data_path)}</p><ol><li><a href="${esc(d.release_url)}" target="_blank" rel="noopener">Neue folder-mind.exe von GitHub herunterladen</a>.</li><li>Offene Eingaben absenden und gewünschte Chats speichern.</li><li>Unten eine Sicherung erstellen und Folder-Mind beenden.</li><li>Die bisherige EXE durch die neue ersetzen und die neue EXE starten.</li></ol><p>Die neue Version findet denselben Datenordner automatisch. Datenbank-Erweiterungen werden ohne erneute Einrichtung übernommen. API-Key und Einstellungen bleiben erhalten. Ein aktivierter Windows-Autostart wird auf die gestartete EXE aktualisiert.</p><div id="update-actions"></div><p id="update-result" role="status"></p></section><section class="card"><h2>Automatische Sicherung vor Datenbank-Updates</h2><p>Vor einer Schema-Erweiterung sichert Folder-Mind die bisherige Datenbank und Konfiguration. Originaldateien bleiben dabei unverändert.</p>${d.migration_backups.length?'<ul>'+d.migration_backups.map((name)=>'<li><code>'+esc(name)+'</code></li>').join("")+'</ul>':'<p>Noch keine Datenbank-Erweiterung erforderlich.</p>'}<p class="muted">Eine ältere EXE nach einem Update nicht erneut verwenden. <a href="/sicherung">Sicherungen verwalten →</a></p></section>`;
  const b=button("Sichern & für Update beenden",async()=>{
    if(!await confirmAction("Die neue EXE ist heruntergeladen? Folder-Mind erstellt jetzt eine Sicherung und beendet sich danach. Nicht abgesendete oder ungespeicherte Chat-Eingaben vorher speichern."))return;
    b.disabled=true;$("#update-result").textContent="Verarbeitung wird pausiert und eine Sicherung erstellt …";
    try{const result=await api("/api/update/prepare",{confirmed:true});$("#update-result").textContent=result.message+" Sicherung: "+result.backup;}
    catch(e){b.disabled=false;throw e;}
  });
  $("#update-actions").append(b);
}
