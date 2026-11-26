# Erzeugt den Windows-Installer: Staging nach FEATURES.md (tools\release.py, baut die Programme), dann Inno Setup.
# Aufruf (im Projektordner):  powershell -File installer\build-installer.ps1
# Voraussetzung: Rust (cargo), Node.js mit npm (die Oberfläche ui/ ist eine Tauri-/Svelte-Anwendung), Python 3 und Inno Setup 6 (winget install JRSoftware.InnoSetup).
# Optional: $env:VC_STAGE = Ausgabeordner des Staging.
# WebView2: Die Oberfläche braucht die Microsoft-WebView2-Laufzeit. Der Installer prüft sie und führt, falls sie fehlt, den Evergreen-Bootstrapper
#   MicrosoftEdgeWebview2Setup.exe leise aus (Internet beim Installieren nötig). Die Datei liegt NICHT im Repository: dieses Skript lädt sie von
#   https://go.microsoft.com/fwlink/p/?LinkId=2124703 nach build\webview2 und prüft die Authenticode-Signatur (Microsoft Corporation, gültig).
#   $env:VC_WEBVIEW2_BOOTSTRAPPER = Pfad einer vorhandenen Datei (statt Download), $env:VC_WEBVIEW2_SHA256 = erwartete SHA-256 (optional, zusätzlich zur Signatur;
#   Microsoft aktualisiert die Datei, daher kein fester Wert im Repository). $env:VC_WEBVIEW2_OHNE = 1 baut einen Installer ohne Bootstrapper (nur Prüfung mit Hinweis).
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

# Der Build-Ordner aus .cargo/config.toml ist ein Mac-Pfad: lokal überschreiben (außer der Aufrufer hat schon einen gesetzt)
if (-not $env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = Join-Path $env:LOCALAPPDATA 'vintagecoder-target' }

$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches[0].Groups[1].Value

# Staging nach FEATURES.md erzeugen (baut die Programme; Auswahl und Abhaengigkeiten prueft das Skript)
$stage = if ($env:VC_STAGE) { $env:VC_STAGE } else { Join-Path $root 'build\stage-windows' }
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) { throw 'Node.js/npm nicht gefunden (winget install OpenJS.NodeJS.LTS): die Oberfläche (ui/) wird mit npm gebaut' }
python tools\release.py stage --os windows --out $stage --build --force --oberflaeche tauri
if ($LASTEXITCODE -ne 0) { throw 'release.py stage fehlgeschlagen' }
# Die Tauri-Oberfläche muss das Frontend eingebettet haben (das prüft release.py stage schon; hier noch einmal als Sichtprüfung)
$exe = Join-Path $stage 'vintagecoder.exe'
if (-not [Text.Encoding]::GetEncoding(28591).GetString([IO.File]::ReadAllBytes($exe)).Contains('assets/index-')) { throw 'vintagecoder.exe enthält kein eingebettetes Frontend (assets/index-)' }

# WebView2-Bootstrapper beschaffen und prüfen (siehe Kopf)
$wv2 = $null
if ($env:VC_WEBVIEW2_OHNE -ne '1') {
  $wv2 = if ($env:VC_WEBVIEW2_BOOTSTRAPPER) { $env:VC_WEBVIEW2_BOOTSTRAPPER } else { Join-Path $root 'build\webview2\MicrosoftEdgeWebview2Setup.exe' }
  if (-not (Test-Path $wv2)) {
    New-Item -ItemType Directory -Force -Path (Split-Path $wv2) | Out-Null
    Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $wv2 -UseBasicParsing
  }
  $sig = Get-AuthenticodeSignature $wv2
  if ($sig.Status -ne 'Valid' -or $sig.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') { throw "WebView2-Bootstrapper: Signatur ungültig oder nicht von Microsoft ($($sig.Status))" }
  $hash = (Get-FileHash -Algorithm SHA256 $wv2).Hash
  if ($env:VC_WEBVIEW2_SHA256 -and ($hash -ne $env:VC_WEBVIEW2_SHA256.ToUpper())) { throw "WebView2-Bootstrapper: SHA-256 $hash weicht von VC_WEBVIEW2_SHA256 ab" }
  Write-Host "WebView2-Bootstrapper: $wv2 (SHA-256 $hash, Signatur gültig)"
}

$iscc = @(
  (Get-Command ISCC.exe -ErrorAction SilentlyContinue).Source,
  "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
  "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
  "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) { throw 'Inno Setup 6 nicht gefunden (winget install JRSoftware.InnoSetup)' }

$isccArgs = @("/DAppVersion=$version", "/DStage=$stage")
if ($wv2) { $isccArgs += "/DWebView2Setup=$((Resolve-Path $wv2).Path)" }
& $iscc @isccArgs (Join-Path $PSScriptRoot 'vintagecoder.iss')
if ($LASTEXITCODE -ne 0) { throw 'ISCC fehlgeschlagen' }
Write-Host "Fertig: $(Join-Path $PSScriptRoot "output\VintageCoder-$version-Setup.exe")"
