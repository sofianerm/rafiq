# Rafiq

**A companion that lives on the edge of your Windows screen.** *Rafiq* (رفيق) means "companion" in
Arabic: a small notch on the side of the screen that opens on hover and keeps the useful things one
glance away. Free and open source (MIT).

*[Version française plus bas.](#français)*

## What it shows

| Module | What you get |
|---|---|
| Prayer times | Countdown to the next prayer under the ring, the day's times in the card, a quiet notification at each adhan and five minutes before a prayer's time runs out. Times are calculated for your city (Aladhan, choice of method) or read from your mosque's Masjidbox page (with iqama). |
| Public transport | Next departures at up to four stops, delays included. Worldwide through Transitous, or transport.opendata.ch in Switzerland. |
| Weather | Temperature under the ring, rain risk for the next hours, four-day forecast (Open-Meteo). |
| Clipboard | The last 30 texts you copied, one click away; pin the ones you keep. Password-manager copies are ignored. |
| Clippy (AI) | Select text anywhere, press **Ctrl+Alt+C**: it is proofread, translated or rewritten and pasted back. DeepSeek, OpenAI or Gemini — bring your own API key, stored only on your PC. |
| Name of Allah of the day | One of the 99 names, with its meaning. |
| Screen time | Time spent in each app today, kept on your PC. |
| AI usage | How much of your Claude, Codex, Cursor, Grok, Antigravity or GLM allowance is left, and whether Claude is still working. |

On first launch, Rafiq introduces itself and asks a few questions (your first name, your city, prayer
times, your stops, which modules you want). Everything can be changed later in Settings → Profile,
and the modules can be reordered or hidden.

## Install

Download `Rafiq_<version>_x64-setup.exe` from the [latest release](https://github.com/sofianerm/rafiq/releases/latest) and run it. It installs
for the current user, without administrator rights. The installer is **not code-signed**, so Windows
SmartScreen shows *Windows protected your PC* the first time: choose **More info**, then **Run anyway**.

**macOS** (Apple Silicon and Intel): download `Rafiq_<version>_macOS-universal.dmg`, open it and drag
Rafiq into Applications. The app is **not signed by Apple**, so macOS refuses it the first time: open
**System Settings → Privacy & Security** and click **Open Anyway**, or run once in Terminal
`xattr -cr /Applications/Rafiq.app`. Rafiq lives in the menu bar (no Dock icon); its data is in
`~/Library/Application Support/rafiq`. Not on the Mac yet: Clippy's global shortcut and the
"I prayed" button in notifications. Built by [`.github/workflows/macos.yml`](.github/workflows/macos.yml).

Uninstalling with *Delete the application data* ticked also removes `%APPDATA%\rafiq` (settings,
cached times, API keys).

## Build from source

Prerequisites: Rust (MSVC toolchain), Node.js (for the Tauri CLI), WebView2 (ships with Windows 11).

```powershell
cd windows
cargo build                 # debug build
.\target\debug\rafiq.exe
cargo test
node scripts/check-ui-scripts.mjs

# installer
cargo build --release --locked -p rafiq-hook --target-dir target/hook
cd rafiq
npx @tauri-apps/cli@2 build --config tauri.bundle.conf.json
# → windows\target\release\bundle\nsis\Rafiq_<version>_x64-setup.exe
```

Architecture, providers and the module contract: [`windows/README.md`](windows/README.md) and
[`windows/rafiq/ui/modules/README.md`](windows/rafiq/ui/modules/README.md).

## Data sources and attribution

Rafiq calls these services directly from your PC, without an account or API key (except your own
key for Clippy):

- **Transitous** — public transport worldwide, a community-run service built on MOTIS, used under its
  [usage policy](https://transitous.org/api/). Data from [Transitous and its sources](https://transitous.org/sources/),
  including © [OpenStreetMap](https://www.openstreetmap.org/copyright) contributors.
- **transport.opendata.ch** — Swiss public transport.
- **Aladhan** — calculated prayer times ([aladhan.com](https://aladhan.com/)).
- **Masjidbox** — a mosque's published prayer and iqama times, read from its public page.
- **Open-Meteo** — weather and city search ([open-meteo.com](https://open-meteo.com/), CC BY 4.0).

## License

MIT — see [`LICENSE`](LICENSE).

Designed and built by Sofiane.

---

## Français

**Un compagnon sur le bord de ton écran Windows.** *Rafiq* (رفيق), c'est « compagnon » en arabe : une
petite pastille sur le côté de l'écran, qui s'ouvre au survol et garde l'utile à portée de regard.
Gratuit et open source (MIT).

- **Prière** : décompte jusqu'à la prochaine prière, horaires du jour, notification discrète à l'adhan
  et 5 minutes avant la fin du temps. Calculés pour ta ville (Aladhan) ou ceux de ta mosquée (Masjidbox, avec l'iqama).
- **Transports** : prochains départs à tes arrêts (jusqu'à 4), retards compris. Partout avec Transitous,
  ou transport.opendata.ch en Suisse.
- **Météo**, **presse-papier** (30 derniers copiés, épingles), **Clippy** (Ctrl+Alt+C sur un texte
  sélectionné : correction, traduction, reformulation, avec ta propre clé d'API), **un nom d'Allah par
  jour**, **temps d'écran**, et l'**usage de tes IA**.

Au premier lancement, Rafiq se présente et te pose quelques questions (prénom, ville, prière, arrêts,
modules). Tout se change ensuite dans les Settings → Profil.

**Installation** : télécharge `Rafiq_<version>_x64-setup.exe` dans la [dernière release](https://github.com/sofianerm/rafiq/releases/latest) et lance-le. L'installeur n'est pas signé : au premier
lancement, Windows SmartScreen affiche « Windows a protégé votre ordinateur » → **Informations
complémentaires** → **Exécuter quand même**.

**Mac** : télécharge `Rafiq_<version>_macOS-universal.dmg`, ouvre-le et glisse Rafiq dans Applications.
L'app n'est pas signée par Apple : au premier lancement macOS la refuse → **Réglages Système →
Confidentialité et sécurité** → **Ouvrir quand même** (ou, une fois dans le Terminal :
`xattr -cr /Applications/Rafiq.app`). Rafiq vit dans la barre des menus. Pas encore sur Mac : le
raccourci Ctrl+Alt+C de Clippy et le bouton « J'ai prié » des notifications.

**Sources** : Transitous ([ses sources](https://transitous.org/sources/), © [OpenStreetMap](https://www.openstreetmap.org/copyright)),
transport.opendata.ch, Aladhan, Masjidbox, Open-Meteo.

Licence MIT. Conçu et développé par Sofiane.
