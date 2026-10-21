# Konzept in 14 Punkten

- Eine lokale Rust-Anwendung mit eingebetteter deutscher Weboberfläche und automatischem Browserstart.
- Ein Einrichtungsassistent mit serverseitiger Ordnerauswahl; Ablage und Suche funktionieren auch ohne KI.
- Ein zentraler Pfadwächter für Änderungen im Datenordner, einschliesslich Schutz vor Symlinks und Junctions.
- Vier Ordnervorlagen und eine anpassbare Struktur mit ein bis vier benannten Ebenen.
- Getrennte Originale, Extrakte, KI-Entwürfe, freigegebene Wissensseiten und eigene Notizen.
- Ein SQLite-Katalog mit Dateiversionen, Prüfsummen, Quellen, Aufträgen und Chatverläufen.
- Dokumentverarbeitung mit Fundstellen und Volltextsuche; Office-Formate, OCR und semantische Suche ab 0.4.
- Eine OpenAI-kompatible Anbindung mit sicherer Schlüsselablage, Datenschutzbestätigung und Verbindungstest.
- Ein Chat mit Bereichsauswahl, Lernmodi, SSE und überprüfbaren Quellenverweisen.
- Entwürfe und Freigaben ohne automatische Änderungen an Originalen oder eigenen Notizen.
- Lokaler Zugriff als Standard; Netzwerkfreigabe nur mit Whitelist, Passwort und HTTPS.
- CSRF-Schutz und ausschliesslich lokale Verwaltung von Einstellungen und Zugriffsliste.
- Hintergrundverarbeitung mit Dateistabilität, Warteschlange, Pause und Wiederholung.
- Sicherungen, Windows-Tray, Autostart, Protokolle, eine einzelne EXE, MIT und dokumentierte Prüfungen.

Der Ablauf entlang 0.1 bis 0.4 steht in `UMSETZUNGSPLAN.md`, die geprüften Versionsstände in `VERSIONEN.md`, die tatsächliche Abnahme in `ABNAHME.md`.
