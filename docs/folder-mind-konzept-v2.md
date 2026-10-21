# Folder-Mind – Produkt- und Technikkonzept

**Status:** Entwurf 2.0
**Programmiersprache:** Rust
**Produktart:** Lokal laufende Anwendung mit Hintergrunddienst und Weboberfläche (Web UI)
**Zielgruppe:** Lernende (z. B. Informatik-Lehre), die ihre Lernunterlagen geordnet ablegen und mit einem KI-Modell darüber chatten wollen

## 1. Kurzbeschreibung

Folder-Mind macht aus Lernunterlagen eine gut organisierte, durchsuchbare Wissensbasis, mit der man über eine Weboberfläche chatten kann. Unterlagen (Skripte, Folien, Notizen, Aufträge) werden in einen Datenordner gelegt. Die Ordnerstruktur bestimmt Fach, Thema und Kapitel – zum Beispiel Lehrjahr → Modul → Kapitel.

**Alles wird über die Web UI erledigt:** Ersteinrichtung, Wahl des Datenordners, Ordnerstruktur, KI-Modell, API-Key, Zugriffsfreigaben, Sicherungen und der Chat selbst. Eine Kommandozeile oder das Bearbeiten von Konfigurationsdateien ist nicht nötig.

Das KI-Modell wird über eine **OpenAI-kompatible API** angebunden (z. B. OpenAI, ein Firmen-Gateway oder ein lokaler Server wie Ollama/LM Studio). Basis-URL, Modellname und API-Key werden in der Web UI eingegeben.

Folder-Mind verändert **ausschliesslich Dateien innerhalb des bei der Einrichtung festgelegten Datenordners**. Antworten im Chat verweisen auf die verwendeten Quellen. Ist keine passende Grundlage vorhanden, sagt Folder-Mind dies klar.

## 2. Ziele

- **Einfach für Lernende:** Starten, Browser öffnen, Einrichtungsassistent durchklicken, loslegen.
- **Alles in der Web UI:** Keine Konfigurationsdateien von Hand, keine Befehle.
- **Gute Organisation:** Eine klare, optional anpassbare Ordnerstruktur für Fächer, Module und Kapitel.
- **Chat mit eigenen Unterlagen:** Fragen stellen, Erklärungen und Zusammenfassungen erhalten – mit Quellenangabe.
- **Frei wählbares Modell:** Jeder Anbieter mit OpenAI-kompatibler API.
- **Sicher per Standard:** Web UI nur auf dem eigenen Gerät erreichbar; andere Geräte nur nach ausdrücklicher Freigabe (IP- oder MAC-Whitelist).
- **Begrenzter Schreibbereich:** Das Programm schreibt nur in den festgelegten Datenordner.
- **Nachweisbarkeit:** Jede Antwort und jeder Entwurf kann auf Datei und Seite/Folie/Abschnitt zurückgeführt werden.

## 3. Bewusste Abgrenzung

Folder-Mind ist **kein** eigenes Sprachmodell und keine Websuchmaschine. Es nutzt nur das in der Web UI eingerichtete Modell.

Nicht zum Startumfang gehören:

- automatische inhaltliche Umordnung von Dateien durch die KI;
- Zugriff auf Dateien ausserhalb des Datenordners;
- Mehrbenutzer-Konten und Rollen (ein Gerät, eine lernende Person);
- Unterstützung jedes denkbaren Dateiformats;
- eine Garantie, dass ein Sprachmodell nie einen Fehler macht.

## 4. Bedienidee

### 4.1 Erster Start – Einrichtung komplett über die Web UI

1. Folder-Mind wird gestartet (Doppelklick bzw. als Dienst).
2. Der Browser öffnet automatisch `http://127.0.0.1:8484`.
3. Ein **Einrichtungsassistent** führt durch folgende Schritte:
   1. **Sprache und Name** (optional).
   2. **Datenordner wählen:** Pfad über eine Ordnerauswahl festlegen. Nur dieser Ordner wird später vom Programm verändert. Das Programm prüft Schreibrechte und zeigt den Pfad deutlich an.
   3. **Ordnerstruktur festlegen (optional):** Vorlage wählen oder eigene Struktur bauen (siehe Abschnitt 5). Wird nichts gewählt, gilt die Standardvorlage.
   4. **KI-Modell verbinden (optional):** Basis-URL, API-Key und Modell eingeben, „Verbindung testen“ klicken. Ohne Modell sind Ablage, Übersicht und Volltextsuche trotzdem nutzbar.
   5. **Zugriff:** Standard „nur dieses Gerät“. Optional weitere Geräte per IP- oder MAC-Adresse freigeben.
   6. **Zusammenfassung und Abschluss:** Alle Einstellungen werden angezeigt und bestätigt.
4. Nach dem Abschluss erscheint die Startseite mit Übersicht und Chat.

Alle Einstellungen aus dem Assistenten können später unter **Einstellungen** in der Web UI geändert werden.

### 4.2 Alltag

1. Die lernende Person legt Dateien in den passenden Kapitelordner – per Dateiexplorer oder per **Upload in der Web UI** (Drag & Drop direkt in ein Kapitel).
2. Folder-Mind erkennt die neue Datei, wartet, bis sie vollständig kopiert ist, und verarbeitet sie im Hintergrund.
3. Text wird mit Fundstellen extrahiert; optional erstellt das Modell eine Zusammenfassung als Entwurf.
4. Die Person prüft den Entwurf in der Web UI und gibt ihn frei (oder lässt die Freigabe in den Einstellungen automatisch zu).
5. Im **Chat** können Fragen zu einem Kapitel, einem Modul oder allen Unterlagen gestellt werden. Die Antwort zeigt die Quellen an.

## 5. Ordnerstruktur

### 5.1 Standardvorlage „Lehre“

~~~text
<Datenordner>/
├── unterlagen/                      # Originale (werden nie verändert)
│   └── 1-lehrjahr/
│       └── modul-117-netzwerk/
│           └── kapitel-01-grundlagen/
│               └── skript.pdf
├── extrakte/                        # automatisch erzeugt, neu erzeugbar
├── wissen/
│   └── 1-lehrjahr/modul-117-netzwerk/
│       ├── kapitel-01-grundlagen.generated.md
│       ├── kapitel-01-grundlagen.notes.md
│       └── kapitel-01-grundlagen.md
├── chats/                           # gespeicherte Chatverläufe (optional)
├── protokolle/
├── backups/
├── .folder-mind/                    # Konfiguration, Datenbank, Index
│   ├── config.toml
│   ├── folder-mind.sqlite
│   └── index/
~~~

### 5.2 Ordnerstruktur über die Web UI anpassen

In **Einstellungen → Ordnerstruktur** kann die Person:

- eine Vorlage wählen: *Lehre (Lehrjahr → Modul → Kapitel)*, *Schule (Fach → Thema)*, *Frei (beliebige Tiefe)*;
- die Ebenen benennen (z. B. „Lehrjahr“, „Modul“, „Kapitel“) und die Anzahl Ebenen festlegen (1–4);
- Ordner direkt in der Web UI anlegen, umbenennen und – nach Bestätigung – leere Ordner löschen;
- Namensregeln aktivieren (z. B. Kleinbuchstaben, Bindestriche statt Leerzeichen, Nummerierung);
- die Struktur als Baum ansehen, inklusive Anzahl Dateien und Verarbeitungsstatus.

Die Ordnerstruktur ist die verbindliche Zuordnung. Die KI darf **keine** Dateien verschieben. Vorschläge der KI (z. B. „Diese Datei passt eher zu Modul 123“) werden höchstens als Hinweis angezeigt; verschoben wird nur, wenn die Person auf „Verschieben“ klickt.

## 6. Die drei Wissensebenen

| Ebene | Zweck | Darf automatisch überschrieben werden? |
|---|---|---|
| Original | Unveränderte Datei in `unterlagen/` | Nein |
| Extrakt | Text mit Seiten-/Folienverweisen | Ja, bei erneuter Verarbeitung |
| Wissen | Entwurf, freigegebene Seite und eigene Notizen | Nur der Entwurf |

- **kapitel.generated.md:** automatisch erzeugter Entwurf; darf ersetzt werden.
- **kapitel.notes.md:** eigene Notizen der lernenden Person; werden nie vom Programm verändert (Bearbeitung in der Web UI möglich).
- **kapitel.md:** freigegebene Wissensseite.

## 7. Verarbeitungsstatus

~~~text
gefunden → wartet_auf_stabile_datei → geprüft → extrahiert
        → entwurf_erstellt → wartet_auf_freigabe → freigegeben → indiziert
                                      └──────────→ fehlgeschlagen
~~~

Der Status jeder Datei ist in der Web UI sichtbar (farbige Markierung im Ordnerbaum). Fehlgeschlagene Dateien bleiben erhalten und können per Klick erneut verarbeitet werden. Eine SHA-256-Prüfsumme verhindert unnötige Doppelverarbeitung.

## 8. Dokumentverarbeitung

**Startumfang:** PDF mit Text, TXT, Markdown, HTML.
**Danach:** DOCX, PPTX (wichtig für Unterrichtsfolien), CSV/XLSX, OCR für Scans.

Jeder Extrakt-Abschnitt enthält: Abschnitts-ID, Pfad und Hash der Originaldatei, Seite/Folie/Zeilenbereich, Text, Verarbeitungszeit und Extraktor.

## 9. Markdown-Standard

~~~md
---
id: 7b87f6c2-...
titel: Kapitel 01 – Grundlagen
pfad: 1-lehrjahr/modul-117-netzwerk/kapitel-01-grundlagen
status: freigegeben
quellen:
  - datei: skript.pdf
    hash: "sha256:..."
    stellen: ["S. 2", "S. 12"]
generator:
  version: "folder-mind 0.1"
  modell: "gpt-4o-mini"
---

# Kapitel 01 – Grundlagen

## Zusammenfassung
## Wichtige Begriffe
## Zusammenhänge und Beispiele
## Typische Prüfungsfragen
## Offene Punkte
## Quellen
~~~

## 10. Chat und Suche

### 10.1 Chat in der Web UI

- Chatfenster mit Auswahl des **Bereichs**: alle Unterlagen, ein Lehrjahr, ein Modul oder ein Kapitel.
- **Modi** (Auswahl per Knopf):
  - *Erklären* – Begriffe einfach erklären lassen;
  - *Zusammenfassen* – Kapitel kurz zusammenfassen;
  - *Abfragen* – das Modell stellt Lernfragen und prüft die Antworten;
  - *Freier Chat* – allgemeine Fragen ohne Unterlagenbezug (deutlich gekennzeichnet, optional abschaltbar).
- Jede Antwort zeigt die verwendeten Quellen als anklickbare Verweise (Datei + Seite/Folie). Ein Klick öffnet die Fundstelle.
- Chatverläufe können gespeichert, umbenannt und gelöscht werden (gespeichert unter `chats/`).

### 10.2 Belegbasierte Antworten

Folder-Mind sucht zuerst passende Abschnitte (Volltext + optional semantisch) und übergibt nur diese dem Modell. Findet es keine passende Grundlage, antwortet es zum Beispiel:

> In deinen Unterlagen wurde dazu keine belegte Antwort gefunden.

Die verwendeten Abschnitts-IDs werden pro Antwort gespeichert.

### 10.3 Suche

- **Volltextsuche** funktioniert immer, auch ohne Modell.
- **Semantische Suche** nutzt den Embedding-Endpunkt der OpenAI-kompatiblen API (optional, eigenes Embedding-Modell einstellbar).

## 11. KI-Anbindung (OpenAI-kompatible API)

Einstellbar unter **Einstellungen → KI-Modell**:

| Feld | Beispiel | Hinweis |
|---|---|---|
| Anbieter-Vorlage | OpenAI, Ollama, LM Studio, Benutzerdefiniert | füllt die Basis-URL vor |
| Basis-URL | `https://api.openai.com/v1` | muss `/v1/chat/completions` unterstützen |
| API-Key | `sk-...` | Eingabe verdeckt, wird nie wieder im Klartext angezeigt |
| Chat-Modell | `gpt-4o-mini` | Auswahl aus `/v1/models`, falls verfügbar, sonst freie Eingabe |
| Embedding-Modell | `text-embedding-3-small` | optional |
| Temperatur, max. Tokens | 0.2 / 1500 | Erweiterte Einstellungen |

- Knopf **„Verbindung testen“** prüft URL, Key und Modell und zeigt eine verständliche Fehlermeldung (z. B. „API-Key ungültig“, „Modell nicht gefunden“).
- Der API-Key wird verschlüsselt gespeichert (Schlüsselbund des Betriebssystems, ersatzweise verschlüsselte Datei im Ordner `.folder-mind/`) und nie in Protokolle geschrieben.
- **Datenschutz-Hinweis in der Web UI:** Bei einem Cloud-Anbieter werden Textausschnitte aus den Unterlagen an diesen Anbieter gesendet. Vor der ersten Nutzung muss die Person dies bestätigen. Bei einer lokalen URL (127.0.0.1) entfällt der Hinweis.
- Optional: Anzeige des geschätzten Tokenverbrauchs pro Chat.

## 12. Zugriffsschutz der Web UI

### 12.1 Standard: nur das eigene Gerät

- Der Server lauscht standardmässig **nur auf 127.0.0.1 (und ::1)**. Von anderen Geräten ist die Web UI dann technisch gar nicht erreichbar.

### 12.2 Freigabe für andere Geräte (Whitelist)

Unter **Einstellungen → Zugriff** kann die Person den Netzwerkzugriff aktivieren und erlaubte Geräte eintragen:

- **IP-Adresse** (z. B. `192.168.1.25`) oder IP-Bereich (z. B. `192.168.1.0/24`);
- **MAC-Adresse** (z. B. `AA:BB:CC:DD:EE:FF`).

Ablauf:

1. Mindestens ein Eintrag muss vorhanden sein, sonst bleibt der Server auf 127.0.0.1.
2. Erst dann lauscht der Server zusätzlich auf der gewählten Netzwerkschnittstelle.
3. Jede Anfrage wird geprüft: Ist die Quell-IP (oder die per ARP/Nachbartabelle aufgelöste MAC-Adresse) nicht auf der Liste, wird die Verbindung sofort mit „Zugriff verweigert“ abgelehnt und protokolliert.
4. Anfragen von 127.0.0.1 sind immer erlaubt.
5. Die Web UI zeigt eine Liste der zuletzt abgelehnten Zugriffsversuche mit Knopf „Dieses Gerät freigeben“ (mit Bestätigung).

**Wichtige Einschränkungen (werden in der Web UI erklärt):**

- MAC-Adressen sind nur im **gleichen lokalen Netz** (gleiches Subnetz) sichtbar. Über Router oder VPN hinweg funktioniert nur die IP-Whitelist.
- MAC- und IP-Adressen lassen sich fälschen. Deshalb gilt bei aktivem Netzwerkzugriff zusätzlich:
  - Anmeldung mit Passwort (wird beim Aktivieren festgelegt);
  - HTTPS mit automatisch erzeugtem, lokalem Zertifikat;
  - Einstellungen und Zugriffsliste können **nur vom eigenen Gerät (127.0.0.1)** aus geändert werden.

## 13. Schreibbereich und Dateisicherheit

- Bei der Einrichtung wird **ein** Datenordner festgelegt. Das Programm schreibt, verschiebt oder löscht **nur** innerhalb dieses Ordners.
- Jeder Dateipfad wird vor dem Zugriff normalisiert und geprüft (kein `..`, keine symbolischen Links nach aussen, keine absoluten Pfade aus Eingaben). Verstösse werden abgelehnt und protokolliert.
- Originale in `unterlagen/` werden nur gelesen; Ausnahme: Upload, Umbenennen oder Verschieben, das die Person ausdrücklich in der Web UI auslöst.
- Löschen erfolgt nur nach Bestätigung und verschiebt zuerst in einen Papierkorb innerhalb des Datenordners.
- Text in Dokumenten wird nie als Anweisung an das Programm behandelt (Schutz vor Prompt-Injection). Das Modell hat keinen Zugriff auf Dateisystem-Funktionen.
- Ein späterer Wechsel des Datenordners ist in der Web UI möglich (mit Option „vorhandene Daten mitnehmen“).

## 14. Technische Architektur

~~~text
 Browser (Web UI)
      │  HTTP/HTTPS
┌─────▼──────────────────────────────────────────┐
│ Folder-Mind (ein Programm)                     │
│  ├─ Web-Server + Zugriffsprüfung (IP/MAC)      │
│  ├─ Einrichtungsassistent / Einstellungen      │
│  ├─ Chat-Logik + Quellenprüfung                │
│  ├─ Watcher + Auftragswarteschlange            │
│  ├─ Extraktoren (PDF, TXT, MD, HTML …)         │
│  ├─ Pfadwächter (nur Datenordner)              │
│  └─ OpenAI-kompatibler Client                  │
└───────┬─────────────────────┬──────────────────┘
        │                     │
  Datenordner             KI-Anbieter
  (Dateien, SQLite,       (Cloud oder lokal)
   Index)
~~~

### 14.1 Rust-Workspace

~~~text
folder-mind/
├── crates/
│   ├── fm-core/        # Domänenmodell, Regeln, Pfadwächter
│   ├── fm-catalog/     # SQLite und Migrationen
│   ├── fm-watcher/     # Ordnerbeobachtung
│   ├── fm-extract/     # Dokumentformate
│   ├── fm-knowledge/   # Markdown, Entwürfe, Freigaben
│   ├── fm-index/       # Volltext- und semantische Suche
│   ├── fm-ai/          # OpenAI-kompatibler Client (Chat, Embeddings, Models)
│   ├── fm-access/      # IP-/MAC-Whitelist, Anmeldung, TLS
│   ├── fm-web/         # HTTP-API + ausgelieferte Web UI
│   └── fm-app/         # Startprogramm, öffnet Browser
├── web/                # Frontend (in die Binärdatei eingebettet)
├── migrations/
└── tests/
~~~

### 14.2 Bausteine

- **Tokio** – asynchrone Laufzeit;
- **Axum** + **tower** – Web-Server, Middleware für Zugriffsprüfung;
- **rustls** / **rcgen** – HTTPS mit lokalem Zertifikat;
- **notify** + regelmässiger Kontroll-Scan – Ordnerbeobachtung;
- **SQLite** (sqlx oder rusqlite) – Katalog;
- **Tantivy** – Volltextindex;
- **reqwest** – Aufrufe an die OpenAI-kompatible API (Streaming für Chat);
- **keyring** – sichere Ablage des API-Keys;
- **rust-embed** – Web UI in die Programmdatei einbetten;
- **serde**, **toml**, **thiserror**, **tracing**, **sha2**, **uuid**, **argon2** (Passwort).

Frontend: schlicht und leichtgewichtig (z. B. HTML + htmx oder ein kleines Svelte-/Vue-Frontend), Antworten werden per Streaming angezeigt.

## 15. Datenmodell (SQLite)

| Tabelle | Inhalt |
|---|---|
| documents | Originaldatei, Pfad, Grösse, Hash, Typ, Status |
| document_versions | frühere Hashes und Stände |
| chunks | Textabschnitte mit Seite/Folie |
| knowledge_pages | Markdown-Dateien, Status, Quellen |
| jobs | Aufträge |
| chats / messages | Chatverläufe, verwendete Abschnitts-IDs |
| folder_schema | gewählte Vorlage, Ebenennamen, Namensregeln |
| access_rules | IP-/MAC-Einträge, Bezeichnung, Erstellt am |
| access_log | erlaubte/abgelehnte Zugriffe |
| settings | übrige Einstellungen, Schema-Version |

Der Index kann jederzeit aus SQLite, Extrakten und Markdown neu aufgebaut werden.

## 16. Konfiguration

Alle Einstellungen werden ausschliesslich über die Web UI gesetzt. Intern speichert Folder-Mind sie in `.folder-mind/config.toml` (lesbar, aber nicht zum Bearbeiten gedacht):

~~~toml
[paths]
data_dir = "C:/Users/elias/Lernen/folder-mind"

[folders]
template = "lehre"
levels = ["Lehrjahr", "Modul", "Kapitel"]

[ai]
base_url = "https://api.openai.com/v1"
chat_model = "gpt-4o-mini"
embedding_model = "text-embedding-3-small"
api_key_ref = "keyring:folder-mind"   # kein Klartext

[access]
network_enabled = false
bind = ["127.0.0.1", "::1"]
allowed_ips = []
allowed_macs = []

[automation]
stable_file_wait_seconds = 30
max_ai_jobs = 1
auto_approve = false

[quality]
require_citations = true
allow_free_chat = true
~~~

Der Ort der Konfiguration selbst (Zeiger auf den Datenordner) liegt im Benutzerprofil des Betriebssystems, damit das Programm beim Start den Datenordner findet.

## 17. Web UI – Seitenübersicht

| Seite | Inhalt |
|---|---|
| Einrichtung | Assistent beim ersten Start (Abschnitt 4.1) |
| Start | Übersicht: neue Dateien, offene Freigaben, Fehler, letzte Chats |
| Unterlagen | Ordnerbaum mit Status, Upload per Drag & Drop, Ordner anlegen/umbenennen |
| Kapitel | Entwurf, freigegebene Seite, eigene Notizen und Quellen nebeneinander |
| Chat | Chat mit Bereichs- und Modusauswahl, Quellenverweise |
| Suche | Volltext und semantische Suche |
| Warteschlange | laufende Aufträge, Pause, Wiederholen, Neuindizieren |
| Einstellungen | Datenordner, Ordnerstruktur, KI-Modell, Zugriff, Automatisierung |
| Sicherung | Backup erstellen, prüfen, wiederherstellen |
| Protokoll | Verarbeitungs- und Zugriffsprotokoll |

## 18. Hintergrundbetrieb

- Leichte, mittlere und schwere Aufgaben werden getrennt; standardmässig läuft nur ein KI-Auftrag gleichzeitig.
- Pause-Knopf, Zeitfenster und maximale Dateigrösse in der Web UI einstellbar.
- Optional: Autostart beim Anmelden (in der Web UI aktivierbar).

## 19. Sicherung und Wiederherstellung

Über die Seite **Sicherung**: Backup als ZIP innerhalb von `backups/` (Originale, Wissen, Notizen, Chats, Datenbank, Konfiguration ohne API-Key). Wiederherstellung mit Vorschau („Probelauf“) und anschliessendem Neuaufbau des Index.

## 20. Qualitätsregeln

1. Das Modell darf keine Ordnerpfade ändern und keine Dateien verschieben.
2. Schreibzugriffe ausserhalb des Datenordners sind technisch blockiert.
3. Eine freigegebene Wissensseite hat mindestens eine Quelle.
4. Eine Antwort im Unterlagen-Modus ohne Quellenbeleg wird verworfen.
5. Eigene Notizen werden nie automatisch verändert.
6. Der API-Key erscheint nie in Protokollen, Backups oder der Web UI im Klartext.
7. Ohne Whitelist-Eintrag ist die Web UI nur lokal erreichbar.

## 21. Teststrategie

- **Unit-Tests:** Pfadwächter (inkl. `..` und Symlinks), Whitelist-Prüfung (IPv4, IPv6, CIDR, MAC), Statuswechsel, Markdown.
- **Integrations-Tests:** Einrichtungsassistent komplett über die HTTP-API durchspielen; Musterdateien verarbeiten.
- **KI-Tests:** Mock-Server mit OpenAI-kompatibler API (gültig, falscher Key, Zeitüberschreitung, fehlerhafte Antwort).
- **Sicherheitstests:** Zugriff von nicht freigegebener IP wird abgelehnt; Einstellungen von entferntem Gerät aus nicht änderbar.
- **Retrieval-Tests:** Fragenkatalog mit erwarteten Quellen und bewusst nicht beantwortbaren Fragen.

## 22. Umsetzungsplan

### Version 0.1 – Web UI und Fundament
- Programm startet Web-Server auf 127.0.0.1 und öffnet den Browser;
- Einrichtungsassistent: Datenordner, Standard-Ordnerstruktur;
- Pfadwächter, SQLite, Scan, Status, Ordnerbaum in der Web UI;
- PDF-Text, TXT, Markdown, HTML; Volltextsuche.

### Version 0.2 – KI-Chat
- Einstellungen KI-Modell mit Verbindungstest, sichere Key-Ablage;
- Chat mit Bereichsauswahl, Streaming und Quellenverweisen;
- Entwürfe und Freigabe in der Web UI.

### Version 0.3 – Organisation und Zugriff
- Ordnerstruktur-Editor und Vorlagen; Upload per Drag & Drop;
- Netzwerkzugriff mit IP-/MAC-Whitelist, Passwort und HTTPS;
- Hintergrund-Watcher, Warteschlange, Pause.

### Version 0.4 – Komfort
- DOCX, PPTX, XLSX, OCR; semantische Suche;
- Lernmodus „Abfragen“, Sicherung/Wiederherstellung in der Web UI, Autostart.

## 23. Abnahmekriterien für Version 0.1

- Nach dem ersten Start ist die komplette Einrichtung ohne Kommandozeile möglich.
- Die Web UI ist von einem anderen Gerät im Netz **nicht** erreichbar.
- Eine Datei im richtigen Kapitelordner wird erkannt, genau einmal verarbeitet und im Ordnerbaum angezeigt.
- Versuche, ausserhalb des Datenordners zu schreiben, werden blockiert und protokolliert.
- Die Volltextsuche findet Begriffe aus den Unterlagen.

## 24. Offene Entscheidungen

1. Welches Frontend-Framework (htmx vs. Svelte/Vue)?
2. Soll die MAC-Whitelist schon in 0.3 kommen oder zuerst nur IP?
3. Soll der „Freie Chat“ ohne Unterlagenbezug standardmässig aktiv sein?
4. Welche Ordnervorlagen werden mitgeliefert (Lehre Informatik, KV, allgemein)?
5. Welche Lizenz soll das Projekt erhalten?

## 25. Referenzen

- [Tokio](https://tokio.rs/) · [Axum](https://docs.rs/axum/latest/axum/) · [notify](https://docs.rs/notify/latest/notify/) · [Tantivy](https://docs.rs/tantivy/latest/tantivy/)
- [OpenAI API Referenz (Chat Completions, Embeddings, Models)](https://platform.openai.com/docs/api-reference)
