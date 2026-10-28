# Installation und Deinstallation

Windows 11 x64 (Build 22000+). Eine Installer-EXE enthält App, .NET und Windows-App-SDK-Laufzeitdateien. Installer aktuell unsigniert.

Inno Setup verwendet `PrivilegesRequired=lowest` und `PrivilegesRequiredOverridesAllowed=dialog commandline`:

- Standard: **Nur für mich**, `%LOCALAPPDATA%/Programs/Flowstate`, keine Elevation erforderlich.
- Optional: **Für alle Benutzer**, `%ProgramFiles%/Flowstate`, UAC/Administrator erforderlich.
- Startmenü-Verknüpfung und optional Desktop-Verknüpfung.
- Installationsmodus/-pfad werden bei Folgeinstallationen übernommen.

Das Schließen des Fensters minimiert standardmäßig in den Infobereich. Rechtsklick → Beenden beendet die App. Dieses Verhalten ist in den Einstellungen änderbar.

Benutzerdaten liegen für jeden Benutzer getrennt in `%LOCALAPPDATA%/Flowstate`. Updates und Deinstallation entfernen diese Daten nicht. Zur endgültigen Entfernung eigener Daten nach dem Beenden diesen Ordner selbst löschen; er enthält auch Backups und Exporte. Daten anderer Benutzer werden nie automatisch gelöscht.

Automatisierte Installation mit `/CURRENTUSER /VERYSILENT /SUPPRESSMSGBOXES /NORESTART`; systemweit `/ALLUSERS`. Sicherheitsabfragen nicht umgehen. Der Windows-CI-Lauf ersetzt keinen Test als echter Standardbenutzer auf Windows 11.
