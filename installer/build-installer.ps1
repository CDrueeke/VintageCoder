# Erzeugt den Windows-Installer: Staging nach FEATURES.md (tools\release.py, baut die Programme), dann Inno Setup.
# Aufruf (im Projektordner):  powershell -File installer\build-installer.ps1
# Voraussetzung: Rust (cargo), Python 3 und Inno Setup 6 (winget install JRSoftware.InnoSetup). Optional: $env:VC_STAGE = Ausgabeordner des Staging.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

# Der Build-Ordner aus .cargo/config.toml ist ein Mac-Pfad: lokal überschreiben (außer der Aufrufer hat schon einen gesetzt)
if (-not $env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = Join-Path $env:LOCALAPPDATA 'vintagecoder-target' }

$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches[0].Groups[1].Value

# Staging nach FEATURES.md erzeugen (baut die Programme; Auswahl und Abhaengigkeiten prueft das Skript)
$stage = if ($env:VC_STAGE) { $env:VC_STAGE } else { Join-Path $root 'build\stage-windows' }
python tools\release.py stage --os windows --out $stage --build --force
if ($LASTEXITCODE -ne 0) { throw 'release.py stage fehlgeschlagen' }

$iscc = @(
  (Get-Command ISCC.exe -ErrorAction SilentlyContinue).Source,
  "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
  "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
  "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
) | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
if (-not $iscc) { throw 'Inno Setup 6 nicht gefunden (winget install JRSoftware.InnoSetup)' }

& $iscc "/DAppVersion=$version" "/DStage=$stage" (Join-Path $PSScriptRoot 'vintagecoder.iss')
if ($LASTEXITCODE -ne 0) { throw 'ISCC fehlgeschlagen' }
Write-Host "Fertig: $(Join-Path $PSScriptRoot "output\VintageCoder-$version-Setup.exe")"
