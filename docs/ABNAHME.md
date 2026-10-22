# Abnahme Folder-Mind 0.4

Die Anwendung ist bis Version 0.4 implementiert. Eine Windows-x86_64-MSVC-EXE wird mitgeliefert. Die abschliessende interaktive Windows-Abnahme ist noch offen: Die lokale Entwicklungsumgebung ist Linux; automatisierte native Windows-Prüfungen laufen zusätzlich auf GitHub Actions.

## Abnahmekriterien aus Abschnitt 23

| Kriterium | Status | Nachweis / Grenze |
| --- | --- | --- |
| Nach dem ersten Start ist die komplette Einrichtung ohne Kommandozeile möglich. | **nicht erfüllt** für die abschliessende Windows-Abnahme | Der vollständige Assistent mit Ordnerwahl, Vorlage, KI-Verbindung und Abschluss ist per HTTP und im Browser bestanden. Windows-Doppelklick, nativer Browserstart und Tray wurden hier nicht ausgeführt. |
| Die Web UI ist von einem anderen Gerät im Netz nicht erreichbar. | **erfüllt** im Standardbetrieb | HTTP bindet nur 127.0.0.1 und ::1. Ein Sockettest bestätigt: Loopback erreichbar, Geräte-IP:8484 verweigert. Netzwerkfreigabe erfordert validierte Whitelist, Passwort und separaten TLS-Listener. |
| Eine Datei im richtigen Kapitelordner wird erkannt, genau einmal verarbeitet und im Ordnerbaum angezeigt. | **erfüllt** | HTTP-Test mit zwei Scans ohne zusätzliche Dokumentversion; Watcher-/Stabilitätstest; Browserprüfung mit sieben Formaten, Drag-and-drop und Pause/Fortsetzen. |
| Versuche, ausserhalb des Datenordners zu schreiben, werden blockiert und protokolliert. | **erfüllt** in den ausgeführten Sicherheitsprüfungen | Unit-Tests für Traversierung, absolute/Windows-/UNC-Pfade, Gerätenamen, Symlinks, Hardlinks und Verschieben. HTTP-Pfadangriffe auf Ordner und Notizen liefern Fehler und erscheinen im Protokoll. Der zusätzliche Windows-Junction-Test wurde auch nativ auf GitHub Actions erfolgreich ausgeführt. |
| Die Volltextsuche findet Begriffe aus den Unterlagen. | **erfüllt** | Tantivy-Unit-Test, HTTP-Test und Browserprüfung mit PDF/TXT/MD/HTML; unbeantwortbare Frage liefert die vorgesehene Meldung ohne KI-Aufruf. |

„Erfüllt“ bezeichnet die implementierten und hier direkt nachgewiesenen Funktionen. Es ersetzt keine native Windows-Freigabe des Gesamtprodukts.

## Versionsprüfungen

| Stand | Release-Build | Tests unter Linux | Clippy mit `-D warnings` | Windows |
| --- | --- | --- | --- | --- |
| 0.1 | bestanden | 8 bestanden | bestanden | späterer Cross-Check der Folgestände |
| 0.2 | bestanden | 12 bestanden | bestanden | MSVC-Cross-Check bestanden |
| 0.3 | bestanden | 17 bestanden | bestanden | MSVC-Cross-Check und Release-Build bestanden |
| 0.4 | bestanden | 29 bestanden | bestanden | Clippy für alle Targets und MSVC-Release-Build bestanden |

Die jeweiligen vollständigen Cargo-Ausgaben liegen unter `docs/pruefungen/`. Der Cross-Linker meldet fehlende Microsoft-SDK-PDB-Dateien (LNK4099). Das betrifft Debug-Symbole der statischen C-Laufzeit; der Release-Link ist erfolgreich. Windows-Clippy ist ohne Warnungen.

## Browser- und HTTP-Prüfung

Die lokale Chromium-Prüfung umfasst:

- Sechs Einrichtungsschritte mit serverseitiger Ordnerauswahl und lokalem OpenAI-Mock.
- Tatsächlichen htmx-Statusabruf und lokale, eingebettete Ressourcen.
- Upload und Verarbeitung von PDF, TXT, MD, HTML, DOCX, PPTX und XLSX; Drag-and-drop eines weiteren Dokuments.
- Pause/Fortsetzen und Verarbeitung einer während der Pause angelegten Datei.
- Volltextsuche, Embeddings und semantische Suche gegen den Mock.
- SSE mit gültigen Quellen, Öffnen einer Quelle und gespeicherte Chat-Quellen.
- Gesprächskontext ohne Speicherung; Abschalten der Speicherung erzeugt keine weiteren gespeicherten Nachrichten.
- Entwurf, Freigabe und unveränderte eigene Notizen.
- ZIP-Erstellung, Probelauf, bestätigte Wiederherstellung und Prüfung des wiederhergestellten Inhalts.
- Datenordnerwechsel mit Mitnahme in ein leeres Ziel.
- Abgewiesene Pfadangriffe samt Protokollnachweis.
- Abgewiesene Netzwerkaktivierung ohne Whitelist; anschliessend tatsächlicher HTTPS-Aufruf mit ausdrücklich vertrautem Testzertifikat und geprüftem SHA-256-Fingerabdruck.
- Alle neun eingerichteten Ansichten bei 390 Pixel Breite ohne horizontalen Seitenüberlauf; Hell/Dunkel und keine JavaScript-Ausnahmen.

Der KI-Mock prüft zusätzlich falsche Schlüssel, fehlende Modelle, Zeitüberschreitungen, fehlerhaftes SSE und Schlüssel-Echos über mehrere SSE-Blöcke. Quellenlose Modellantworten werden abgelehnt. Ein Katalogtest belegt, dass frühere Quellen nach Verarbeitung einer neuen Version und nach Löschung weiterhin als archivierter Extrakt vorhanden sind.

## Windows-Binärprüfung

- PE-Maschine: `IMAGE_FILE_MACHINE_AMD64`.
- Subsystem: `IMAGE_SUBSYSTEM_WINDOWS_GUI`; kein Konsolen-Subsystem im Release.
- Statische C-Laufzeit. Die Importliste enthält ausschliesslich Windows-Systembibliotheken, insbesondere kernel32, user32, iphlpapi, advapi32, WinRT, ole32/oleaut32, bcrypt, gdi32, comctl32 und shell32.
- Kein OpenSSL, keine separate SQLite-/OCR-/C-Laufzeit-DLL und keine Python-/Node-Laufzeit in der Lieferung.

## Native Prüfungen, die noch fehlen

Windows-Doppelklick/Browserstart, Tray-Nachrichtenloop, Windows Credential Manager, tatsächliche MAC-Auflösung in einem lokalen Netz, HKCU-Autostart und Windows-OCR wurden in der lokalen Linux-Umgebung nicht nativ ausgeführt. Der enthaltene `windows-latest`-Workflow führt Builds und automatisierte Tests unter Windows aus; die [aktuellen Ergebnisse](https://github.com/747elias/folder-mind/actions/workflows/windows.yml) stehen auf GitHub. Interaktive Tray-/OCR-/Netztests bleiben zusätzlich manuell durchzuführen.

## Veröffentlichung auf GitHub

Der vollständige Quellcode liegt unter [747elias/folder-mind](https://github.com/747elias/folder-mind), einschliesslich der Tags 0.1 bis 0.4. Der erste native Windows-Lauf hat die Junction-Sicherheitsprüfung erfolgreich ausgeführt und anschliessend eine durch Git-Zeilenendenkonvertierung beschädigte PDF-Testdatei erkannt. Version 0.4.1 schützt die Binärdateien ausdrücklich durch `.gitattributes`. Die lokalen Nachweise oben dokumentieren weiterhin die ursprüngliche Linux-/Cross-Build-Abnahme; ergänzende native Ergebnisse werden im GitHub-Release verlinkt.

Die begründeten funktionalen Abweichungen, insbesondere geprüfte statt vorzeitige SSE-Anzeige, explizite KI-Entwürfe, Kontextgrenzen, ZIP-Grenze und fehlender SCM-Dienst, stehen in README → „Abweichungen vom Konzept“.
