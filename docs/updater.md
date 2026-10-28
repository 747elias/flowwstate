# Update-System

Quelle: https://api.github.com/repos/747elias/flowwstate/releases/latest. Keine Tokens in der App. Opt-in/out in Settings; maximal täglich automatisch, manuell jederzeit. GitHub erhält lediglich die normale HTTP-Anfrage, keine Aufgabeninhalte.

Die App ignoriert Drafts und Prereleases. SemVer-Vergleich über NuGet.Versioning; Tags `v1.2.3` oder `flowstate-v1.2.3`. Installerversion folgt der zentralen Version in Directory.Build.props. Ein Beta-Release ist über die Releases-Seite manuell installierbar und wird nicht als Stable-Update verteilt.

Installer-Asset: `Flowstate-Setup-<Version>-win-x64.exe`. Prüfsummen: `SHA256SUMS.txt`. Nur HTTPS und exakte GitHub-Repository-/Tag-Pfade werden akzeptiert. GitHub-CDN-Weiterleitungen erfolgen über den HTTP-Client; initiale Download-URLs werden streng geprüft. Dateigröße ist auf 600 MB begrenzt. Downloads landen unter einem zufälligen lokalen Namen; SHA-256 wird vor Ausführung verglichen. Downloads werden bei Fehler gelöscht.

Nach ausdrücklicher Installationsbestätigung: Daten speichern, Download prüfen, Installer mit vorhandenem Installationspfad und Benutzer-/Systemmodus starten, App schließen. Die Installer-UI bleibt sichtbar. Es gibt keine stille Elevation oder Sicherheitsumgehung.

Kein Code-Signing-Zertifikat vorhanden. Die SHA-256-Datei schützt gegen Beschädigung, nicht gegen eine gleichzeitige Manipulation von Installer und Manifest in einem kompromittierten Repository. Authenticode-Prüfung und vollständige Upgrade-Abnahme sind offen. Ein Offlinefehler blockiert die Kernfunktionen nicht.

## Release-Prozess

Normale Pushes bauen/testen und liefern CI-Artefakte. Ein Commit auf main, dessen Nachricht mit `release:` beginnt, veröffentlicht nach allen erfolgreichen Jobs eine als Prerelease markierte Version mit Tag `flowstate-v<Version>`. Vor neuer Veröffentlichung Version zentral erhöhen und Release Notes aktualisieren. Stable-Veröffentlichung erst nach Schließen der Abnahmepunkte; der Workflow publiziert bewusst ausschließlich Prereleases.
