; Inno-Setup-Skript für den VintageCoder (Windows, x64).
; Der Installer packt nur noch das Staging ein, das "python tools\release.py stage --os windows --out <Ordner> --build" erzeugt
; (Programme, data\, Beispiele, Lizenzen, installer.inc). Was drin ist, bestimmt FEATURES.md.
; Bauen: installer\build-installer.ps1 (ruft das Skript auf und dann ISCC).
; Direkt: ISCC.exe /DAppVersion=0.1.0 /DStage=<Staging-Ordner> vintagecoder.iss
; installer.inc im Staging setzt HasIde (mit Entwicklungsumgebung) und HasExamples (mit Beispielen).

#ifndef AppVersion
  #define AppVersion "0.1.0"
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
Name: "german"; MessagesFile: "compiler:Languages\German.isl"

[Tasks]
#ifdef HasIde
Name: "desktopicon"; Description: "Symbol auf dem &Desktop anlegen"; Flags: unchecked
#endif
Name: "addtopath"; Description: "Kommandozeilenprogramm &vcc zum PATH hinzufügen"; Flags: unchecked
#ifdef HasExamples
Name: "examples"; Description: "&Beispielprojekte nach ""Dokumente\VintageCoder\Beispiele"" kopieren"
#endif

[Files]
; Programme (Die App sucht data\ ab dem Ort der .exe nach oben: data\ liegt daher direkt daneben.)
#ifdef HasIde
Source: "{#Stage}\vintagecoder.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\VintageCoder.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\licenses\*"; DestDir: "{app}\licenses"; Flags: ignoreversion recursesubdirs createallsubdirs
#endif
Source: "{#Stage}\vcc.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\vc-lsp.exe"; DestDir: "{app}"; Flags: ignoreversion
; Hardware-Datenbank, Vorlagen, Hilfe (nach Auswahl, mit release.json)
Source: "{#Stage}\data\*"; DestDir: "{app}\data"; Flags: ignoreversion recursesubdirs createallsubdirs
; Lizenz, Hinweise auf fremde Inhalte, Dokumentation, Erweiterungen
Source: "{#Stage}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\THIRD-PARTY-NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\docs\*"; DestDir: "{app}\docs"; Excludes: ".DS_Store"; Flags: ignoreversion recursesubdirs createallsubdirs skipifsourcedoesntexist
Source: "{#Stage}\editors\*"; DestDir: "{app}\editors"; Flags: ignoreversion recursesubdirs createallsubdirs skipifsourcedoesntexist
#ifdef HasExamples
; Beispiele in den Dokumenten des Benutzers (schreibbar, das Bauen legt dort Dateien an); ROM-Auszuege liefert das Staging nie mit
Source: "{#Stage}\examples\*"; DestDir: "{userdocs}\{#AppName}\Beispiele"; Flags: ignoreversion recursesubdirs createallsubdirs onlyifdoesntexist; Tasks: examples
#endif

#ifdef HasIde
[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#Exe}"; WorkingDir: "{userdocs}\{#AppName}"; IconFilename: "{app}\VintageCoder.ico"
Name: "{group}\{#AppName} entfernen"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#Exe}"; WorkingDir: "{userdocs}\{#AppName}"; IconFilename: "{app}\VintageCoder.ico"; Tasks: desktopicon
#endif

[Registry]
#ifdef HasIde
; Dateiendung .vcprj (Projektdatei) mit VintageCoder öffnen
Root: HKA; Subkey: "Software\Classes\.vcprj"; ValueType: string; ValueData: "VintageCoder.Project"; Flags: uninsdeletevalue uninsdeletekeyifempty
Root: HKA; Subkey: "Software\Classes\VintageCoder.Project"; ValueType: string; ValueData: "VintageCoder-Projekt"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\VintageCoder.Project\DefaultIcon"; ValueType: string; ValueData: "{app}\VintageCoder.ico"
Root: HKA; Subkey: "Software\Classes\VintageCoder.Project\shell\open\command"; ValueType: string; ValueData: """{app}\{#Exe}"" ""%1"""
#endif
; PATH (nur wenn gewählt und noch nicht enthalten)
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: (not IsAdminInstallMode) and NeedsPath(ExpandConstant('{app}'))
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: IsAdminInstallMode and NeedsPath(ExpandConstant('{app}'))

#ifdef HasIde
[Run]
Filename: "{app}\{#Exe}"; Description: "{#AppName} starten"; WorkingDir: "{userdocs}\{#AppName}"; Flags: nowait postinstall skipifsilent
#endif

[Code]
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
