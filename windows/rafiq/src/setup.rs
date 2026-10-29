//! Ce qui rend Rafiq général : prénom, ville, source et méthode des horaires de prière, source et
//! arrêts des transports. Rempli par l'accueil (premier lancement), modifiable ensuite dans les
//! Settings → Profil. Le tout vit dans config.json, à côté du reste.

use crate::config::{City, TransitStop};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup {
    pub onboarded: bool,
    pub user_name: String,
    pub city: Option<City>,
    pub prayer_source: String,
    pub prayer_method: u32,
    pub prayer_masjid: String,
    pub prayer_notify: bool,
    pub transit_source: String,
    pub transit_stops: Vec<TransitStop>,
}

#[tauri::command]
pub fn get_setup(app: AppHandle) -> Setup {
    let c = app.state::<crate::AppState>().cfg.lock().unwrap().clone();
    Setup {
        onboarded: c.onboarded,
        user_name: c.user_name,
        city: c.city,
        prayer_source: c.prayer_source,
        prayer_method: c.prayer_method,
        prayer_masjid: c.prayer_masjid,
        prayer_notify: c.prayer_notify,
        transit_source: c.transit_source,
        transit_stops: c.transit_stops,
    }
}

/// Remplace tout le profil d'un coup (la page envoie le profil entier) ; le notch le reçoit en
/// événement `setup`, et les horaires de prière sont relus aussitôt.
#[tauri::command]
pub fn set_setup(app: AppHandle, setup: Setup) {
    {
        let state = app.state::<crate::AppState>();
        let mut c = state.cfg.lock().unwrap();
        c.onboarded = setup.onboarded;
        c.user_name = setup.user_name.trim().chars().take(40).collect();
        c.city = setup.city.clone();
        c.prayer_source = if setup.prayer_source == "masjidbox" { "masjidbox".into() } else { "aladhan".into() };
        c.prayer_method = setup.prayer_method;
        c.prayer_masjid = setup.prayer_masjid.trim().to_string();
        c.prayer_notify = setup.prayer_notify;
        c.transit_source = if setup.transit_source == "opendata" { "opendata".into() } else { "transitous".into() };
        c.transit_stops = setup.transit_stops.iter().take(4).cloned().collect();
        crate::config::save(&c);
    }
    crate::prayer::request_refresh();
    let _ = app.emit("setup", get_setup(app.clone()));
}

/// Les seules pages que les modules peuvent ouvrir : celles que les sources demandent de citer, et
/// celles où l'on crée une clé d'API pour Clippy.
const LINKS: [&str; 8] = [
    "https://transitous.org/sources/",
    "https://www.openstreetmap.org/copyright",
    "https://aladhan.com/",
    "https://open-meteo.com/",
    "https://transport.opendata.ch/",
    // Où créer une clé d'API pour Clippy (accueil)
    "https://platform.deepseek.com/api_keys",
    "https://platform.openai.com/api-keys",
    "https://aistudio.google.com/apikey",
];

#[tauri::command]
pub fn open_link(url: String) {
    if !LINKS.contains(&url.as_str()) {
        return;
    }
    let mut cmd = std::process::Command::new("cmd");
    cmd.args(["/C", "start", "", &url]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let _ = cmd.spawn();
}
