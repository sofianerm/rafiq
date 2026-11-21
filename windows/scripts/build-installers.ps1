# Construit les deux installeurs de Rafiq, l'un après l'autre (~6 min chacun) :
#   Rafiq_<version>_x64-setup.exe          général : aucune donnée perso, l'accueil au premier lancement
#   Rafiq-Sofiane_<version>_x64-setup.exe  perso : config de Sofiane intégrée (presets/sofiane.json), sans accueil
# Même identifiant, même dossier d'installation : l'un remplace l'autre. Aucun ne touche à une config
# déjà présente dans %APPDATA%\rafiq, et aucun n'emporte de clé d'API (ai.json n'est jamais intégré).
#
# Mises à jour automatiques : les installeurs sont signés avec la clé de l'updater, gardée hors du dépôt
# (%USERPROFILE%\.tauri\rafiq-updater.key + .password, à sauvegarder : sans elle, plus aucune mise à
# jour ne peut être publiée). Le script écrit ensuite latest.json pour l'installeur général.
# À joindre à la release GitHub : Rafiq_<v>_x64-setup.exe, son .sig et latest.json (pas le perso).
# Usage, depuis windows\ :  powershell -File scripts\build-installers.ps1
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root

$keyDir = Join-Path $env:USERPROFILE '.tauri'
$key = Join-Path $keyDir 'rafiq-updater.key'
if (-not (Test-Path $key)) { throw "updater key not found: $key" }
$env:TAURI_SIGNING_PRIVATE_KEY = [IO.File]::ReadAllText($key).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = [IO.File]::ReadAllText((Join-Path $keyDir 'rafiq-updater.password')).Trim()

cargo build --release --locked -p rafiq-hook --target-dir target/hook
if ($LASTEXITCODE) { throw 'rafiq-hook build failed' }

$version = (Select-String -Path rafiq\Cargo.toml -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$nsis = Join-Path $root 'target\release\bundle\nsis'
$name = "Rafiq_${version}_x64-setup.exe"
$built = Join-Path $nsis $name

Push-Location rafiq
try {
    # Le perso d'abord, mis de côté sous son nom ; puis le général, qui garde le nom par défaut
    npx --yes @tauri-apps/cli@2 build --config tauri.bundle.conf.json --features perso
    if ($LASTEXITCODE) { throw 'perso build failed' }
    Move-Item -Force $built (Join-Path $nsis "Rafiq-Sofiane_${version}_x64-setup.exe")
    Move-Item -Force "$built.sig" (Join-Path $nsis "Rafiq-Sofiane_${version}_x64-setup.exe.sig")

    npx --yes @tauri-apps/cli@2 build --config tauri.bundle.conf.json
    if ($LASTEXITCODE) { throw 'general build failed' }
} finally {
    Pop-Location
    Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY, Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
}

# Le flux lu par l'updater (https://github.com/sofianerm/rafiq/releases/latest/download/latest.json).
# Les installations perso se mettent à jour avec l'installeur général : leur config reste en place.
$feed = [ordered]@{
    version  = $version
    notes    = "Rafiq $version"
    pub_date = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    platforms = [ordered]@{
        'windows-x86_64' = [ordered]@{
            signature = [IO.File]::ReadAllText("$built.sig").Trim()
            url       = "https://github.com/sofianerm/rafiq/releases/download/v$version/$name"
        }
    }
}
[IO.File]::WriteAllText((Join-Path $nsis 'latest.json'), ($feed | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding $false))
Get-ChildItem $nsis | Where-Object { $_.Name -like "*$version*" -or $_.Name -eq 'latest.json' } | Select-Object Name, Length, LastWriteTime
