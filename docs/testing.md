# Testnachweise

## Automatisiert

Die xUnit-Suite prüft Priorität/Deadline, geschützte Termine, volle Kalender, Teilbarkeit/Mindestfragmente, Startzeit, Kontext, Energie, Abhängigkeiten/Zyklen, Pausen, vorhandene Planung, Sommerzeit/Sonderzeiten, Serienaufgaben, Eingabeerkennung, Timer, SemVer und Downloadherkunft. SQLite-Tests prüfen echte Dateien und Migrationen, CRUD/Beziehungen, Fokus/Kalender, Export/Backup/Wiederherstellung sowie Ablehnung ungültiger Imports ohne Datenverlust.

Lokaler Lauf: `dotnet test tests/Flowstate.Tests -c Release`. Ergebnisse werden zusätzlich als TRX-Artefakt in GitHub Actions gespeichert.

Windows-CI: native WinUI-Veröffentlichung, Inno-Installer, Installation per `/CURRENTUSER`, Prozessstart, Datenbank-Erstellung, Navigation durch alle 12 Seiten via UI Automation, Dashboard-Inhaltsprüfung und Screenshot, Deinstallation und Erhalt der Benutzerdaten. Diese Prüfungen sind erst nach grünen zugehörigen Schritten als bestanden zu werten. CI-Runner ist Windows Server, keine manuelle Windows-11-Abnahme.

## Noch manuell auf Windows 11 erforderlich

- Installation als echter Standardbenutzer ohne UAC; Installation systemweit mit UAC.
- Installer-/Updatepfade als Benutzer und Administrator, Daten-/Einstellungsübernahme, Abbruch/Netzausfall.
- Navigation, Erstellen/Bearbeiten/Löschen, Kalender-Drops und Tastaturbedienung.
- Screenreader, 125/150/200 % DPI, Hochkontrast, Touch, kleine Fenster, mehrere Monitore.
- Tray, Hotkey-Konflikt, Fokusende-Benachrichtigung, Autostart.
- Audioausgabe/Wechsel des Audiogeräts, Loop/Fades, langlaufende Sessions, Standby/Ruhezustand und manuelle Uhränderungen.
- Malware-/Signaturprüfungen sind nicht durch SHA-256 oder Build abgedeckt.
- Startzeit, RAM/CPU und große Datenbestände: noch keine gemessenen Benchmarks.

Es werden keine manuellen Tests oder Performancezahlen als durchgeführt ausgewiesen, wenn dafür kein Nachweis vorliegt.

## Gemessene AutoPlan-Laufzeit

28.10.2026, Linux-Entwicklungscontainer, Debian 13, .NET 10.0.14, Release-Build. Ein Aufwärmlauf, anschließend 10 Wiederholungen, 14 verfügbare Tage à 8 Stunden, 5-Minuten-Pausen. Keine Aussage zur Windows-UI-Leistung.

| Aufgaben | Median (ms) | Min–Max (ms) | Erzeugte Blöcke | Restliche Aufgaben |
|---|---:|---:|---:|---:|
| 100 | 10,83 | 9,48–20,10 | 199 | 5 |
| 1000 | 68,13 | 42,68–99,23 | 275 | 867 |

Reproduzieren: `dotnet run --project tools/Flowstate.Benchmarks -c Release`. Die große Restmenge im zweiten Fall entsteht durch die begrenzte Zeitkapazität. Messwerte sind maschinenabhängig.
