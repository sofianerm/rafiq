//! Prayer times for the notch's prayer module, from a mosque's public Masjidbox page.
//!
//! The page (`masjidbox.com/horaires-de-priere/<slug>`) is rendered server-side with the next seven
//! days already in it, as `window.REDUX_STATE = '<escaped JSON>'`. Reading that needs no API key,
//! where the JSON API behind the page (`api.masjidbox.com`) refuses a request without one. The page
//! cannot be fetched from the notch itself (CORS), so it is read here and pushed as `prayer`.
//!
//! Seven days are kept on disk, so the countdown keeps working offline for up to a week.
//!
//! Sans mosquée configurée (`prayer_source` = "aladhan", le cas général), les horaires sont
//! calculés par l'API Aladhan (sans clé) pour la ville choisie à l'accueil et la méthode de calcul,
//! et mis dans la même forme que ceux de Masjidbox (sans iqama), pour que le module n'ait qu'un format.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const PAGE: &str = "https://masjidbox.com/horaires-de-priere/";
/// Times change by a minute or two a day; four readings a day is plenty
const POLL_SECS: u64 = 6 * 3600;
const RETRY_SECS: u64 = 300;

static REFRESH: AtomicBool = AtomicBool::new(false);
static SNAP: Mutex<Option<PrayerSnapshot>> = Mutex::new(None);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrayerSnapshot {
    /// "loading" | "ok" | "stale" (kept from disk or a failed read) | "error"
    pub status: String,
    pub slug: String,
    pub name: String,
    pub address: String,
    /// Masjidbox's timetable, one object per day, passed through as is: `date`, `fajr`, `sunrise`,
    /// `dhuhr`, `asr`, `maghrib`, `isha` (ISO 8601 with offset), `iqamah` {same keys, plus
    /// `jumuah` on Fridays}, `jumuah` [..] on Fridays, `hijri` {formatted, ..}, `special` {imsak, iftar}
    pub days: Vec<serde_json::Value>,
    pub fetched_at: u64,
    pub note: String,
}

pub fn request_refresh() {
    REFRESH.store(true, Ordering::Relaxed);
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn store_path() -> PathBuf {
    crate::config::config_path().with_file_name("prayer.json")
}

fn load_persisted() -> PrayerSnapshot {
    std::fs::read_to_string(store_path())
        .ok()
        .and_then(|t| serde_json::from_str::<PrayerSnapshot>(&t).ok())
        .map(|mut s| {
            s.status = "stale".into();
            s
        })
        .unwrap_or_else(|| PrayerSnapshot { status: "loading".into(), ..Default::default() })
}

/// The mosque's slug from config, which may also be the whole page URL pasted in
pub fn slug_of(value: &str) -> String {
    value.trim().trim_end_matches('/').rsplit('/').next().unwrap_or("").split(['?', '#']).next().unwrap_or("").to_string()
}

/// JavaScript's `unescape`: `%XX` is a Latin-1 code unit (the page writes `Gen%E8ve`, not UTF-8),
/// `%uXXXX` a UTF-16 one. Anything malformed is kept literally.
fn js_unescape(s: &str) -> String {
    let b = s.as_bytes();
    let hex = |h: &[u8]| std::str::from_utf8(h).ok().and_then(|h| u16::from_str_radix(h, 16).ok());
    let mut units: Vec<u16> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            if b.get(i + 1) == Some(&b'u') && i + 6 <= b.len() {
                if let Some(u) = hex(&b[i + 2..i + 6]) {
                    units.push(u);
                    i += 6;
                    continue;
                }
            } else if i + 3 <= b.len() {
                if let Some(u) = hex(&b[i + 1..i + 3]) {
                    units.push(u);
                    i += 3;
                    continue;
                }
            }
        }
        // Not an escape: the page's escaped JSON is ASCII, but take a whole UTF-8 char to be safe
        let ch = s[i..].chars().next().unwrap();
        let mut buf = [0u16; 2];
        units.extend_from_slice(ch.encode_utf16(&mut buf));
        i += ch.len_utf8();
    }
    String::from_utf16_lossy(&units)
}

/// Pulls the mosque out of the page's HTML
fn parse_page(html: &str, slug: &str) -> Result<PrayerSnapshot, String> {
    let marker = "window.REDUX_STATE = '";
    let start = html.find(marker).ok_or("the page has no REDUX_STATE (layout changed?)")? + marker.len();
    let len = html[start..].find('\'').ok_or("REDUX_STATE is not terminated")?;
    let json: serde_json::Value =
        serde_json::from_str(&js_unescape(&html[start..start + len])).map_err(|e| format!("REDUX_STATE: {e}"))?;
    let m = &json["masjidbox"]["masjidboxAthany"];
    let days = m["timetable"].as_array().cloned().unwrap_or_default();
    if days.is_empty() {
        return Err("no timetable on the page (unknown mosque?)".into());
    }
    Ok(PrayerSnapshot {
        status: "ok".into(),
        slug: slug.to_string(),
        name: m["name"].as_str().unwrap_or("").to_string(),
        address: m["address"].as_str().unwrap_or("").to_string(),
        days,
        fetched_at: now_ms(),
        note: String::new(),
    })
}

fn read_once(slug: &str) -> Result<PrayerSnapshot, String> {
    if slug.is_empty() {
        return Err("no mosque set (prayer_masjid in config.json)".into());
    }
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build();
    let html = agent
        .get(&format!("{PAGE}{slug}"))
        .set("User-Agent", concat!("Rafiq/", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, _) => format!("HTTP {code}"),
            other => other.to_string(),
        })?
        .into_string()
        .map_err(|e| e.to_string())?;
    parse_page(&html, slug)
}

const ALADHAN: &str = "https://api.aladhan.com/v1/calendar/";

/// Où lire les horaires, d'après la config. `key` identifie la source : un instantané gardé sur
/// disque pour une autre source (autre ville, autre mosquée) n'est plus montré.
enum Source {
    Masjidbox(String),
    Aladhan { lat: f64, lon: f64, method: u32, city: String },
    Missing,
}

impl Source {
    fn from_config(c: &crate::config::Config) -> Source {
        if c.prayer_source == "masjidbox" {
            return Source::Masjidbox(slug_of(&c.prayer_masjid));
        }
        match &c.city {
            Some(city) => Source::Aladhan { lat: city.lat, lon: city.lon, method: c.prayer_method, city: city.name.clone() },
            None => Source::Missing,
        }
    }
    fn key(&self) -> String {
        match self {
            Source::Masjidbox(slug) => slug.clone(),
            Source::Aladhan { lat, lon, method, .. } => format!("aladhan:{lat:.4},{lon:.4}:{method}"),
            Source::Missing => String::new(),
        }
    }
    fn read(&self) -> Result<PrayerSnapshot, String> {
        match self {
            Source::Masjidbox(slug) => read_once(slug),
            Source::Aladhan { lat, lon, method, city } => read_aladhan(*lat, *lon, *method, city, &self.key()),
            Source::Missing => Err("choisis ta ville dans les Settings → Profil".into()),
        }
    }
}

/// Le nom des méthodes de calcul qu'on propose, comme Aladhan les numérote.
fn method_name(method: u32) -> &'static str {
    match method {
        1 => "Université des sciences islamiques, Karachi",
        2 => "ISNA (Amérique du Nord)",
        3 => "Ligue islamique mondiale",
        4 => "Umm al-Qura, La Mecque",
        5 => "Autorité égyptienne",
        12 => "UOIF (France)",
        13 => "Diyanet (Turquie)",
        15 => "Moonsighting Committee",
        _ => "Aladhan",
    }
}

/// "2026-10-26T05:27:00+01:00 (CET)" ou "05:27 (CET)" : on garde la date ISO, sans la zone en clair
fn iso_of(v: &serde_json::Value) -> Option<String> {
    let s = v.as_str()?.split(' ').next()?;
    chrono::DateTime::parse_from_rfc3339(s).ok().map(|_| s.to_string())
}

/// Un jour du calendrier d'Aladhan, au format de Masjidbox (voir `PrayerSnapshot::days`)
fn aladhan_day(d: &serde_json::Value) -> Option<serde_json::Value> {
    let t = &d["timings"];
    let fajr = iso_of(&t["Fajr"])?;
    let mut day = serde_json::json!({
        // Minuit du jour, dans le décalage horaire de ce jour-là
        "date": format!("{}T00:00:00{}", &fajr[..10], &fajr[19..]),
        "fajr": fajr,
    });
    for (ours, theirs) in [("sunrise", "Sunrise"), ("dhuhr", "Dhuhr"), ("asr", "Asr"), ("maghrib", "Maghrib"), ("isha", "Isha")] {
        day[ours] = serde_json::Value::String(iso_of(&t[theirs])?);
    }
    let h = &d["date"]["hijri"];
    if let (Some(n), Some(m), Some(y)) = (h["day"].as_str(), h["month"]["ar"].as_str(), h["year"].as_str()) {
        day["hijri"] = serde_json::json!({ "formatted": format!("{} {m} {y}", n.trim_start_matches('0')) });
    }
    Some(day)
}

fn read_aladhan(lat: f64, lon: f64, method: u32, city: &str, key: &str) -> Result<PrayerSnapshot, String> {
    use chrono::Datelike;
    let today = chrono::Local::now().date_naive();
    let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build();
    let mut days = Vec::new();
    // Le mois en cours, et le suivant quand la semaine qui vient déborde dessus
    let mut months = vec![(today.year(), today.month())];
    let later = today + chrono::Duration::days(7);
    if later.month() != today.month() {
        months.push((later.year(), later.month()));
    }
    for (y, m) in months {
        let v: serde_json::Value = agent
            .get(&format!("{ALADHAN}{y}/{m}"))
            .query("latitude", &format!("{lat:.4}"))
            .query("longitude", &format!("{lon:.4}"))
            .query("method", &method.to_string())
            .query("iso8601", "true")
            .set("User-Agent", concat!("Rafiq/", env!("CARGO_PKG_VERSION")))
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, _) => format!("HTTP {code}"),
                other => other.to_string(),
            })?
            .into_json()
            .map_err(|e| e.to_string())?;
        days.extend(v["data"].as_array().into_iter().flatten().filter_map(aladhan_day));
    }
    // D'hier (Icha reste à cocher jusqu'à Fajr) à dans une semaine
    let from = (today - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
    let to = later.format("%Y-%m-%d").to_string();
    days.retain(|d| d["date"].as_str().and_then(|s| s.get(..10)).is_some_and(|s| s >= from.as_str() && s <= to.as_str()));
    if days.is_empty() {
        return Err("Aladhan n'a renvoyé aucun horaire".into());
    }
    Ok(PrayerSnapshot {
        status: "ok".into(),
        slug: key.to_string(),
        name: city.to_string(),
        address: method_name(method).to_string(),
        days,
        fetched_at: now_ms(),
        note: String::new(),
    })
}

fn current() -> PrayerSnapshot {
    SNAP.lock().unwrap().clone().unwrap_or_else(load_persisted)
}

fn broadcast(app: &AppHandle, snap: PrayerSnapshot, save: bool) {
    if save {
        if let Ok(t) = serde_json::to_string_pretty(&snap) {
            let _ = std::fs::write(store_path(), t);
        }
    }
    *SNAP.lock().unwrap() = Some(snap.clone());
    let _ = app.emit("prayer", &snap);
}

fn sleep_interruptible(secs: u64) {
    for _ in 0..secs {
        if REFRESH.swap(false, Ordering::Relaxed) {
            return;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn start(app: AppHandle) {
    *SNAP.lock().unwrap() = Some(load_persisted());
    let notifier = app.clone();
    std::thread::spawn(move || notify_loop(notifier));
    std::thread::spawn(move || loop {
        let source = Source::from_config(&app.state::<crate::AppState>().cfg.lock().unwrap());
        let wait = match source.read() {
            Ok(snap) => {
                broadcast(&app, snap, true);
                POLL_SECS
            }
            Err(e) => {
                crate::applog(&format!("prayer: {e}"));
                // Keep the days already known (the countdown still works from them), marked stale
                let mut snap = current();
                // Les horaires d'une autre ville ou d'une autre mosquée seraient faux : on les oublie
                if snap.slug != source.key() {
                    snap = PrayerSnapshot { slug: source.key(), ..Default::default() };
                }
                snap.status = if snap.days.is_empty() { "error".into() } else { "stale".into() };
                snap.note = e;
                broadcast(&app, snap, false);
                RETRY_SECS
            }
        };
        sleep_interruptible(wait);
    });
}

// ---- Notifications ------------------------------------------------------------------------------
// Two per prayer, both silent Windows toasts: one at the adhan, and one five minutes before the
// prayer's time runs out, unless it has been ticked as prayed (in the notch card, or with the
// toast's own button). A prayer's time runs out at the next one; Fajr's at sunrise, and Isha's at
// the Islamic midnight, halfway from Maghrib to the next Fajr.

const REMIND_BEFORE_MS: i64 = 5 * 60 * 1000;
/// A toast fires only close to its time, so starting the app mid-afternoon does not replay the morning
const FIRE_WINDOW_MS: i64 = 3 * 60 * 1000;
const PRAYERS: [(&str, &str); 5] = [("fajr", "Fajr"), ("dhuhr", "Dohr"), ("asr", "Asr"), ("maghrib", "Maghrib"), ("isha", "Icha")];

static FIRED: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// "YYYY-MM-DD:dhuhr" for each prayer ticked as prayed; None until first read from disk
static PRAYED: Mutex<Option<std::collections::BTreeSet<String>>> = Mutex::new(None);

fn prayed_path() -> PathBuf {
    crate::config::config_path().with_file_name("prayer-done.json")
}

fn with_prayed<R>(f: impl FnOnce(&mut std::collections::BTreeSet<String>) -> R) -> R {
    let mut guard = PRAYED.lock().unwrap();
    let set = guard.get_or_insert_with(|| {
        std::fs::read_to_string(prayed_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    });
    f(set)
}

fn prayed_list() -> Vec<String> {
    with_prayed(|s| s.iter().cloned().collect())
}

fn set_prayed_inner(key: &str, done: bool) {
    with_prayed(|s| {
        if done {
            s.insert(key.to_string());
        } else {
            s.remove(key);
        }
        // A week is all anyone looks back on; the keys start with the date, so they sort by it
        // (the test toast's key does not, and goes too)
        let cutoff = (chrono::Local::now() - chrono::Duration::days(7)).format("%Y-%m-%d").to_string();
        s.retain(|k| k.as_str() >= cutoff.as_str() && k.starts_with(|c: char| c.is_ascii_digit()));
        if let Ok(t) = serde_json::to_string(&*s) {
            let _ = std::fs::write(prayed_path(), t);
        }
    });
}

#[derive(Debug, Clone, PartialEq)]
struct Alert {
    id: String,
    at_ms: i64,
    title: String,
    body: String,
    /// Set on a reminder: the prayer it is about, which suppresses it once ticked
    prayed_key: Option<String>,
}

fn ts(v: &serde_json::Value) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(v.as_str()?).ok().map(|d| d.timestamp_millis())
}

fn clock(ms: i64) -> String {
    use chrono::TimeZone;
    chrono::Local.timestamp_millis_opt(ms).single().map(|t| t.format("%H:%M").to_string()).unwrap_or_default()
}

fn alerts(days: &[serde_json::Value]) -> Vec<Alert> {
    let mut out = Vec::new();
    for (i, d) in days.iter().enumerate() {
        let date = d["date"].as_str().and_then(|s| s.get(..10)).unwrap_or("").to_string();
        for (k, name) in PRAYERS {
            let Some(at) = ts(&d[k]) else { continue };
            let iqama = ts(&d["iqamah"][k]).map(|t| format!(" · iqama {}", clock(t))).unwrap_or_default();
            out.push(Alert {
                id: format!("adhan:{date}:{k}"),
                at_ms: at,
                title: format!("Adhan — {name}"),
                body: format!("{}{iqama}", clock(at)),
                prayed_key: None,
            });
            let end = match k {
                "fajr" => ts(&d["sunrise"]).map(|t| (t, "Chourouq")),
                "dhuhr" => ts(&d["asr"]).map(|t| (t, "Asr")),
                "asr" => ts(&d["maghrib"]).map(|t| (t, "Maghrib")),
                "maghrib" => ts(&d["isha"]).map(|t| (t, "Icha")),
                _ => match (ts(&d["maghrib"]), days.get(i + 1).and_then(|n| ts(&n["fajr"]))) {
                    (Some(m), Some(f)) => Some((m + (f - m) / 2, "Minuit")),
                    _ => None,
                },
            };
            if let Some((end, what)) = end {
                out.push(Alert {
                    id: format!("remind:{date}:{k}"),
                    at_ms: end - REMIND_BEFORE_MS,
                    title: format!("{name} se termine dans 5 min"),
                    body: format!("{what} à {} — as-tu prié {name} ?", clock(end)),
                    prayed_key: Some(format!("{date}:{k}")),
                });
            }
        }
        for (n, v) in d["jumuah"].as_array().into_iter().flatten().enumerate() {
            let Some(at) = ts(v) else { continue };
            let iqama = ts(&d["iqamah"]["jumuah"][n]).map(|t| format!(" · iqama {}", clock(t))).unwrap_or_default();
            out.push(Alert {
                id: format!("adhan:{date}:jumuah{n}"),
                at_ms: at,
                title: "Joumouâa".into(),
                body: format!("{}{iqama}", clock(at)),
                prayed_key: None,
            });
        }
    }
    out
}

/// The toast's app id. An installed copy is registered under the bundle identifier; a build run
/// from target\ is not registered at all, and Windows drops a toast from an unknown id, so it
/// borrows PowerShell's, as the Tauri notification plugin does.
#[cfg(windows)]
fn app_id(app: &AppHandle) -> String {
    let dev = std::env::current_exe().map(|p| p.to_string_lossy().contains("\\target\\")).unwrap_or(true);
    if dev {
        tauri_winrt_notification::Toast::POWERSHELL_APP_ID.to_string()
    } else {
        app.config().identifier.clone()
    }
}

#[cfg(windows)]
fn toast(app: &AppHandle, a: &Alert) {
    use tauri_winrt_notification::{Duration as ToastDuration, Toast};
    let mut t = Toast::new(&app_id(app)).title(&a.title).text1(&a.body).sound(None).duration(ToastDuration::Short);
    if let Some(key) = a.prayed_key.clone() {
        let app = app.clone();
        t = t.add_button("J'ai prié", "prayed").on_activated(move |action| {
            if action.as_deref() == Some("prayed") {
                set_prayed_inner(&key, true);
                let _ = app.emit("prayer_done", prayed_list());
            }
            Ok(())
        });
    }
    if let Err(e) = t.show() {
        crate::applog(&format!("prayer: toast failed: {e}"));
    }
}

#[cfg(not(windows))]
fn toast(_app: &AppHandle, _a: &Alert) {}

/// A one-off silent toast for the other modules (Clippy's errors), with no button
pub fn notify(app: &AppHandle, title: &str, body: &str) {
    toast(app, &Alert { id: String::new(), at_ms: 0, title: title.into(), body: body.into(), prayed_key: None });
}

fn notify_loop(app: AppHandle) {
    loop {
        std::thread::sleep(Duration::from_secs(15));
        if !app.state::<crate::AppState>().cfg.lock().unwrap().prayer_notify {
            continue;
        }
        let now = now_ms() as i64;
        for a in alerts(&current().days) {
            if now < a.at_ms || now >= a.at_ms + FIRE_WINDOW_MS {
                continue;
            }
            if a.prayed_key.as_deref().is_some_and(|k| with_prayed(|s| s.contains(k))) {
                continue;
            }
            {
                let mut fired = FIRED.lock().unwrap();
                if fired.contains(&a.id) {
                    continue;
                }
                fired.push(a.id.clone());
                if fired.len() > 100 {
                    fired.drain(..50);
                }
            }
            toast(&app, &a);
        }
    }
}

#[tauri::command]
pub fn get_prayed() -> Vec<String> {
    prayed_list()
}

#[tauri::command]
pub fn set_prayed(app: AppHandle, key: String, done: bool) {
    set_prayed_inner(&key, done);
    let _ = app.emit("prayer_done", prayed_list());
}

#[tauri::command]
pub fn prayer_test_notify(app: AppHandle) {
    toast(
        &app,
        &Alert {
            id: String::new(),
            at_ms: 0,
            title: "Dohr se termine dans 5 min".into(),
            body: "Test — les rappels arriveront comme ceci".into(),
            prayed_key: Some("test".into()),
        },
    );
}

#[tauri::command]
pub fn get_prayer() -> PrayerSnapshot {
    current()
}

#[tauri::command]
pub fn refresh_prayer() {
    request_refresh();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescape_latin1_and_utf16() {
        assert_eq!(js_unescape("Gen%E8ve"), "Genève");
        assert_eq!(js_unescape("%u0627%u0644"), "ال");
        assert_eq!(js_unescape("100%"), "100%");
        assert_eq!(js_unescape("%7B%22a%22%3A1%7D"), "{\"a\":1}");
    }

    #[test]
    fn alerts_adhan_and_reminders() {
        let days: Vec<serde_json::Value> = serde_json::from_str(
            r#"[{"date":"2026-09-25T00:00:00+02:00","fajr":"2026-09-25T06:00:00+02:00","sunrise":"2026-09-25T07:30:00+02:00",
                 "dhuhr":"2026-09-25T13:30:00+02:00","asr":"2026-09-25T16:45:00+02:00","maghrib":"2026-09-25T19:30:00+02:00",
                 "isha":"2026-09-25T21:00:00+02:00","iqamah":{"dhuhr":"2026-09-25T13:40:00+02:00","jumuah":["2026-09-25T12:55:00+02:00"]},
                 "jumuah":["2026-09-25T12:30:00+02:00"]},
                {"date":"2026-09-26T00:00:00+02:00","fajr":"2026-09-26T05:30:00+02:00"}]"#,
        )
        .unwrap();
        let a = alerts(&days);
        let at = |id: &str| a.iter().find(|x| x.id == id).map(|x| x.at_ms).unwrap();
        let t = |s: &str| chrono::DateTime::parse_from_rfc3339(s).unwrap().timestamp_millis();
        assert_eq!(at("adhan:2026-09-25:dhuhr"), t("2026-09-25T13:30:00+02:00"));
        // Dohr's time runs out at Asr, Fajr's at sunrise
        assert_eq!(at("remind:2026-09-25:dhuhr"), t("2026-09-25T16:40:00+02:00"));
        assert_eq!(at("remind:2026-09-25:fajr"), t("2026-09-25T07:25:00+02:00"));
        // Isha's at the Islamic midnight: halfway from 19:30 to 05:30 is 00:30
        assert_eq!(at("remind:2026-09-25:isha"), t("2026-09-26T00:25:00+02:00"));
        assert_eq!(at("adhan:2026-09-25:jumuah0"), t("2026-09-25T12:30:00+02:00"));
        let r = a.iter().find(|x| x.id == "remind:2026-09-25:dhuhr").unwrap();
        assert_eq!(r.prayed_key.as_deref(), Some("2026-09-25:dhuhr"));
        // The next day has no Isha, so no reminder can be worked out for it
        assert!(!a.iter().any(|x| x.id == "remind:2026-09-26:isha"));
    }

    #[test]
    fn slug_from_url_or_slug() {
        assert_eq!(slug_of("https://masjidbox.com/horaires-de-priere/fondation-x/"), "fondation-x");
        assert_eq!(slug_of("fondation-x"), "fondation-x");
        assert_eq!(slug_of("https://masjidbox.com/prayer-times/abc?x=1"), "abc");
    }

    #[test]
    fn parses_embedded_state() {
        let state = r#"{"masjidbox":{"masjidboxAthany":{"name":"Mosquée","address":"1 rue","timetable":[{"date":"2026-09-23T00:00:00+02:00","fajr":"2026-09-23T06:00:00+02:00"}]}}}"#;
        let escaped: String = state
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c.to_string() } else { format!("%{:02X}", c as u32) })
            .collect();
        let html = format!("<script>window.REDUX_STATE = '{escaped}';</script>");
        let s = parse_page(&html, "m").unwrap();
        assert_eq!(s.name, "Mosquée");
        assert_eq!(s.days.len(), 1);
        assert_eq!(s.days[0]["fajr"], "2026-09-23T06:00:00+02:00");
        assert!(parse_page("<html></html>", "m").is_err());
    }

    /// Réseau : `cargo test -- --ignored live_aladhan`
    #[test]
    #[ignore]
    fn live_aladhan() {
        let s = read_aladhan(45.749, 4.848, 12, "Lyon", "k").unwrap();
        assert!(s.days.len() >= 7, "{} days", s.days.len());
        eprintln!("{}", s.days[1]);
    }

    #[test]
    fn aladhan_day_takes_the_masjidbox_shape() {
        let d = serde_json::json!({
            "timings":{"Fajr":"2026-10-26T05:27:00+01:00 (CET)","Sunrise":"2026-10-26T07:08:00+01:00","Dhuhr":"2026-10-26T12:19:00+01:00",
                       "Asr":"2026-10-26T15:04:00+01:00","Maghrib":"2026-10-26T17:31:00+01:00","Isha":"2026-10-26T19:05:00+01:00"},
            "date":{"hijri":{"day":"05","month":{"ar":"جمادى الأولى"},"year":"1448"}}
        });
        let day = aladhan_day(&d).unwrap();
        assert_eq!(day["date"], "2026-10-26T00:00:00+01:00");
        assert_eq!(day["fajr"], "2026-10-26T05:27:00+01:00");
        assert_eq!(day["isha"], "2026-10-26T19:05:00+01:00");
        assert_eq!(day["hijri"]["formatted"], "5 جمادى الأولى 1448");
        // Same alerts as a Masjidbox day, minus the iqama
        let a = alerts(&[day]);
        assert!(a.iter().any(|x| x.id == "remind:2026-10-26:dhuhr"));
        assert!(aladhan_day(&serde_json::json!({"timings":{}})).is_none());
    }
}
