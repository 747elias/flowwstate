# Architektur

## Schichten

- `Flowstate.Core`: Entitäten, Validierung, deterministischer Scheduler, Zeitfenster/DST-Regeln, Fokuszeit und lokale Eingabeerkennung; keine UI-Abhängigkeit.
- `Flowstate.Infrastructure`: EF-Core-SQLite-Datenbank, versionierte Migrationen, transaktionale Speicherung, JSON-Backups und GitHub-Updater.
- `Flowstate.App`: native WinUI-3-Oberfläche, CommunityToolkit-MVVM-Zustand, DI, Dialoge und Windows-/Audiointegration. Seiten werden über native Controls aufgebaut; kein WebView.
- `Flowstate.Tests`: xUnit-Kern- und SQLite-Integrationstests.

## Speicherung

Eine App-Instanz je Windows-Benutzer; pro Benutzer `%LOCALAPPDATA%/Flowstate/flowstate.db`. Tabellen für Aufgaben, Projekte, Blöcke, Sitzungen, Workspaces, Reviews und versionierte Einstellungen. Projekt-/Aufgabenbezüge sind Fremdschlüssel mit SET NULL. Checklisten, Tags, Kontexte und Abhängigkeiten sind EF-Primitive-Collections.

`Store` serialisiert Schreiboperationen mit einem Semaphore. Jede Speicherung erstellt eine eingefrorene Kopie und ersetzt den kleinen lokalen Datensatz in einer Transaktion; bei Fehler erfolgt Rollback. Die UI lädt nach einem Speicherfehler den letzten Datenbankstand. Das Verfahren ist für überschaubare persönliche Datenbestände gedacht. Inkrementelle Entity-Updates und Virtualisierung sind vor großen Beständen erforderlich.

Migrationen sind eingecheckt. Vor ausstehenden Migrationen wird über SQLite Backup API ein konsistentes Datenbank-Backup erstellt. JSON-Import prüft Schema, IDs, Verweise und Werte, sichert den aktuellen Stand und speichert transaktional. Importe starten weder Timer noch Workspace-Aktionen.

## AutoPlan

1. Geschützte Blöcke von Verfügbarkeitsfenstern abziehen.
2. Inaktive/archivierte Aufgaben ausschließen, bereits geleistete und geplante Zeit abziehen.
3. Harte Bedingungen prüfen: Startzeit, Abhängigkeiten, Kontext, Energie, Mindestdauer und Teilbarkeit.
4. Frühestmögliche Aufgaben nach Priorität, Deadline, begonnenem Status und Projektkontinuität bewerten.
5. Blöcke und Pausen einfügen; kleine unbrauchbare Restfragmente vermeiden.
6. Nicht planbare Aufgaben mit Gründen zurückgeben.

Kein global optimales Ergebnis garantiert; deterministische Heuristik. Deadlines sind weiche Ziele und verspätete Blöcke werden markiert. UTC-Offsets sichern Vergleiche. Mehrdeutige/nicht existierende lokale Zeiten bei DST werden mit verständlichem Fehler abgelehnt. Übernahme vergleicht den aktuellen Datenstand mit der Vorschau, um veraltete Pläne zu verhindern. Eine Undo-Kopie wird mitgespeichert. Manuelle Kalenderänderungen verwerfen dieses Undo, damit sie nicht versehentlich überschrieben werden.

## Timer

UTC-Startanker plus akkumulierte aktive Zeit; keine gezählten UI-Ticks. Ein DispatcherTimer aktualisiert nur die Anzeige. Lock/Standby zählt als verstrichene Zeit. Pausen zählen nicht. Erfassung ist auf das gesetzte Sitzungsziel begrenzt (außer Stoppuhr). Zustand bleibt über Navigation/Neustart erhalten; Session-ID verhindert doppelte Speicherung. Rückwärts gestellte Uhr erzeugt keine negative Dauer, große Vorwärtssprünge können eine Sitzung beenden. Eine monotone Laufzeitquelle für manuelle Systemuhränderungen steht noch aus.

## Windows

Unpackaged, self-contained WinUI 3. Systemtray und Benachrichtigungen über dokumentierte Shell_NotifyIcon-API. Global Hotkey über RegisterHotKey, Fensterereignisse über SetWindowSubclass. Fremde Skripte/Argumentstrings werden nicht als Workspace-Aktionen ausgeführt. „Nicht stören“ wird über einen unterstützten `ms-settings`-Link angeboten, nicht durch Registry-Hacks gesetzt.

## Audio

NAudio-Ausgabe, White/Brown/Pink Noise in Echtzeit, begrenzte Amplitude und sanfte Pegeländerung. Eigene lokale Audio-Dateien werden resampelt und geloopt. Bei nicht nahtlos geschnittenen Benutzerdateien kann der Loop hörbar bleiben.
