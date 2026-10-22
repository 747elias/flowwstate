# Abnahme der Erweiterung 0.5

## Umgesetzte Anforderungen

| Anforderung | Status | Nachweis |
| --- | --- | --- |
| Vorhandene Daten mit neuer EXE weiterverwenden | erfüllt in den automatisierten Prüfungen | Echtes Linux-Upgrade aus der gebauten 0.4.1 mit bestehendem Datenordner, Namen, Anbieter-/Schlüsselverweis, Originaldatei, Notiz und gespeichertem Chat. Zusätzlich Migration einer Datenbank mit Schema 2 einschliesslich Sicherung und Transaktion. |
| Update in der Web UI vorbereiten | erfüllt | Sicherung, Schreibpause, Programmende und anschliessender Neustart mit erhaltenen Daten im Browsertest; HTTP-Test prüft die Sperre weiterer Änderungen. EXE-Austausch erfolgt anschliessend manuell. |
| Übersichtliche Chat-Anordnung | erfüllt | Agenten und Gespräche links, Gespräch mittig, Eingabe unten; mobile Agentenauswahl und zwölf Ansichten bei 390 Pixel ohne horizontalen Seitenüberlauf. |
| Markdown-Antworten | erfüllt | Überschriften, Hervorhebungen, Listen, Tabellen, Code und anklickbare Quellen im aktuellen und im gespeicherten Chat. Unit-/HTTP-/Browsertests blockieren ausführbares HTML und gefährliche Links. |
| Ablage-Agent | erfüllt | Dateisuche nach Namen und Volltexttreffern; konkrete Verschiebevorschläge mit Quelle/Ziel; Bestätigung vor Ausführung, Hash-Abgleich, Schutz bestehender Ziele und Verlauf. Kein Dokument-Erklärmodus und keine Dateisystem-Werkzeuge für das Modell. |
| Agent-Builder und eigene Agenten | erfüllt | KI-Entwurf mit ausdrücklicher Übernahme, direktes Erstellen/Bearbeiten/Löschen und dauerhafte Speicherung. Systemprofile sind geschützt. |
| Zusammenfasser | erfüllt | Eigener auswählbarer Agent mit Bereichsauswahl und geprüften Quellen. Bei grossen Bereichen wird eine verteilte Auswahl von maximal 24 Textabschnitten genutzt; diese Grenze ist in der Oberfläche sichtbar. |
| Input-Ordner mit Unterordnern und allen Dateitypen | erfüllt für Ablage und Zuordnung | Rekursiver Import von verschachtelten Dateien und unveränderten Binärdateien; vorhandene gleichnamige Dateien werden nicht ersetzt. Inhalte nicht unterstützter Formate werden nicht interpretiert. |
| Nur genau unzip.zip automatisch entpacken | erfüllt | Browsertest mit entpacktem Archiv und unverändert abgelegtem normalem ZIP; Originalarchiv bleibt erhalten. Unit-Tests prüfen Traversierung, Laufwerkspfade, Windows-Aliase, doppelte Pfade, Dateipräfix-Kollisionen und Symlinks. |
| Automatische Zuordnung und Rückfragen | erfüllt | Mock-KI erstellt neue Zielordner bei hoher Sicherheit. Niedrige Sicherheit und widersprüchliche Regeln lassen die Quelle im Eingang. Pfade ausserhalb von unterlagen werden abgewiesen. |
| Einlernen | erfüllt | Persistente Regeln, freie KI-Hinweise und bestätigte Beispiele. Identische Inhalte können direkt anhand eines Beispiels zugeordnet werden. Browserprüfung belegt Folgedatei über Regel und KI-Kontext mit Lernbeispiel. Kein Modell-Finetuning. |
| Verlauf mit Eingängen und Zielorten | erfüllt | Dauerhafte Einträge für Ablage, Chat-Verschiebung und Entpacken; Quelle, Ziel, Zeitpunkt und Grund, Seitennavigation sowie sichere Rücknahme unveränderter Dateien. Neustart-Erhalt geprüft. |
| Schlüssel-/Zugriffsschutz | erfüllt in den automatisierten Prüfungen | Alte Zugriffstests bleiben erfolgreich. Neue Mutation-Endpunkte verwenden CSRF und die bestehende Geräte-/Login-Prüfung. Profile, Regeln, Explorer-Öffnung und Update-Verwaltung sind lokal geschützt. Strukturierte KI-Vorschläge redigieren auch JSON-escaped Schlüssel-Echos. |

## Prüfungen

Lokal bestanden: **43 Tests**, `cargo build --release --workspace --locked`, Formatprüfung und Clippy mit `-D warnings`. Zusätzlich ist Clippy für sämtliche Windows-MSVC-Targets erfolgreich. Der vollständige Chromium-Test wurde auch mit dem optimierten Release-Build ausgeführt.

Die vollständigen lokalen Cargo-Ausgaben und das Browserprotokoll liegen unter `docs/pruefungen/0.5-*`. Der native Windows-Lauf mit Build, Tests und Clippy wird im GitHub-Release verlinkt; die veröffentlichte EXE stammt aus diesem Lauf.

Das Betriebssystem der lokalen Entwicklung ist Linux. Windows-Doppelklick, Explorer-Öffnung, Tray, Credential Manager, tatsächliche MAC-Auflösung im LAN, HKCU-Autostart und OCR mit realen Windows-Sprachpaketen bleiben interaktive Windows-Prüfungen. Automatisierte Windows-Tests ersetzen diese nicht. Die ursprünglichen fünf Abnahmekriterien und ihre Grenzen sind zusätzlich in `ABNAHME.md` dokumentiert.

## Grenzen und Schutzmassnahmen

- KI-Zuordnung verwendet die Modell-Einschätzung mit Standardschwelle 0,9. Diese Einschätzung kann falsch sein. Regeln, Rückfragen und der rückverfolgbare Verlauf bleiben deshalb Teil der Bedienung.
- Bei Dateien ohne unterstützte Textextraktion oder über 10 MB verwendet die Zuordnung Metadaten und Regeln. Die Datei selbst wird unverändert abgelegt.
- ZIP-Import: maximal 512 MB komprimiert/entpackt, 10.000 Einträge und drei Verschachtelungsebenen. Problematische Originalarchive bleiben erhalten.
- Vollständige ZIP-Sicherungen sind auf 512 MB und 9.999 Nutzdateien plus Manifest begrenzt. Für grössere Bestände vor dem Update eine Explorer-Kopie des Datenordners anlegen. Die automatische Migrationssicherung von Datenbank und Konfiguration ist davon unabhängig.
- Nur gespeicherte bzw. abgeschlossene Eingaben sind dauerhaft. Nicht abgesendete Texte oder Chats mit abgeschalteter Speicherung vor einem Programmwechsel selbst sichern.

Die automatische Eingangsablage erweitert auf ausdrücklichen Benutzerwunsch die ursprüngliche Abgrenzung des Konzepts. Das Modell erhält weiterhin keine Dateisystem-Werkzeuge. Verschiebeentscheidungen werden durch die Anwendung validiert und ausschliesslich über den zentralen Pfadwächter ausgeführt.
