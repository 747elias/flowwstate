#define AppVersion "0.1.0-beta.1"
[Setup]
AppId=Flowstate
AppName=Flowstate
AppVersion={#AppVersion}
AppVerName=Flowstate {#AppVersion}
AppPublisher=Flowstate
AppPublisherURL=https://github.com/747elias/flowwstate
DefaultDirName={autopf}\Flowstate
DefaultGroupName=Flowstate
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog commandline
UsePreviousPrivileges=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.22000
OutputDir=..\artifacts\installer
OutputBaseFilename=Flowstate-Setup-{#AppVersion}-win-x64
SetupIconFile=..\src\Flowstate.App\Assets\Flowstate.ico
UninstallDisplayIcon={app}\Flowstate.App.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
DisableProgramGroupPage=yes
[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
[Tasks]
Name: desktopicon; Description: "Desktop-Verknüpfung erstellen"; Flags: unchecked
[Files]
Source: "..\artifacts\app\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
[Icons]
Name: "{autoprograms}\Flowstate"; Filename: "{app}\Flowstate.App.exe"
Name: "{autodesktop}\Flowstate"; Filename: "{app}\Flowstate.App.exe"; Tasks: desktopicon
[Run]
Filename: "{app}\Flowstate.App.exe"; Description: "Flowstate starten"; Flags: nowait postinstall skipifsilent runasoriginaluser
[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "Flowstate"; Flags: uninsdeletevalue dontcreatekey
