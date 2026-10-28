# Construit les deux installeurs de Rafiq, l'un après l'autre (~6 min chacun) :
#   Rafiq_<version>_x64-setup.exe          général : aucune donnée perso, l'accueil au premier lancement
#   Rafiq-Sofiane_<version>_x64-setup.exe  perso : config de Sofiane intégrée (presets/sofiane.json), sans accueil
# Même identifiant, même dossier d'installation : l'un remplace l'autre. Aucun ne touche à une config
# déjà présente dans %APPDATA%\rafiq, et aucun n'emporte de clé d'API (ai.json n'est jamais intégré).
# Usage, depuis windows\ :  powershell -File scripts\build-installers.ps1
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

cargo build --release --locked -p rafiq-hook --target-dir target/hook
if ($LASTEXITCODE) { throw 'rafiq-hook build failed' }

$version = (Select-String -Path rafiq\Cargo.toml -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$nsis = Join-Path $root 'target\release\bundle\nsis'
$built = Join-Path $nsis "Rafiq_${version}_x64-setup.exe"

Push-Location rafiq
try {
    # Le perso d'abord, mis de côté sous son nom ; puis le général, qui garde le nom par défaut
    npx --yes @tauri-apps/cli@2 build --config tauri.bundle.conf.json --features perso
    if ($LASTEXITCODE) { throw 'perso build failed' }
    Move-Item -Force $built (Join-Path $nsis "Rafiq-Sofiane_${version}_x64-setup.exe")

    npx --yes @tauri-apps/cli@2 build --config tauri.bundle.conf.json
    if ($LASTEXITCODE) { throw 'general build failed' }
} finally {
    Pop-Location
}
Get-ChildItem $nsis -Filter '*.exe' | Select-Object Name, Length, LastWriteTime
