# Entwicklungsstatus und offene Abnahme

Version: 0.1.0-beta.1. **Das Projekt erfüllt noch nicht alle Anforderungen der PDF und ist kein Stable Release.**

| Bereich | Implementiert | Noch offen / Abnahme |
|---|---|---|
| Foundation | Native WinUI 3, .NET 10, DI, MVVM-Zustand, Theme, Navigation, lokale Fehlerlogs | UI-Virtualisierung, Performance-Benchmarks, umfassende Barrierefreiheitsprüfung |
| Aufgaben | CRUD, Status, Priorität, Projekte, Deadline/Startdatum, Tags/Kontexte, Energie, Notizen, Checklisten, Abhängigkeiten, Wiederholungen, Filter/Suche, Archiv, Sammelaktionen | Hierarchische Unteraufgaben als eigene Entitäten, Anhänge, komplexe Wiederholungsregeln, ausgereifte Serien-Ausnahmen |
| Projekte | CRUD, Beschreibung/Zieldatum/Archiv, Fortschritt, gefilterte Aufgaben, Zeitübersicht | Eigene Farben/Icons im Editor, Projektzeitziele |
| Kalender | Tages-/Wochenraster, Monat, Agenda, Konflikte, manuelles Timeboxing, Drag & Drop, Dauer im Editor | Resize direkt am Block, Fokus-Sessions als Kalenderlayer, externe Integrationen/ICS |
| AutoPlan | 5 Modi, harte Bedingungen, Scoring, Vorschau, Übernahme, Gründe, Pausen, Undo | Erweiterte Präferenzgewichte und automatische Hinweise auf veraltete Pläne |
| Fokus | Pomodoro/Deep Work/Custom/Stopwatch, Pause/Resume/Stop/Reset/Verlängern, persistenter Timer, Aufgabenzuordnung, kompakte Ansicht | Automatischer nächster Zyklus, monotone Uhr bei Systemzeitänderung, lange-Pause-Zähler über Neustart, vollständige Tray-Menü-Parität |
| Sound | Drei Noise-Typen, Mixer, Standardmix speichern, Dateiimport, Loop, Fades | Mehrere parallele Benutzerdateien, benannte Preset-Sammlung, lizenzierte Natur-/Café-/Musik-Sammlung, Crossfade für Dateiloops |
| Workspaces | CRUD, bestätigte Programm-/Dokument-/Ordner-/URL-Starts, Fokusdauer | Sound-/Projekt-Preset-Zuordnung und Fensteranordnung |
| Quick Capture | Global Hotkey, lokale deutschsprachige Erkennung, Editor zur Kontrolle | Eigenständiges kleines Overlay, umfangreichere Sprachsyntax |
| Analytics/Review | Tages-/7-/30-Tageswerte, Balkendiagramme, Projektzeiten, Review-Notizen | Freier Datumsbereich, alle geforderten Kennzahlen, Benachrichtigungsautomatik |
| Einstellungen | Themes, Akzent, Startseite, Tray, Hotkey, Autostart, Fokus/Arbeitszeiten, Dashboard-Sichtbarkeit/Reihenfolge, Updates, Daten | Vollständige DE/EN-Lokalisierung, Fenstergeometrie, alle Detailpräferenzen aus PDF |
| Daten | SQLite/EF, Fremdschlüssel, Migration, Sicherung vor Migration, JSON-Export/Import und Backup | Regelmäßiger geplanter Backup-Job, In-App-Restore von rohen DB-Migrationsbackups |
| Installer/Updates | Inno per-user/per-machine, Self-contained, Deinstallation mit Datenerhalt, GitHub-Checks, SHA-256, Stable-Kanal | Code-Signing-Zertifikat, Standardbenutzer/UAC/Mehrbenutzer/Upgrade-Ende-zu-Ende auf sauberem Windows 11 |

Windows-API-Verhalten, DPI, Touch, Screenreader, Langzeitbetrieb, Audioausgabe und Standby werden durch einen erfolgreichen Build nicht automatisch als geprüft gewertet. Konkrete Nachweise stehen in `docs/testing.md` und GitHub Actions.
