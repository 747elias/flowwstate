"use strict";
async function chatPage() {
  document.body.classList.add("chat-mode");
  const initialParams = new URLSearchParams(location.search);
  const [docs, agentData] = await Promise.all([api("/api/documents"), api("/api/agents")]);
  let saved = await api("/api/chats"), agents = agentData.agents;
  let chatId = null, historyMessages = [], busy = false;
  let agentId = new URLSearchParams(location.search).get("agent") || "erklaeren";
  const getAgent = () => agents.find((a) => a.id === agentId) || agents[0];
  if (!state.allow_free_chat) agents = agents.filter((a) => a.id !== "frei");
  $("#content").innerHTML = `<section class="chat-workspace"><div class="chat-topbar"><div><span class="eyebrow">DEIN ASSISTENT</span><h1 id="agent-title"></h1><p class="muted" id="agent-description"></p></div><div id="conversation-actions" class="row"></div></div><div class="chat-options"><label>Bereich<select id="scope"><option value="">Alle Unterlagen</option>${docs.directories.map((p) => `<option value="${esc(p)}">${esc(p)}</option>`).join("")}</select></label><label class="save-choice"><input id="save-chat" type="checkbox" checked> Verlauf speichern</label></div><div id="messages" class="messages" role="log" aria-live="polite"></div><form id="chat-form" class="chat-composer"><label class="sr-only" for="question">Deine Nachricht</label><textarea id="question" rows="2" placeholder="Schreibe eine Nachricht …" required maxlength="16000"></textarea><div class="composer-bottom"><small id="composer-hint">Enter zum Senden · Umschalt+Enter für eine neue Zeile</small><button id="send" title="Nachricht senden">Senden ↑</button></div></form><p class="chat-footnote">Folder-Mind kann Fehler machen. Prüfe wichtige Antworten und vorgeschlagene Änderungen.</p></section>`;
  function updateAgent() {
    const a = getAgent(); agentId = a.id;
    $("#agent-title").textContent = a.name;
    $("#agent-description").textContent = a.description;
    $("#scope").disabled = ["organizer", "builder"].includes(a.kind);
    $("#scope").value = a.scope || "";
    $("#question").placeholder = a.kind === "organizer" ? "Wo liegt mein Dokument? Oder: Verschiebe … nach …" : a.kind === "builder" ? "Beschreibe den Agenten, den du erstellen möchtest …" : a.kind === "summary" ? "Was soll ich für dich zusammenfassen?" : "Wobei kann ich dir helfen?";
    $("#composer-hint").textContent = a.kind === "summary" ? "Bis zu 24 Textabschnitte pro Zusammenfassung. Für grosse Bestände einen gezielten Bereich wählen." : "Enter zum Senden · Umschalt+Enter für eine neue Zeile";
    $("#question").value = sessionStorage.getItem("folder-mind-draft-" + agentId) || "";
    sidebar();
  }
  function newChat(id = agentId) {
    if (busy) return;
    agentId = id; chatId = null; historyMessages = [];
    $("#messages").replaceChildren();
    const welcome = document.createElement("div"); welcome.className = "chat-welcome";
    welcome.innerHTML = '<span class="assistant-avatar">◈</span><h2>Was möchtest du erledigen?</h2><p id="welcome-description" class="muted"></p><div id="chat-starters" class="starter-grid"></div>';
    $("#messages").append(welcome);
    updateAgent();
    $("#welcome-description").textContent = getAgent().description;
    const samples = getAgent().kind === "organizer" ? ["Wo ist mein Netzwerk-Dokument gespeichert?", "Ich möchte ein Dokument verschieben."] : getAgent().kind === "builder" ? ["Erstelle einen Agenten, der mir Karteikarten aus meinen Unterlagen erstellt.", "Ich möchte einen Assistenten für Prüfungsvorbereitung erstellen."] : getAgent().kind === "summary" ? ["Fasse die ausgewählten Unterlagen zusammen.", "Erstelle eine kurze Merkliste der wichtigsten Punkte."] : ["Erkläre mir die wichtigsten Begriffe.", "Hilf mir, dieses Thema zu verstehen."];
    for (const text of samples) $("#chat-starters").append(button(text, () => { $("#question").value = text; $("#question").focus(); }, true));
    history.replaceState({}, "", "/chat?agent=" + encodeURIComponent(agentId));
  }
  function sidebar() {
    $("#nav").className = "chat-navigation";
    $("#nav").innerHTML = '<div id="chat-nav-tools"></div><p class="nav-label">AGENTEN</p><div id="agent-list"></div><a href="/agenten" class="nav-subtle">Agenten verwalten ＋</a><p class="nav-label">GESPRÄCHE</p><div id="chat-history"></div><details class="nav-areas"><summary>Arbeitsbereiche</summary><div id="area-links"></div></details>';
    $("#chat-nav-tools").append(button("＋ Neuer Chat", () => newChat(), true));
    for (const a of agents) {
      const b = button(a.name, () => {newChat(a.id); document.body.classList.remove("nav-open");}, true);
      b.className = "agent-button" + (a.id === agentId ? " selected" : "");
      b.title = a.description; b.setAttribute("aria-pressed", String(a.id === agentId));
      $("#agent-list").append(b);
    }
    for (const row of saved.chats) {
      const b = button(row[1], () => loadChat(row[0]), true);
      b.className = "history-button" + (row[0] === chatId ? " selected" : "");
      $("#chat-history").append(b);
    }
    if (!saved.chats.length) $("#chat-history").innerHTML = '<p class="muted nav-empty">Deine gespeicherten Gespräche erscheinen hier.</p>';
    $("#area-links").innerHTML = pages.filter(([p]) => p !== "/chat").map(([p, t]) => `<a href="${p}">${t}</a>`).join("");
  }
  function addMessage(role, text, html = null, name = null) {
    $(".chat-welcome")?.remove();
    const el = document.createElement("article"); el.className = "chat-message " + (role === "user" ? "from-user" : "from-assistant");
    el.innerHTML = `<div class="message-label">${role === "user" ? "Du" : '<span class="assistant-avatar">◈</span> ' + esc(name || getAgent().name)}</div><div class="answer markdown"></div><div class="sources"></div><div class="proposals"></div>`;
    if (html === null) el.querySelector(".answer").textContent = text;
    else el.querySelector(".answer").innerHTML = html;
    $("#messages").append(el); return el;
  }
  function showProposals(el, proposals) {
    for (const p of proposals) {
      const card = document.createElement("section"); card.className = "proposal-card";
      if (p.kind === "move") card.innerHTML = `<h3>Verschieben vorbereiten</h3><dl><dt>Von</dt><dd>${esc(p.source)}</dd><dt>Nach</dt><dd>${esc(p.target)}</dd></dl><p class="muted">Die Datei wird erst mit deiner Bestätigung verschoben.</p>`;
      else card.innerHTML = `<h3>Agentenentwurf: ${esc(p.agent?.name)}</h3><p>${esc(p.agent?.description)}</p><p>${p.agent?.grounded ? "Mit Quellen aus deinen Unterlagen" : "Freier Chat"} · ${esc(p.agent?.scope || "Alle Unterlagen")}</p><details><summary>Anweisungen prüfen</summary><pre>${esc(p.agent?.instructions)}</pre></details>`;
      const tools = document.createElement("div"); tools.className = "row"; card.append(tools);
      if (["offen","verschiebt"].includes(p.status)) tools.append(
        button(p.kind === "move" ? "Verschieben bestätigen" : "Agenten übernehmen", async () => {
          const result = await api("/api/actions/confirm", {id: p.id});
          tools.replaceChildren(); tools.textContent = p.kind === "move" ? "✓ Datei verschoben. Der Vorgang steht im Eingangsverlauf." : "✓ Agent erstellt.";
          p.status = "ausgeführt";
          if (p.kind === "agent") {
            agents = (await api("/api/agents")).agents; sidebar();
            const link = document.createElement("a");link.href="/chat?agent="+encodeURIComponent(result.proposal.agent.id);link.textContent="Mit diesem Agenten chatten →";tools.append(link);
          }
        }),
        button("Verwerfen", async () => {await api("/api/actions/reject", {id: p.id});p.status="verworfen";tools.replaceChildren();tools.textContent="Vorschlag verworfen.";}, true),
      ); else tools.textContent = "Status: " + p.status;
      el.querySelector(".proposals").append(card);
    }
  }
  async function loadChat(id) {
    if (busy) return;
    chatId = id;
    const stored = saved.chat_agents.find((r) => r[0] === id)?.[1];
    agentId = agents.some((a) => a.id === stored) ? stored : "erklaeren";
    updateAgent(); $("#save-chat").checked = true;
    $("#messages").replaceChildren(); historyMessages = [];
    saved.messages.forEach((m, i) => {
      if (m[0] !== id) return;
      historyMessages.push({role:m[1],content:m[2]});
      const el = addMessage(m[1], m[2], saved.rendered?.[i] || null);
      const ids = JSON.parse(m[3]), refs = saved.sources.filter((c) => ids.includes(c.id));
      el.querySelector(".sources").innerHTML = sourceLinks(refs);
    });
    const open = saved.proposals.filter((p) => p.chat_id === id);
    if (open.length) {
      const last = [...$("#messages").querySelectorAll(".from-assistant")].at(-1);
      if (last) showProposals(last, open);
    }
    document.body.classList.remove("nav-open");
    history.replaceState({}, "", "/chat?chat=" + encodeURIComponent(id));
    $("#question").focus();
  }
  $("#conversation-actions").append(
    button("Umbenennen", async () => {
      if (!chatId) return notice("Speichere zuerst eine Nachricht in diesem Gespräch.");
      const title=prompt("Neuer Titel",saved.chats.find((c)=>c[0]===chatId)?.[1]||"");
      if (title) {await api("/api/chats/action",{id:chatId,action:"rename",title});saved=await api("/api/chats");sidebar();}
    },true),
    button("Löschen",async()=>{
      if (chatId && await confirmAction("Dieses gespeicherte Gespräch löschen?")) {await api("/api/chats/action",{id:chatId,action:"delete",confirmed:true});saved=await api("/api/chats");newChat();}
    },true),
  );
  $("#question").oninput=()=>sessionStorage.setItem("folder-mind-draft-"+agentId,$("#question").value);
  $("#question").onkeydown=(e)=>{if(e.key==="Enter"&&!e.shiftKey&&!e.isComposing){e.preventDefault();if(!busy)$("#chat-form").requestSubmit();}};
  $("#chat-form").onsubmit=(e)=>{e.preventDefault();action(async()=>{
    if(busy)return;
    const question=$("#question").value.trim();if(!question)return;
    busy=true;$("#send").disabled=true;$("#send").textContent="Antwort läuft …";
    addMessage("user",question);const el=addMessage("assistant","Antwort wird vorbereitet …");
    const answer=el.querySelector(".answer");
    try {
      const response=await fetch("/api/chat",{method:"POST",headers:{"Content-Type":"application/json","x-csrf-token":state.csrf},body:JSON.stringify({question,scope:$("#scope").value,agent_id:agentId,chat_id:$("#save-chat").checked?chatId:null,history:$("#save-chat").checked&&chatId?[]:historyMessages.slice(-12),save:$("#save-chat").checked})});
      if(!response.ok){const error=await response.json();throw Error(error.error||"Chat fehlgeschlagen");}
      const reader=response.body.getReader(),decoder=new TextDecoder();let buffer="",text="",sources=[],completed=false;
      while(true){
        const {value,done}=await reader.read();buffer+=decoder.decode(value||new Uint8Array(),{stream:!done});let pos;
        while((pos=buffer.indexOf("\n\n"))>=0){
          const event=buffer.slice(0,pos);buffer=buffer.slice(pos+2);
          const type=event.split("\n").find((l)=>l.startsWith("event:"))?.slice(6).trim();
          const data=event.split("\n").filter((l)=>l.startsWith("data:")).map((l)=>l.slice(5).trim()).join("\n");if(!data)continue;
          const j=JSON.parse(data);
          if(type==="sources"){sources=j.sources;if(j.free)el.querySelector(".sources").textContent="Freier Chat · ohne Unterlagenbezug";}
          if(type==="token"){text+=j.text;answer.textContent=text;}
          if(type==="done"){
            completed=true;chatId=j.chat_id;answer.innerHTML=j.html;showProposals(el,j.proposals||[]);
            if(sources.length)el.querySelector(".sources").innerHTML=sourceLinks(sources);
            historyMessages.push({role:"user",content:question},{role:"assistant",content:text});
          }
        }
        if(done)break;
      }
      if(!completed)throw Error("Die Verbindung wurde unterbrochen. Deine Eingabe bleibt erhalten.");
      $("#question").value="";sessionStorage.removeItem("folder-mind-draft-"+agentId);
      if($("#save-chat").checked){saved=await api("/api/chats");sidebar();}
      el.scrollIntoView({behavior:"smooth",block:"nearest"});
    }catch(error){answer.textContent="Die Anfrage ist fehlgeschlagen: "+error.message;throw error;}
    finally{busy=false;$("#send").disabled=false;$("#send").textContent="Senden ↑";$("#question").focus();}
  });};
  newChat(agentId);
  const selected = initialParams.get("chat");
  if(selected)await loadChat(selected);
}
