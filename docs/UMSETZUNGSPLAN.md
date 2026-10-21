# Umsetzungsplan

Die verbindliche Grundlage ist `folder-mind-konzept-v2.md`, ergänzt durch die Entscheidungen im Auftrag: HTML/htmx, MAC ab 0.3, freier Chat aktiv, vier Vorlagen und MIT.

## 0.1 – Fundament

Zehn Workspace-Crates, zentrale Pfadprüfung, SQLite-Migration, Einrichtung mit Ordner-Browser, lokale IPv4/IPv6-Listener, Extraktion von PDF/TXT/MD/HTML und Tantivy-Suche. Prüfung mit Pfadangriffen und vollständigem HTTP-Einrichtungsablauf.

## 0.2 – KI und Wissen

OpenAI-kompatibler Client, Windows Credential Manager, Verbindungstest, Datenschutzbestätigung, Chat mit Bereichen und Quellen, SSE, Wissensentwürfe, Notizen und Freigaben. Tests gegen einen lokalen Mock-Server einschliesslich Authentifizierungsfehlern und ungültigen Antworten.

## 0.3 – Organisation und Zugriff

Vier Vorlagen, Ordnerverwaltung und Uploads, stabile Dateien, Watcher und Kontrollscan, Warteschlange und Pause. IP/CIDR/MAC-Regeln, Argon2-Anmeldung, TLS und lokale Administrationsgrenze. Zugriffstests für entfernte Geräte.

## 0.4 – Komfort und Lieferung

Office-Extraktion, Windows-OCR, semantische Suche, Lernmodus, Chatverwaltung, Sicherung und Wiederherstellung mit Vorschau. Tray und Autostart, deutsche Dokumentation, PowerShell-Build, Windows-CI und Abnahmebericht.

Jeder Versionsstand wird mit `cargo build --release`, `cargo test` und `cargo clippy --workspace --all-targets -- -D warnings` geprüft. Windows-Build und native Laufzeitprüfung werden im Abschlussbericht getrennt ausgewiesen.
