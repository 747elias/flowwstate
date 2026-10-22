# Folder-Mind 0.5

Folder-Mind verwaltet Lernunterlagen in einem lokalen Datenordner und bietet Volltextsuche, einen KI-Chat mit Quellen und überprüfbare Wissensentwürfe. Die deutsche Weboberfläche ist vollständig in der Programmdatei enthalten.

## Installation und erster Start

1. `folder-mind.exe` auf einen Windows-10/11-Rechner mit x86_64-Prozessor herunterladen. Die EXE benötigt weder Python noch Node, Office, einen separaten Datenbankserver oder mitgelieferte DLLs.
2. EXE doppelklicken. Der Browser öffnet `http://127.0.0.1:8484`.
3. Im Assistenten Namen, Datenordner und eine Vorlage wählen. Der Ordner-Browser kann Laufwerke und UNC-Pfade öffnen, navigieren und nach ausdrücklichem Auftrag einen Ordner anlegen. Pfade mit Leerzeichen sind möglich.
4. Optional einen KI-Anbieter verbinden. Ohne Modell funktionieren Ablage und Volltextsuche.
5. Optional weitere Geräte freigeben; für den normalen lokalen Betrieb alles deaktiviert lassen.
6. Zusammenfassung prüfen und Einrichtung abschliessen.

Die EXE ist nicht codesigniert. Windows oder betriebliche Richtlinien können deshalb eine Prüfung oder Freigabe verlangen. Es wird kein Administratorrecht für den normalen Betrieb benötigt.

Im Infobereich von Windows befindet sich das Tray-Icon mit **Öffnen**, **Pause/Fortsetzen** und **Beenden**. Ohne Einrichtung führt jede Anwendungsseite zum Assistenten. Der Ort des Datenordners wird unter `%APPDATA%\folder-mind\data-dir.txt` gespeichert. Vor der Einrichtung werden keine Lernunterlagen geschrieben.

## Update von 0.4.1 auf 0.5 ohne Datenverlust

1. In der bisherigen Version bei Bedarf unter **Sicherung** eine Sicherung erstellen. Gewünschte Chatverläufe vorher speichern; nicht gespeicherte oder noch nicht abgesendete Eingaben sind keine Datenbankinhalte.
2. Die bisherige Version über das Tray-Menü **Beenden** schliessen.
3. Die neue `folder-mind.exe` herunterladen, die bisherige EXE ersetzen und die neue EXE mit demselben Windows-Benutzer starten.
4. Folder-Mind übernimmt den bisherigen Datenordner automatisch. Der Assistent erscheint nicht erneut. Unterlagen, gespeicherte Chats, Notizen, Anbieter-Einstellungen und der Verweis auf den API-Key im Windows Credential Manager bleiben erhalten.

Vor der Datenbank-Erweiterung werden die bisherige SQLite-Datenbank und die unveränderte Konfiguration automatisch als `backups/vor-update-*.sqlite` und `*.toml` gesichert. Die Migration läuft in einer Transaktion und entfernt keine bisherigen Tabellen oder Inhalte. Originaldateien werden dabei nicht verändert. Ein aktivierter Windows-Autostart wird auf die gestartete EXE aktualisiert.

Ab 0.5 bietet **Update → Sichern & für Update beenden** denselben Ablauf mit einer vollständigen ZIP-Sicherung vor dem Beenden. Anschliessend die EXE austauschen und neu starten. Der Austausch der EXE allein löscht keine gespeicherten Nutzerdaten. Nach dem Update keine ältere EXE mit dem aktualisierten Datenordner starten. Bei Beständen über der ZIP-Sicherungsgrenze den vollständigen Datenordner vor dem Update zusätzlich im Explorer kopieren.

## Bedienung

- **Start:** Dateizahl, offene Wissensfreigaben, Fehler und aktiver Datenordner.
- **Eingang:** Explorer-Ordner öffnen, automatische Zuordnung beobachten, Rückfragen beantworten, Regeln und Beispiele einlernen und den Verschiebeverlauf prüfen.
- **Unterlagen:** Zielordner wählen, Dateien auswählen oder hineinziehen; Ordner anlegen, umbenennen, leere Ordner löschen und Dateien verschieben. Löschen erfordert Bestätigung und verschiebt in `.papierkorb/`.
- **Kapitel:** Entwurf erzeugen, prüfen und freigeben. Eigene Notizen werden separat gespeichert und bei der KI-Verarbeitung nicht überschrieben.
- **Chat:** Agent und Bereich wählen. Agenten und gespeicherte Gespräche stehen links, das Gespräch in der Mitte und die Eingabe unten. Antworten zeigen Markdown mit Überschriften, Listen, Tabellen, Code und anklickbaren Quellen. Neue Chats werden standardmässig gespeichert; die Auswahl kann abgeschaltet werden. Nicht gespeicherter Gesprächskontext bleibt nur im Browser.
- **Agenten:** Erklärer, Ablage, Agent-Builder, Zusammenfasser, Lerntutor und freier Chat. Eigene Agenten lassen sich über den Builder entwerfen oder mit Name, Anweisungen, Quellenmodus und Standardbereich direkt erstellen, bearbeiten und löschen. Gespeicherte Chats bleiben beim Löschen eines Profils erhalten.
- **Update:** Neue EXE herunterladen, Sicherung erstellen und die Anwendung zum Austauschen der EXE beenden.
- **Suche:** Volltextsuche funktioniert ohne KI. Für semantische Suche ein Embedding-Modell einstellen und den semantischen Index aufbauen. Nach Dokumentänderungen kann dieser erneut aufgebaut werden; alte Vektoren werden bei der Suche nicht für neue Dateiversionen verwendet.
- **Warteschlange:** Status und Fehler ansehen, pausieren, wiederholen oder den Volltextindex neu aufbauen.
- **Einstellungen:** Datenordner, Vorlagen, ein bis vier Ebenennamen, Namensregeln, KI, Netzwerk, Dateigrösse, Zeitfenster, Freigabe und Autostart ändern.
- **Sicherung:** ZIP erstellen, herunterladen, prüfen, einen Probelauf ansehen und die Wiederherstellung ausdrücklich bestätigen.
- **Protokoll:** Verarbeitungs- und Zugriffsprotokolle ansehen. Dateien liegen unter `<Datenordner>\protokolle\`.

Die Vorlagen heissen **Lehre Informatik**, **Lehre allgemein**, **Schule** und **Frei**. Neue Ordner können automatisch Kleinbuchstaben und Bindestriche statt Leerzeichen verwenden. Die tatsächliche Ordnerstruktur bestimmt die Zuordnung; das Modell verschiebt keine Dateien.

### Automatische Ablage im Input-Ordner

Lege Dateien oder vollständige Ordner mit Unterordnern in `<Datenordner>\input\`. Die Hintergrundverarbeitung wartet auf stabile Dateien und berücksichtigt Dateiname, Ordnerkontext, bei unterstützten Formaten einen Textauszug sowie deine gespeicherten Regeln und Beispiele. Bei eindeutigem Ziel werden passende Ordner unter `unterlagen/` angelegt und die Dateien unverändert dorthin verschoben. Bestehende gleichnamige Dateien bleiben erhalten; neue Dateien erhalten bei Bedarf einen nummerierten Namen.

Alle Dateitypen können abgelegt und nach Metadaten zugeordnet werden. Für Formate ohne Textextraktion stehen inhaltliche Suche, Zusammenfassung und Fragen zum Dateiinhalt nicht zur Verfügung; ihr Status lautet **abgelegt**. Für Zuordnungsausschnitte werden unterstützte Dateien bis 10 MB gelesen. Grössere Dateien können anhand von Metadaten und Regeln zugeordnet werden; die eingestellte Maximalgrösse für die anschliessende Textextraktion gilt weiterhin.

- **Regeln:** Schlüsselwort und/oder Dateiendung mit Zielordner verknüpfen. Sind beide angegeben, müssen beide zutreffen. Widersprüchliche Ziele führen zu einer Rückfrage.
- **KI-Hinweise:** Zusätzliche freie Anweisungen mit Zielordner speichern. Sie werden dem eingerichteten Modell bei der Zuordnung mitgegeben.
- **Rückfragen:** Zielordner in der Web UI auswählen oder neu benennen und bestätigen. Optional die Zuordnung als Beispiel und ein Schlüsselwort als neue Regel merken. Ohne eingerichtetes Modell bleiben unklare Dateien für diese Entscheidung im Eingang.
- **Lernen:** Bestätigte Beispiele bleiben lokal gespeichert; bis zu 30 der zuletzt bestätigten Beispiele werden künftigen KI-Anfragen mitgegeben. Identische Dateiinhalte können direkt anhand eines bestätigten Beispiels zugeordnet werden. Das Modell selbst wird nicht neu trainiert. Regeln und Beispiele können entfernt werden.
- **Verlauf:** Quelle, Ziel, Zeitpunkt und Begründung bleiben gespeichert, auch nach einem Neustart. Unveränderte, automatisch abgelegte Dateien lassen sich von dort zurück in den Eingang legen. Sie warten dann auf eine neue Entscheidung.

Automatisches Verschieben ist standardmässig aktiv und unter **Einstellungen → Automatische Eingangsablage** abschaltbar. Die KI-Sicherheitsschwelle beträgt standardmässig 0,9. Das ist eine Einschätzung des Modells, keine Garantie. Bei Unsicherheit oder einer Rückfrage wird nicht automatisch verschoben. KI-Aufrufe verwenden denselben Anbieter und dieselbe Cloud-Zustimmung wie der Chat und können Kosten verursachen.

### ZIP-Regel

Ausschliesslich Dateien mit dem **exakten, kleingeschriebenen Namen `unzip.zip` im Input-Ordner** werden automatisch entpackt. Das gilt auch für Unterordner. Andere Namen, beispielsweise `material.zip` oder `Unzip.zip`, bleiben ZIP-Dateien und werden als Datei zugeordnet.

Das Originalarchiv bleibt unter `eingangsarchive/<Vorgang>/unzip.zip` erhalten. Entpackte Dateien durchlaufen dieselbe Zuordnung wie andere Eingänge. Absolute Pfade, `..`, Windows-Gerätenamen, Links, Spezialdateien und doppelte Pfade werden abgewiesen. Grenzen: 512 MB komprimiert und entpackt pro Archiv, höchstens 10.000 Einträge und drei verschachtelte ZIP-Ebenen. Bei einem Fehler bleibt das Original erhalten; die Web UI zeigt eine Rückfrage.

### Dateiaktionen im Chat

Der Agent **Ablage** beantwortet beispielsweise „Wo ist mein Netzwerk-Dokument gespeichert?“ oder „Verschiebe skript.pdf nach unterlagen/netzwerk“. Er arbeitet mit Dateilisten und bereitet typisierte Vorschläge vor. **Verschieben bestätigen** zeigt Quelle und Ziel und führt die Aktion erst nach deinem Klick aus. Eine zwischenzeitlich geänderte Datei wird abgewiesen. Der Agent erklärt keine Dokumentinhalte. Das Modell erhält weiterhin keine Dateisystem-Werkzeuge.

Der **Agent-Builder** erstellt einen überprüfbaren Agentenentwurf. Erst **Agenten übernehmen** speichert das Profil; danach erscheint es in der Agentenauswahl. Für neue Profile und deren Bearbeitung ist der lokale Administrationszugang über 127.0.0.1 erforderlich.

### Verarbeitung und Quellen

Der Watcher meldet Änderungen; zusätzlich läuft alle fünf Sekunden ein Kontrollscan. Standardmässig muss eine Datei 30 Sekunden unveränderte Grösse und Änderungszeit aufweisen. Metadaten werden nach dem Lesen erneut geprüft. SHA-256 verhindert erneute Verarbeitung identischer Inhalte. Fehlerhafte Originale bleiben erhalten und können erneut verarbeitet werden.

Unterstützt werden PDF mit Text, TXT/UTF-8, Markdown, HTML, DOCX, PPTX, XLSX, CSV sowie Bild-OCR. Windows-OCR erkennt Bilder und textlose PDF-Seiten. Die Texterkennung verwendet die im Windows-Benutzerprofil verfügbaren OCR-Sprachen; gesperrte oder beschädigte Dateien werden verständlich als Fehler angezeigt. Office-Extraktion benötigt keine Office-Installation. Layout, Bilder und Diagramme in Office-Dokumenten werden nicht als Text interpretiert.

Quellen enthalten Abschnitts-ID, Originalpfad, Hash, Seite/Folie/Zeilen bzw. Abschnitt, Zeit und Extraktor. PDF-Verweise öffnen die Seite. Andere Quellen zeigen den konkreten Extrakt und einen Download des Originals. Frühere Extrakte werden im Katalog archiviert, damit gespeicherte Chat-Belege bei späteren Dateiänderungen nachvollziehbar bleiben. Sie erscheinen nicht in der aktuellen Volltextsuche.

Eine freigegebene Wissensseite benötigt einen gültigen Quellenbeleg. Die Markdown-Dateien enthalten Metadaten und die Abschnitte Zusammenfassung, Wichtige Begriffe, Zusammenhänge und Beispiele, Typische Prüfungsfragen, Offene Punkte und Quellen. Fehlende Modellabschnitte werden ausdrücklich als ohne weitere belegte Angaben gekennzeichnet. Quellenprüfung kontrolliert gültige Abschnittskennungen; sie ist kein automatischer Beweis der sachlichen Richtigkeit einer Modellantwort.

## KI-Anbieter

| Vorlage | Basis-URL |
| --- | --- |
| OpenAI | `https://api.openai.com/v1` |
| Ollama | `http://127.0.0.1:11434/v1` |
| LM Studio | `http://127.0.0.1:1234/v1` |
| Benutzerdefiniert | eigene OpenAI-kompatible URL |

„Verbindung testen“ prüft die Modellliste und bei gewähltem Modell auch einen Streaming-Chat-Aufruf. Unterstützt werden `/models`, `/chat/completions` mit SSE und `/embeddings` relativ zur Basis-URL. Modellnamen können frei eingegeben oder aus der abgerufenen Liste gewählt werden.

Externe Anbieter benötigen HTTPS und eine ausdrückliche Zustimmung zur Übermittlung von Textausschnitten. Lokale Anbieter können HTTP verwenden. Der API-Key liegt ausschliesslich im **Windows Credential Manager** des angemeldeten Benutzers. Die Konfiguration enthält nur eine Referenz. Ein leeres Schlüsselfeld lässt den bisherigen Schlüssel unverändert; zum Entfernen gibt es eine eigene Auswahl. Bei einem Fehler des Credential Managers wird kein Klartext-Ersatz angelegt.

Das Modell erhält ausgewählte Textabschnitte als unzuverlässige Daten und keine Dateisystem-Werkzeuge. Antworten mit Unterlagenbezug ohne gültigen Quellenbeleg werden verworfen. Fehlt schon bei der Suche eine passende Grundlage, erscheint die entsprechende deutsche Meldung ohne KI-Aufruf. KI-Aufträge für Chat, Agent-Builder, Eingangszuordnung, Entwürfe und Embeddings werden seriell ausgeführt.

## Netzwerk, Passwort und MAC-Freigaben

Standardmässig lauscht HTTP ausschliesslich auf `127.0.0.1:8484` und, soweit verfügbar, `[::1]:8484`. Es wird keine Firewall-Regel automatisch geöffnet.

Netzwerkzugriff erfordert gleichzeitig mindestens einen gültigen IP/CIDR- oder MAC-Eintrag, ein Passwort mit mindestens zwölf Zeichen und HTTPS. Der zusätzliche HTTPS-Listener verwendet standardmässig `0.0.0.0:8485`. Auf anderen Geräten lautet die Adresse `https://IP-DES-WINDOWS-RECHNERS:8485`. Für IPv6 eine Adresse wie `[::]:8485` als Schnittstelle verwenden. Lokales HTTP bleibt für die Verwaltung erreichbar.

Das selbst erzeugte Zertifikat liegt mit seinem privaten Schlüssel in `.folder-mind/`. Den angezeigten SHA-256-Fingerabdruck des Zertifikats auf dem vertrauenswürdigen Rechner prüfen, bevor das Zertifikat auf einem anderen Gerät akzeptiert wird. Sitzungen verwenden ein sicheres, nur für HTTP verfügbares SameSite-Strict-Cookie, gelten acht Stunden und sind an die Quell-IP gebunden. Anmeldeversuche sind begrenzt. Das Passwort wird mit Argon2 gehasht.

**MAC-Adressen funktionieren nur im gleichen lokalen Netz.** Die Anwendung liest `GetIpNetTable2` aus der Windows-Nachbartabelle. Über Router oder VPN ist die MAC des entfernten Geräts normalerweise nicht sichtbar. Dann eine einzelne IP oder ein CIDR-Netz freigeben. Der Knopf „MAC-Adresse nachschlagen“ erklärt Fehler in der Weboberfläche. Eine noch nicht gefüllte oder veraltete Nachbartabelle kann ebenfalls die Auflösung verhindern.

IP und MAC sind fälschbar; sie ersetzen Passwort und HTTPS nicht. Einstellungen, Anbieter-Tests, Ordnerauswahl, Datenordnerwechsel, Sicherungen und Whitelist-Verwaltung sind ausschliesslich über **127.0.0.1** möglich. Auch `::1` erhält keine Administrationsrechte. Weitergeleitete IP-Header werden nicht als Quell-IP vertraut. Abgelehnte Versuche werden protokolliert und können lokal nach Bestätigung freigegeben werden.

Alle verändernden HTTP-Anfragen benötigen das CSRF-Token. Die Oberfläche lädt ihre Skripte und Styles lokal; Host-Prüfung und Content Security Policy ergänzen den Zugriffsschutz.

## Daten, Sicherungen und Wiederherstellung

```text
<Datenordner>/
  input/            Eingangsdateien, auch verschachtelte Ordner
  eingangsarchive/  unveränderte automatisch entpackte ZIP-Originale
  unterlagen/       Originale
  extrakte/         wiederherstellbare Extrakte mit Fundstellen
  wissen/           *.generated.md, *.notes.md, *.md
  chats/            gespeicherte Verläufe
  protokolle/       Laufzeitprotokoll
  backups/          ZIP-Sicherungen
  .papierkorb/      gelöschte Dateien/Ordner
  .folder-mind/     config.toml, SQLite, Tantivy, Vektoren, lokales TLS
```

Die interne TOML-Konfiguration enthält die Bereiche `paths`, `folders`, `ai`, `access`, `automation` und `quality`. Sie wird über die Weboberfläche verwaltet. Der Pfadwächter verwendet den tatsächlich gewählten und kanonisierten Datenordner; ein Textwert in der Konfiguration erweitert den Schreibbereich nicht.

ZIP-Sicherungen enthalten Originale, Extrakte, Wissen, Notizen, Chats, Konfiguration und einen konsistenten SQLite-Snapshot. Ein Manifest enthält SHA-256-Prüfsummen. Volltextindex, API-Key, TLS-Privatschlüssel, Papierkorb und frühere ZIPs sind nicht enthalten. Der Volltextindex wird aus dem Katalog neu erzeugt; semantische Vektoren können neu angefordert werden.

Vor der Wiederherstellung zeigt ein Probelauf Anlegen/Ersetzen und den Archiv-Hash. Die Bestätigung gilt nur für diesen unveränderten Archivstand. Vorhandene zusätzliche Dateien bleiben erhalten. Vor dem Ersetzen wird der aktuelle Stand gesichert. Netzwerkzugriff, Cloud-Zustimmung und Autostart werden zurückgesetzt; der vorhandene Credential-Manager-Verweis bleibt lokal erhalten. Der API-Key muss auf einem anderen Rechner neu eingegeben werden.

Beim Datenordnerwechsel können vorhandene Daten in einen leeren Zielordner kopiert oder ein neuer/bestehender Folder-Mind-Ordner gewählt werden. Die bisherigen Daten werden nicht gelöscht. Netzwerkzugriff wird dabei deaktiviert.

## Fehlerbehebung

| Problem | Vorgehen |
| --- | --- |
| Browser öffnet nicht | Tray → Öffnen oder `http://127.0.0.1:8484` aufrufen. |
| Port 8484 belegt | Eine bereits laufende gleiche Version wird geöffnet. Bei einem Update die alte Version zuerst im Tray beenden. Bei einem anderen Programm dieses schliessen oder dessen Port ändern. |
| Datenordner nicht verfügbar | Laufwerk/UNC-Verbindung wiederherstellen. Ein Startfehler erscheint als Windows-Meldung; die Daten werden nicht an einen anderen Ort geschrieben. |
| Datei bleibt wartend | Kopiervorgang beenden und Stabilitätswartezeit abwarten. Pause und Zeitfenster kontrollieren. |
| Extraktion fehlgeschlagen | Leserechte, Dateigrösse, Verschlüsselung und Format prüfen; anschliessend Warteschlange → Erneut verarbeiten. |
| OCR fehlt | Verfügbare Windows-OCR-Sprache prüfen oder eine PDF mit eingebettetem Text verwenden. Die Anwendung lädt keine externen OCR-Programme nach. |
| API-Key ungültig | Schlüssel und Berechtigungen beim Anbieter prüfen und über Einstellungen ersetzen. |
| Modell nicht gefunden | `/models` prüfen oder den genauen Modellnamen eingeben. |
| Zeitüberschreitung | Lokalen Modellserver starten bzw. URL und Netzwerkverbindung kontrollieren. |
| Keine Quellen / Antwort verworfen | Suchbegriff, Bereich, Dokumentverarbeitung und Modellfähigkeit zum Zitieren prüfen. |
| Alte Quelle | Der archivierte Extrakt bleibt sichtbar; ein geändertes Original lässt sich nicht als alte Dateiversion herunterladen. |
| Semantischer Index passt nicht | Nach Anbieter-/Modellwechsel den semantischen Index neu aufbauen. |
| Remote-Zugriff verweigert | Lokal Whitelist und abgelehnte Versuche prüfen; für geroutete Verbindungen IP statt MAC verwenden. |
| HTTPS nicht erreichbar | Schnittstelle, Port, Firewall, Whitelist und die Statusmeldung unter Zugriff prüfen. |
| Einstellungen über IPv6 gesperrt | Verwaltung ausdrücklich über `http://127.0.0.1:8484` öffnen. |

## Aus Quellcode bauen

Zum **Bauen** werden Rust stable mit MSVC-Toolchain und die Microsoft C++ Build Tools/Windows SDK benötigt. Für die Nutzung der fertigen EXE sind diese Werkzeuge nicht erforderlich.

```powershell
rustup target add x86_64-pc-windows-msvc
.\build-windows.ps1
```

Das Skript prüft Formatierung, Tests und Clippy und führt danach aus:

```powershell
cargo build --release --locked --target x86_64-pc-windows-msvc
```

Ergebnis: `target\x86_64-pc-windows-msvc\release\folder-mind.exe`. Die C-Laufzeit ist statisch eingebunden, der Release-Build verwendet das Windows-GUI-Subsystem. HTML, htmx und JavaScript werden durch `rust-embed` eingebettet; ein Frontend-Build mit Node ist nicht erforderlich.

Die fertige Windows-EXE steht unter [GitHub Releases](https://github.com/747elias/folder-mind/releases). `.github/workflows/windows.yml` führt auf `windows-latest` Build, Tests und Clippy aus und liefert die EXE als Artefakt. Die [aktuellen Prüfläufe](https://github.com/747elias/folder-mind/actions/workflows/windows.yml) sind auf GitHub einsehbar. `.gitattributes` verhindert insbesondere, dass Git die binären PDF-Beispiele beim Windows-Checkout durch Zeilenendenkonvertierung beschädigt.

Der Workspace enthält die zehn Crates aus Abschnitt 14.1. Direkte Abhängigkeiten sind exakt festgelegt; `Cargo.lock` fixiert den aufgelösten Graphen. Unter Linux sind Entwicklung und portable Tests möglich, Credential Manager, Tray, MAC-Auflösung, Autostart und OCR sind Windows-Funktionen. Zum isolierten Entwickeln können `FOLDER_MIND_POINTER` und `FOLDER_MIND_NO_BROWSER=1` gesetzt werden.

## Prüfungen und Abnahme

Die Versionsstände 0.1 bis 0.4 werden mit Release-Build, Tests und `cargo clippy --workspace --all-targets -- -D warnings` geprüft. Protokolle stehen unter `docs/pruefungen/`. Der lokale Rust-Mock-Server in `crates/fm-ai/tests/mock.rs` deckt Modelle, SSE, Embeddings und Fehlerfälle ab. HTTP-Integrationstests prüfen Einrichtung, Zugriffsschutz, Quellen, Entwürfe und Sicherung/Wiederherstellung.

Die Beispieldateien unter `tests/fixtures/` sind eigens erzeugt: PDF, TXT, Markdown, HTML und Office-Beispiele. Es werden keine fremden Unterrichtsunterlagen mitgeliefert.

Die Erweiterungsprüfung steht unter [`docs/ABNAHME-0.5.md`](docs/ABNAHME-0.5.md). Die ursprüngliche Prüfübersicht und die fünf Abnahmekriterien aus Abschnitt 23 stehen in `docs/ABNAHME.md`. Native Windows-Laufzeittests werden dort von erfolgreichen Cross-Builds unterschieden.

## Abweichungen vom Konzept

- **Geprüfte SSE-Ausgabe:** Die API des Anbieters wird per echtem SSE gelesen. Die Antwort wird vor der Anzeige vollständig gesammelt, auf gültige Quellen-IDs geprüft und gegen Schlüssel-Echos bereinigt. Erst danach liefert Folder-Mind SSE-Blöcke an den Browser. Damit werden unbestätigte Textteile nicht vorzeitig angezeigt; die Ausgabe beginnt entsprechend später.
- **Schlankes Frontend ohne Build-Schritt:** HTML und htmx übernehmen Oberfläche/Statusabruf; Vanilla-JavaScript verarbeitet Assistent, Formulare, Upload und Fetch-SSE. Es gibt keine zusätzliche Frontend-Laufzeit und kein Node-Build-Erfordernis.
- **Automatische Zuordnung auf ausdrücklichen Erweiterungsauftrag:** Ab Version 0.5 dürfen Eingangsdateien durch die geprüfte Backend-Ablage automatisch verschoben werden. Dies erweitert die ursprüngliche Abgrenzung aus Abschnitten 3, 5 und 20 des Konzepts. Das Modell selbst bleibt ohne Dateisystem-Werkzeuge; Aktionen an bereits abgelegten Dateien benötigen weiterhin eine ausdrückliche Chat-/Web-UI-Bestätigung.
- **Explizite Wissensentwürfe:** Wissensentwürfe werden über „Entwurf erstellen“ angefordert. Automatische Eingangszuordnung kann dagegen KI-Aufrufe auslösen. Die optionale automatische Freigabe gilt für diese angeforderten, belegten Entwürfe.
- **Begrenzter Modellkontext:** Chat-Retrieval nutzt bis zu acht, der Zusammenfasser eine über den Bereich verteilte Auswahl von bis zu 24, ein Wissensentwurf bis zu 24 Abschnitte. Für umfangreiche Themen kleinere Kapitelbereiche verwenden. Eine optionale Tokenverbrauchsanzeige ist nicht enthalten.
- **ZIP-Grenze:** Erstellung und Prüfung von Sicherungen sind auf 512 MB unkomprimierten Inhalt und 10.000 Einträge begrenzt, um Speicherbedarf und ZIP-Bomben zu begrenzen. Grössere Bestände müssen auf Datenordner aufgeteilt werden.
- **Zusammenführende Wiederherstellung:** Nicht im Archiv enthaltene zusätzliche Dateien werden erhalten. Die Vorschau macht das ausdrücklich sichtbar.
- **Windows-Hintergrundbetrieb:** Tray und Benutzer-Autostart sind implementiert. Eine Installation als registrierter Windows-SCM-Dienst ist nicht enthalten.
- **Notwendige Ausnahmen zum Datenordner:** Der Benutzerprofil-Zeiger, Credential-Manager-Einträge und der ausdrücklich aktivierte HKCU-Autostart liegen betriebssystembedingt ausserhalb des Datenordners. Beim Einrichten/Auswählen darf nach ausdrücklichem Auftrag ein neuer Zielordner angelegt werden. Normale Schreib-, Verschiebe- und Löschoperationen bleiben an den zentralen Pfadwächter gebunden.
- **Prüfumgebung:** Die Lieferung wird unter Linux gebaut und im Browser getestet; Windows-Build, automatisierte Tests und Clippy laufen zusätzlich nativ auf GitHub Actions. Interaktive Windows-Prüfungen von Tray, Credential Manager, OCR, Nachbartabelle und Autostart bleiben gesondert durchzuführen.

## Lizenz

Folder-Mind steht unter MIT, siehe `LICENSE`. Drittanbieter behalten ihre jeweiligen Lizenzen. Die mitgelieferte htmx-Lizenz steht in `web/HTMX-LICENSE.txt`; weitere eingebettete Lizenztexte stehen in `web/third-party.txt`.
