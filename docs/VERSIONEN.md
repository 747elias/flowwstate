# Versionsstände

## 0.1 – Fundament

Zehn Crates, Einrichtung, Ordner-Browser, Pfadwächter, SQLite, vier Startformate und Tantivy. Release, acht Tests und Clippy bestanden. Lokal als `v0.1.0` gesichert.

## 0.2 – KI und Wissen

OpenAI-Client, Credential-Manager-Anbindung, Mock-API, Verbindungstest, Quellenprüfung, SSE, Chats, Entwürfe, Notizen und Freigaben. Release, zwölf Tests und Clippy bestanden; Windows-Cross-Check und Browserprüfung bestanden. Lokal als `v0.2.0` gesichert.

## 0.3 – Organisation und Zugriff

Ordneraktionen, Uploads, Watcher, Stabilitätsprüfung, Warteschlange, Pause, IP/CIDR/MAC, Argon2, Sitzungen, TLS und lokale Administration. Release, 17 Tests und Clippy bestanden. Windows-Cross-Check und Windows-Release-Build bestanden. Lokal als `v0.3.0` gesichert.

## 0.4 – Komfort und Lieferung

Office, Windows-OCR, semantische Suche, Lernkontext, Quellenarchiv, geprüfte Sicherungen, Wiederherstellung, Datenordnerwechsel, Zeitfenster, Tray und Autostart. Release, 29 Tests, Clippy und Windows-Build geprüft. Windows-Clippy prüft auch alle Test-Targets. Deutsche Anleitung, PowerShell-Build und GitHub Actions liegen bei. Grenzen der nativen Windows-Abnahme siehe `ABNAHME.md`.

## 0.4.1 – Windows-Checkout und GitHub-Veröffentlichung

Der vollständige Quellcode mit den bisherigen Versions-Tags liegt auf GitHub. `.gitattributes` verhindert die Zeilenendenkonvertierung binärer Dateien, die beim ersten Windows-Prüflauf die PDF-Beispieldatei beschädigt hat. Die Anwendung selbst bleibt funktional auf Stand 0.4. Der Release enthält die Windows-EXE und ihre SHA-256-Prüfsumme; der zugehörige native Windows-Prüflauf ist in den Release-Notizen verlinkt.
