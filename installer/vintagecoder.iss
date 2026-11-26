; Inno-Setup-Skript für den VintageCoder (Windows, x64).
; Der Installer packt nur noch das Staging ein, das "python tools\release.py stage --os windows --out <Ordner> --build" erzeugt
; (Programme, data\, Beispiele, Lizenzen, installer.inc). Was drin ist, bestimmt FEATURES.md.
; Bauen: installer\build-installer.ps1 (ruft das Skript auf und dann ISCC).
; Direkt: ISCC.exe /DAppVersion=0.3.0 /DStage=<Staging-Ordner> [/DWebView2Setup=<MicrosoftEdgeWebview2Setup.exe>] vintagecoder.iss
; Die Oberflaeche (vintagecoder.exe) ist eine Tauri-Anwendung und braucht die Microsoft-WebView2-Laufzeit. Mit /DWebView2Setup=<Pfad zu MicrosoftEdgeWebview2Setup.exe>
; (Evergreen-Bootstrapper, laedt build-installer.ps1 und prueft die Signatur; nicht im Repository) wird sie bei Bedarf leise nachinstalliert (Internet noetig);
; ohne diese Angabe prueft der Installer nur und weist auf die Installation hin.
; installer.inc im Staging setzt HasIde (mit Entwicklungsumgebung) und HasExamples (mit Beispielen).

#ifndef AppVersion
  #define AppVersion "0.3.0"
#endif
#ifndef Stage
  #define Stage "..\build\stage-windows"
#endif
#include AddBackslash(Stage) + "installer.inc"
#define AppName "VintageCoder"
#define Exe "vintagecoder.exe"
#ifdef HasIde
  #define OutSuffix ""
  #define IconFile "VintageCoder.ico"
#else
  #define OutSuffix "-CLI"
  #define IconFile "vcc.exe"
#endif

[Setup]
AppId={{6E1F2B7A-4C3D-4A55-9D0E-7B2C51A8F3D4}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher=Christian Drüeke
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
UninstallDisplayIcon={app}\{#IconFile}
SetupIconFile=windows\VintageCoder.ico
LicenseFile={#Stage}\LICENSE
OutputDir=output
OutputBaseFilename=VintageCoder{#OutSuffix}-{#AppVersion}-Setup
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
WizardStyle=modern
DisableProgramGroupPage=yes
ChangesEnvironment=yes
SetupLogging=yes

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"; InfoBeforeFile: "lizenzhinweis.txt"
Name: "english"; MessagesFile: "compiler:Default.isl"; InfoBeforeFile: "lizenzhinweis-en.txt"

[CustomMessages]
german.TaskDesktop=Symbol auf dem &Desktop anlegen
english.TaskDesktop=Create a &desktop shortcut
german.TaskPath=Kommandozeilenprogramm &vcc zum PATH hinzufügen
english.TaskPath=Add the command-line program &vcc to PATH
german.TaskExamples=&Beispielprojekte nach "Dokumente\VintageCoder\Beispiele" kopieren
english.TaskExamples=Copy the &example projects to "Documents\VintageCoder\Examples"
german.FileTypeProject=VintageCoder-Projekt
english.FileTypeProject=VintageCoder project
german.FileTypeMedia=VintageCoder-Mediendatei
english.FileTypeMedia=VintageCoder media file
german.WebView2Need=Die Oberfläche von VintageCoder braucht die Microsoft-WebView2-Laufzeit.
english.WebView2Need=The VintageCoder interface needs the Microsoft WebView2 runtime.
german.WebView2Installing=WebView2-Laufzeit wird installiert (Internetverbindung nötig) ...
english.WebView2Installing=Installing the WebView2 runtime (internet connection required) ...
german.WebView2Failed=Die automatische Installation ist fehlgeschlagen (ist eine Internetverbindung vorhanden?).%nVintageCoder wurde installiert, startet aber erst, wenn die Laufzeit da ist. Bitte "Evergreen Bootstrapper" laden und ausführen:
english.WebView2Failed=The automatic installation failed (is an internet connection available?).%nVintageCoder was installed but will only start once the runtime is present. Please download and run the "Evergreen Bootstrapper":
german.WebView2Missing=Sie wurde auf diesem Rechner nicht gefunden. Bitte "Evergreen Bootstrapper" laden und ausführen, bevor VintageCoder gestartet wird:
english.WebView2Missing=It was not found on this computer. Please download and run the "Evergreen Bootstrapper" before starting VintageCoder:

[Tasks]
#ifdef HasIde
Name: "desktopicon"; Description: "{cm:TaskDesktop}"; Flags: unchecked
#endif
Name: "addtopath"; Description: "{cm:TaskPath}"; Flags: unchecked
#ifdef HasExamples
Name: "examples"; Description: "{cm:TaskExamples}"
#endif

[Files]
; Programme (Die App sucht data\ ab dem Ort der .exe nach oben: data\ liegt daher direkt daneben.)
#ifdef HasIde
Source: "{#Stage}\vintagecoder.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\VintageCoder.ico"; DestDir: "{app}"; Flags: ignoreversion
#endif
Source: "{#Stage}\vcc.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\vc-lsp.exe"; DestDir: "{app}"; Flags: ignoreversion
; Hardware-Datenbank, Vorlagen, Hilfe (nach Auswahl, mit release.json)
Source: "{#Stage}\data\*"; DestDir: "{app}\data"; Flags: ignoreversion recursesubdirs createallsubdirs
; Lizenz, Hinweise auf fremde Inhalte, Dokumentation, Erweiterungen
Source: "{#Stage}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\LICENSING.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\LICENSES\*"; DestDir: "{app}\LICENSES"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "{#Stage}\THIRD-PARTY-NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\docs\*"; DestDir: "{app}\docs"; Excludes: ".DS_Store"; Flags: ignoreversion recursesubdirs createallsubdirs skipifsourcedoesntexist
Source: "{#Stage}\editors\*"; DestDir: "{app}\editors"; Flags: ignoreversion recursesubdirs createallsubdirs skipifsourcedoesntexist
#if defined(HasIde) && defined(WebView2Setup)
Source: "{#WebView2Setup}"; DestDir: "{tmp}"; DestName: "MicrosoftEdgeWebview2Setup.exe"; Flags: deleteafterinstall; Check: NeedsWebView2
#endif
#ifdef HasExamples
; Beispiele in den Dokumenten des Benutzers (schreibbar, das Bauen legt dort Dateien an); ROM-Auszuege liefert das Staging nie mit
Source: "{#Stage}\examples\*"; DestDir: "{userdocs}\{#AppName}\Beispiele"; Flags: ignoreversion recursesubdirs createallsubdirs onlyifdoesntexist; Tasks: examples
#endif

#ifdef HasIde
[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#Exe}"; WorkingDir: "{userdocs}\{#AppName}"; IconFilename: "{app}\VintageCoder.ico"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#Exe}"; WorkingDir: "{userdocs}\{#AppName}"; IconFilename: "{app}\VintageCoder.ico"; Tasks: desktopicon
#endif

[Registry]
#ifdef HasIde
; Dateiendung .vcprj (Projektdatei) mit VintageCoder öffnen
Root: HKA; Subkey: "Software\Classes\.vcprj"; ValueType: string; ValueData: "VintageCoder.Project"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\VintageCoder.Project"; ValueType: string; ValueData: "{cm:FileTypeProject}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\VintageCoder.Project\DefaultIcon"; ValueType: string; ValueData: "{app}\VintageCoder.ico"
Root: HKA; Subkey: "Software\Classes\VintageCoder.Project\shell\open\command"; ValueType: string; ValueData: """{app}\{#Exe}"" ""%1"""
; Mediendateien (wie in der Tauri-Konfiguration fileAssociations): oeffnen im passenden Editor
Root: HKA; Subkey: "Software\Classes\.vcsprite"; ValueType: string; ValueData: "VintageCoder.Media"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\.vcpmg"; ValueType: string; ValueData: "VintageCoder.Media"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\.vcchar"; ValueType: string; ValueData: "VintageCoder.Media"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\.vctiles"; ValueType: string; ValueData: "VintageCoder.Media"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\.vcsong"; ValueType: string; ValueData: "VintageCoder.Media"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\.vcscreen"; ValueType: string; ValueData: "VintageCoder.Media"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\.vcpal"; ValueType: string; ValueData: "VintageCoder.Media"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\VintageCoder.Media"; ValueType: string; ValueData: "{cm:FileTypeMedia}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\VintageCoder.Media\DefaultIcon"; ValueType: string; ValueData: "{app}\VintageCoder.ico"
Root: HKA; Subkey: "Software\Classes\VintageCoder.Media\shell\open\command"; ValueType: string; ValueData: """{app}\{#Exe}"" ""%1"""
#endif
; PATH (nur wenn gewählt und noch nicht enthalten)
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: (not IsAdminInstallMode) and NeedsPath(ExpandConstant('{app}'))
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: IsAdminInstallMode and NeedsPath(ExpandConstant('{app}'))

#ifdef HasIde
[Run]
Filename: "{app}\{#Exe}"; Description: "{cm:LaunchProgram,{#AppName}}"; WorkingDir: "{userdocs}\{#AppName}"; Flags: nowait postinstall skipifsilent
#endif

[Code]
#ifdef HasIde
{ WebView2-Laufzeit (Evergreen) vorhanden? Laut Microsoft steht die Version "pv" unter dieser Kennung, systemweit (WOW6432Node) oder je Benutzer. }
function WebView2Installed: Boolean;
var
  V: string;
begin
  Result := False;
  if RegQueryStringValue(HKLM, 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', V) and (V <> '') and (V <> '0.0.0.0') then Result := True
  else if RegQueryStringValue(HKCU, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', V) and (V <> '') and (V <> '0.0.0.0') then Result := True;
end;

function NeedsWebView2: Boolean;
begin
  Result := not WebView2Installed;
end;

const
  WebView2Seite = 'https://developer.microsoft.com/microsoft-edge/webview2/';

procedure CurStepChanged(CurStep: TSetupStep);
var
  Setup: string;
  ResultCode: Integer;
begin
  if CurStep <> ssPostInstall then Exit;
  if WebView2Installed then Exit;
#ifdef WebView2Setup
  Setup := ExpandConstant('{tmp}\MicrosoftEdgeWebview2Setup.exe');
  WizardForm.StatusLabel.Caption := CustomMessage('WebView2Installing');
  if FileExists(Setup) and Exec(Setup, '/silent /install', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) and (ResultCode = 0) and WebView2Installed then Exit;
  SuppressibleMsgBox(CustomMessage('WebView2Need') + #13#10#13#10 + CustomMessage('WebView2Failed') + #13#10 + WebView2Seite,
    mbError, MB_OK, IDOK);
#else
  SuppressibleMsgBox(CustomMessage('WebView2Need') + #13#10#13#10 + CustomMessage('WebView2Missing') + #13#10 + WebView2Seite,
    mbInformation, MB_OK, IDOK);
#endif
end;
#endif

function NeedsPath(Dir: string): Boolean;
var
  Paths: string;
begin
  Paths := '';
  if IsAdminInstallMode then
    RegQueryStringValue(HKEY_LOCAL_MACHINE, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', Paths)
  else
    RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', Paths);
  Result := Pos(';' + Uppercase(Dir) + ';', ';' + Uppercase(Paths) + ';') = 0;
end;

{ Beim Entfernen den Programmordner wieder aus dem PATH nehmen (der Registry-Eintrag oben macht das nicht von selbst rückgängig) }
procedure RemoveFromPath(Dir: string);
var
  Root: Integer;
  Sub, Paths, Work: string;
  P: Integer;
begin
  if IsAdminInstallMode then
  begin
    Root := HKEY_LOCAL_MACHINE;
    Sub := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';
  end
  else
  begin
    Root := HKEY_CURRENT_USER;
    Sub := 'Environment';
  end;
  if not RegQueryStringValue(Root, Sub, 'Path', Paths) then Exit;
  Work := ';' + Paths + ';';
  P := Pos(';' + Uppercase(Dir) + ';', Uppercase(Work));
  while P > 0 do
  begin
    Delete(Work, P, Length(Dir) + 1);
    P := Pos(';' + Uppercase(Dir) + ';', Uppercase(Work));
  end;
  while (Length(Work) > 0) and (Work[1] = ';') do Delete(Work, 1, 1);
  while (Length(Work) > 0) and (Work[Length(Work)] = ';') do Delete(Work, Length(Work), 1);
  if Work <> Paths then RegWriteExpandStringValue(Root, Sub, 'Path', Work);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then RemoveFromPath(ExpandConstant('{app}'));
end;
